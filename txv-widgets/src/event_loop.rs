//! Central event loop — polls crossterm, fires timers, polls data sources.

use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event as CtEvent};
use crossterm::terminal;
use txv::screen::Screen;

use crate::view::{DrawContext, Event, HandleResult, View};

/// A non-blocking data source that can be polled.
pub trait Pollable: Send {
    /// Check for available data. Must not block.
    fn poll(&mut self) -> Option<Vec<u8>>;
}

/// Unique timer identifier.
pub type TimerId = u64;

/// Whether the loop should continue or quit.
pub enum LoopControl {
    /// Keep running.
    Continue,
    /// Exit the loop.
    Quit,
}

/// Context passed to the handler on each loop iteration.
pub struct RunContext<'a> {
    /// The screen to render to.
    pub screen: &'a mut Screen,
    /// Input events collected this tick.
    pub events: Vec<CtEvent>,
    /// Data from polled sources: (poller_index, data).
    pub poll_data: Vec<(usize, Vec<u8>)>,
}

struct TimerEntry {
    id: TimerId,
    interval: Duration,
    repeat: bool,
    next_fire: Instant,
    callback: Box<dyn FnMut() -> bool>,
}

/// Central event loop that integrates crossterm input, timers, and pollers.
pub struct EventLoop {
    screen: Screen,
    timers: Vec<TimerEntry>,
    pollers: Vec<Box<dyn Pollable>>,
    tick_ms: u64,
    next_timer_id: TimerId,
}

impl EventLoop {
    /// Create a new event loop with the given screen.
    pub fn new(screen: Screen) -> Self {
        Self {
            screen,
            timers: Vec::new(),
            pollers: Vec::new(),
            tick_ms: 50,
            next_timer_id: 1,
        }
    }

    /// Set the tick interval in milliseconds.
    pub fn set_tick_ms(&mut self, ms: u64) {
        self.tick_ms = ms;
    }

    /// Add a timer. Returns an ID for cancellation.
    /// The callback returns `true` to keep the timer alive, `false` to cancel.
    pub fn add_timer(
        &mut self,
        delay_ms: u64,
        repeat: bool,
        callback: Box<dyn FnMut() -> bool>,
    ) -> TimerId {
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        let interval = Duration::from_millis(delay_ms);
        self.timers.push(TimerEntry {
            id,
            interval,
            repeat,
            next_fire: Instant::now() + interval,
            callback,
        });
        id
    }

    /// Cancel a timer by ID.
    pub fn cancel_timer(&mut self, id: TimerId) {
        self.timers.retain(|t| t.id != id);
    }

    /// Add a pollable data source.
    pub fn add_poller(&mut self, poller: Box<dyn Pollable>) {
        self.pollers.push(poller);
    }

