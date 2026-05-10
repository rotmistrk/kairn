//! Program — the correct way to build a TXV application.
//!
//! Program handles the event loop, three-phase dispatch, draw cycle,
//! resize, and quit. The application only provides:
//! - A desktop view (the main content)
//! - A status bar view (preprocess, key→command translation)
//! - A command handler (what to do when commands arrive)
//!
//! # Example
//!
//! ```ignore
//! use txv_core::prelude::*;
//! use txv_core::program::Program;
//!
//! let desktop = MyDesktop::new();
//! let status = MyStatusBar::new();
//!
//! Program::new(desktop, status)
//!     .run(&mut backend, |ctx| {
//!         match ctx.command {
//!             CM_OPEN_FILE => { /* handle */ }
//!             _ => {}
//!         }
//!     });
//! ```
//!
//! You NEVER manually dispatch events.
//! You NEVER call child.handle() yourself.
//! Program does it all correctly.

use std::time::Duration;

use crate::cell::Style;
use crate::commands::CM_QUIT;
use crate::event::Event;
use crate::geometry::Rect;
use crate::group::GroupState;
use crate::run::Backend;
use crate::surface::Surface;
use crate::view::{EventQueue, HandleResult, View, ViewOptions};

/// Context passed to the command handler.
pub struct CommandContext<'a> {
    /// The command ID.
    pub command: u16,
    /// The command data payload.
    pub data: &'a Option<Box<dyn std::any::Any + Send>>,
    /// Queue to emit new commands.
    pub queue: &'a mut EventQueue,
    /// Access to the desktop (child 1 of the group).
    pub desktop: &'a mut dyn View,
}

/// The TXV application runner. Handles event loop, dispatch, draw.
pub struct Program {
    group: GroupState,
}

impl Program {
    /// Create a new Program with a desktop and status bar.
    ///
    /// The status bar MUST have `preprocess: true` in its ViewOptions.
    /// The desktop is the focused child that receives normal events.
    pub fn new(status_bar: Box<dyn View>, desktop: Box<dyn View>) -> Self {
        let mut group = GroupState::new(ViewOptions {
            focusable: true,
            ..ViewOptions::default()
        });
        // Child 0: status bar (preprocess — sees keys first)
        group.insert(status_bar);
        // Child 1: desktop (focused — gets normal events)
        group.insert(desktop);
        group.focused = 1;

        Self { group }
    }

    /// Run the application event loop.
    ///
    /// `handler` is called for every command emitted by views.
    /// The handler receives a `CommandContext` with the command ID,
    /// data, queue, and mutable access to the desktop.
    ///
    /// The loop runs until CM_QUIT is received.
    pub fn run<F>(&mut self, backend: &mut dyn Backend, mut handler: F)
    where
        F: FnMut(&mut CommandContext),
    {
        backend.enter();
        let (w, h) = backend.size();
        let mut surface = Surface::new(w, h);
        let mut queue = EventQueue::new();

        // Initial layout
        self.layout(w, h);

        loop {
            // Draw (only if dirty)
            if self.group.any_dirty() {
                surface.fill(' ', Style::default());
                for child in &self.group.children {
                    child.draw(&mut surface);
                }
                self.group.view.dirty = false;
                for child in &mut self.group.children {
                    child.mark_redrawn();
                }
                // DEBUG: log cells at row 11, cols 10-15 to catch stale $
                for cx in 10..16 {
                    log::trace!(
                        "cell({},{})=({:?}, fg={:?})",
                        cx,
                        11,
                        surface.cell(cx, 11).ch,
                        surface.cell(cx, 11).style.fg
                    );
                }
                backend.flush(&surface);
            }

            // Poll event
            if let Some(event) = backend.poll_event(Duration::from_millis(50)) {
                if let Event::Resize(nw, nh) = &event {
                    surface = Surface::new(*nw, *nh);
                    self.layout(*nw, *nh);
                }
                // Three-phase dispatch (preprocess → focused → postprocess)
                self.group.dispatch(&event, &mut queue);
            } else {
                // Tick
                self.group.dispatch(&Event::Tick, &mut queue);
            }

            // Process commands from queue
            let events = queue.drain();
            let mut quit = false;
            for ev in events {
                if let Event::Command { id, .. } = &ev {
                    log::debug!("Program: command {}", id);
                    if *id == CM_QUIT {
                        quit = true;
                        break;
                    }
                }
                // Re-dispatch through the group (Desktop handles its own commands)
                if self.group.dispatch(&ev, &mut queue) == HandleResult::Consumed {
                    continue;
                }
                // Unhandled command → app handler (fallback)
                if let Event::Command { id, ref data } = ev {
                    let desktop = &mut *self.group.children[1];
                    let mut ctx = CommandContext {
                        command: id,
                        data,
                        queue: &mut queue,
                        desktop,
                    };
                    handler(&mut ctx);
                }
            }

            if quit {
                break;
            }
        }

        backend.leave();
    }