    /// Run the event loop with a callback handler.
    /// Enters raw mode and alternate screen. Restores terminal on exit.
    pub fn run<F>(&mut self, mut handler: F) -> io::Result<()>
    where
        F: FnMut(&mut RunContext) -> LoopControl,
    {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        crossterm::execute!(stdout, terminal::EnterAlternateScreen)?;

        let result = self.run_inner(&mut handler, &mut stdout);

        let _ = crossterm::execute!(stdout, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();

        result
    }

    /// Run the event loop with a root View.
    /// Dispatches events to the view, draws it each frame.
    /// The loop exits when the view consumes a CM_QUIT command.
    pub fn run_view(&mut self, root: &mut dyn View) -> io::Result<()> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        crossterm::execute!(stdout, terminal::EnterAlternateScreen)?;

        let result = self.run_view_inner(root, &mut stdout);

        let _ = crossterm::execute!(stdout, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();

        result
    }

    fn run_view_inner(&mut self, root: &mut dyn View, out: &mut impl Write) -> io::Result<()> {
        let mut tick: u64 = 0;
        let mut running = true;
        while running {
            // 1. Poll crossterm events
            let ct_events = self.collect_events()?;

            // 2. Fire expired timers
            self.fire_timers();

            // 3. Poll data sources
            let poll_data = self.collect_poll_data();

            // 4. Convert and dispatch events to root view
            for ct_ev in &ct_events {
                let view_event = match ct_ev {
                    CtEvent::Key(k) => Event::Key(*k),
                    CtEvent::Resize(w, h) => Event::Resize(*w, *h),
                    _ => continue,
                };
                if let Event::Key(_) = &view_event {
                    root.handle(&view_event);
                } else if let Event::Resize(w, h) = &view_event {
                    self.screen.resize(*w, *h);
                    let rect = txv::layout::Rect {
                        x: 0,
                        y: 0,
                        w: *w,
                        h: *h,
                    };
                    root.set_bounds(rect);
                    root.handle(&view_event);
                }
            }

            // Dispatch data events
            for (source_id, payload) in poll_data {
                let ev = Event::Data { source_id, payload };
                root.handle(&ev);
            }

            // Dispatch tick
            root.handle(&Event::Tick);
            tick += 1;

            // Check for quit command
            let quit_ev = Event::Command(crate::view::commands::CM_QUIT);
            if root.handle(&quit_ev) == HandleResult::Consumed {
                running = false;
            }

            // 5. Draw
            let ctx = DrawContext {
                app_focused: true,
                tick,
            };
            let mut surface = self.screen.full_surface();
            root.draw(&mut surface, &ctx);

            // 6. Flush
            self.screen.flush(out)?;
        }
        Ok(())
    }

    fn run_inner<F>(&mut self, handler: &mut F, out: &mut impl Write) -> io::Result<()>
    where
        F: FnMut(&mut RunContext) -> LoopControl,
    {
        loop {
            // 1. Poll crossterm events
            let events = self.collect_events()?;

            // 2. Fire expired timers
            self.fire_timers();

            // 3. Poll data sources
            let poll_data = self.collect_poll_data();

            // 4. Call handler
            let mut ctx = RunContext {
                screen: &mut self.screen,
                events,
                poll_data,
            };
            let control = handler(&mut ctx);

            // 5. Flush screen
            self.screen.flush(out)?;

            // 6. Check loop control
            if matches!(control, LoopControl::Quit) {
                break;
            }
        }
        Ok(())
    }

    fn collect_events(&self) -> io::Result<Vec<CtEvent>> {
        let mut events = Vec::new();
        let timeout = Duration::from_millis(self.tick_ms);
        if event::poll(timeout)? {
            events.push(event::read()?);
            // Drain any additional queued events without blocking
            while event::poll(Duration::ZERO)? {
                events.push(event::read()?);
            }
        }
        Ok(events)
    }

    fn fire_timers(&mut self) {
        let now = Instant::now();
        let mut to_remove = Vec::new();
        for (i, timer) in self.timers.iter_mut().enumerate() {
            if now >= timer.next_fire {
                let keep = (timer.callback)();
                if !keep || !timer.repeat {
                    to_remove.push(i);
                } else {
                    timer.next_fire = now + timer.interval;
                }
            }
        }
        // Remove in reverse order to preserve indices
        for i in to_remove.into_iter().rev() {
            self.timers.remove(i);
        }
    }

    fn collect_poll_data(&mut self) -> Vec<(usize, Vec<u8>)> {
        let mut data = Vec::new();
        for (i, poller) in self.pollers.iter_mut().enumerate() {
            if let Some(bytes) = poller.poll() {
                data.push((i, bytes));
            }
        }
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_event_loop() {
        let screen = Screen::new(80, 24);
        let el = EventLoop::new(screen);
        assert_eq!(el.tick_ms, 50);
        assert!(el.timers.is_empty());
        assert!(el.pollers.is_empty());
    }

    #[test]
    fn set_tick_ms() {
        let screen = Screen::new(80, 24);
        let mut el = EventLoop::new(screen);
        el.set_tick_ms(100);
        assert_eq!(el.tick_ms, 100);
    }

    #[test]
    fn add_and_cancel_timer() {
        let screen = Screen::new(80, 24);
        let mut el = EventLoop::new(screen);
        let id = el.add_timer(100, false, Box::new(|| true));
        assert_eq!(el.timers.len(), 1);
        el.cancel_timer(id);
        assert!(el.timers.is_empty());
    }

    #[test]
    fn cancel_nonexistent_timer() {
        let screen = Screen::new(80, 24);
        let mut el = EventLoop::new(screen);
        el.add_timer(100, false, Box::new(|| true));
        el.cancel_timer(999);
        assert_eq!(el.timers.len(), 1); // unchanged
    }

    #[test]
    fn timer_ids_increment() {
        let screen = Screen::new(80, 24);
        let mut el = EventLoop::new(screen);
        let id1 = el.add_timer(100, false, Box::new(|| true));
        let id2 = el.add_timer(200, false, Box::new(|| true));
        assert_eq!(id2, id1 + 1);
    }

    #[test]
    fn fire_expired_one_shot_timer() {
        let screen = Screen::new(80, 24);
        let mut el = EventLoop::new(screen);
        let fired = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let fired_clone = fired.clone();
        el.timers.push(TimerEntry {
            id: 1,
            interval: Duration::ZERO,
            repeat: false,
            next_fire: Instant::now(),
            callback: Box::new(move || {
                fired_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                true
            }),
        });
        el.fire_timers();
        assert!(fired.load(std::sync::atomic::Ordering::SeqCst));
        assert!(el.timers.is_empty()); // one-shot removed
    }

    #[test]
    fn fire_repeating_timer_stays() {
        let screen = Screen::new(80, 24);
        let mut el = EventLoop::new(screen);
        el.timers.push(TimerEntry {
            id: 1,
            interval: Duration::from_millis(100),
            repeat: true,
            next_fire: Instant::now(),
            callback: Box::new(|| true),
        });
        el.fire_timers();
        assert_eq!(el.timers.len(), 1); // still there
    }

    #[test]
    fn fire_repeating_timer_callback_false_removes() {
        let screen = Screen::new(80, 24);
        let mut el = EventLoop::new(screen);
        el.timers.push(TimerEntry {
            id: 1,
            interval: Duration::from_millis(100),
            repeat: true,
            next_fire: Instant::now(),
            callback: Box::new(|| false),
        });
        el.fire_timers();
        assert!(el.timers.is_empty());
    }

    struct TestPoller {
        data: Option<Vec<u8>>,
    }

    impl Pollable for TestPoller {
        fn poll(&mut self) -> Option<Vec<u8>> {
            self.data.take()
        }
    }

    #[test]
    fn add_poller_and_collect() {
        let screen = Screen::new(80, 24);
        let mut el = EventLoop::new(screen);
        el.add_poller(Box::new(TestPoller {
            data: Some(b"hello".to_vec()),
        }));
        el.add_poller(Box::new(TestPoller { data: None }));
        let data = el.collect_poll_data();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].0, 0);
        assert_eq!(data[0].1, b"hello");
    }

    #[test]
    fn collect_poll_data_empty() {
        let screen = Screen::new(80, 24);
        let mut el = EventLoop::new(screen);
        el.add_poller(Box::new(TestPoller { data: None }));
        let data = el.collect_poll_data();
        assert!(data.is_empty());
    }

    #[test]
    fn loop_control_variants() {
        assert!(matches!(LoopControl::Continue, LoopControl::Continue));
        assert!(matches!(LoopControl::Quit, LoopControl::Quit));
    }
}