    /// Run exactly N iterations of the event loop. Same dispatch, same
    /// command handling, same draw as `run()`. Stops after N cycles
    /// instead of looping forever. Does not call enter/leave on backend.
    ///
    /// Each cycle: processes all pending events from backend, dispatches
    /// commands, draws. Suitable for testing with MockBackend.
    pub fn run_cycles(&mut self, backend: &mut dyn Backend, handler: &mut dyn FnMut(&mut CommandContext), n: usize) {
        let (w, h) = backend.size();
        let mut surface = Surface::new(w, h);
        let mut queue = EventQueue::new();

        self.layout(w, h);

        for _ in 0..n {
            // Process all pending events
            while let Some(event) = backend.poll_event(Duration::ZERO) {
                if let Event::Resize(nw, nh) = &event {
                    surface = Surface::new(*nw, *nh);
                    self.layout(*nw, *nh);
                }
                self.group.dispatch(&event, &mut queue);

                // Process commands from queue
                let events = queue.drain();
                for ev in events {
                    if let Event::Command { id, .. } = &ev {
                        if *id == CM_QUIT {
                            // Draw final frame and return
                            surface.fill(' ', Style::default());
                            for child in &self.group.children {
                                child.draw(&mut surface);
                            }
                            backend.flush(&surface);
                            return;
                        }
                    }
                    if self.group.dispatch(&ev, &mut queue) == HandleResult::Consumed {
                        continue;
                    }
                    if let Event::Command { id, ref data } = ev {
                        let desktop = &mut *self.group.children[1];
                        let mut ctx = CommandContext {
                            command: id,
                            data,
                            queue: &mut queue,
                            desktop,
                        };
                        handler(&mut ctx);
                    }
                }
            }

            // Tick (simulates idle — triggers autosave, PTY poll, etc.)
            self.group.dispatch(&Event::Tick, &mut queue);

            // Draw
            surface.fill(' ', Style::default());
            for child in &self.group.children {
                child.draw(&mut surface);
            }
            self.group.view.dirty = false;
            for child in &mut self.group.children {
                child.mark_redrawn();
            }
            backend.flush(&surface);
        }
    }

    /// Compute layout: desktop gets all but last row, status gets last row.
    fn layout(&mut self, w: u16, h: u16) {
        let full = Rect::new(0, 0, w, h);
        self.group.view.bounds = full;

        if h >= 2 {
            // Desktop: everything except last row
            self.group.children[1].set_bounds(Rect::new(0, 0, w, h - 1));
            // Status bar: last row
            self.group.children[0].set_bounds(Rect::new(0, h - 1, w, 1));
        } else {
            // Tiny terminal: just show desktop
            self.group.children[1].set_bounds(full);
            self.group.children[0].set_bounds(Rect::new(0, 0, 0, 0));
        }
    }

    /// Access the desktop view (for setup before run).
    pub fn desktop(&self) -> &dyn View {
        &*self.group.children[1]
    }

    /// Mutable access to the desktop (for setup before run).
    pub fn desktop_mut(&mut self) -> &mut dyn View {
        &mut *self.group.children[1]
    }

    /// Access the status bar (for setup before run).
    pub fn status_bar(&self) -> &dyn View {
        &*self.group.children[0]
    }

    /// Mutable access to the status bar (for setup before run).
    pub fn status_bar_mut(&mut self) -> &mut dyn View {
        &mut *self.group.children[0]
    }
}
