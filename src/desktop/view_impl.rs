//! View trait implementation for Desktop.

use txv_core::event::CommandId;
use txv_core::prelude::*;

use super::{Desktop, PANEL_COUNT};

/// Check if a command ID belongs to TiledWorkspace's command space.
fn is_workspace_command(id: CommandId) -> bool {
    use txv_core::commands::CM_TXV_MAX;
    use txv_widgets::tiled_workspace::commands::CM_WORKSPACE_BASE;
    (CM_WORKSPACE_BASE..=CM_TXV_MAX).contains(&id)
}

impl View for Desktop {
    fn bounds(&self) -> Rect {
        self.workspace.bounds()
    }

    fn set_bounds(&mut self, r: Rect) {
        self.workspace.set_bounds(r);
    }

    fn set_sink(&mut self, sink: EventSink) {
        self.workspace.set_sink(sink);
    }

    fn options(&self) -> ViewOptions {
        self.workspace.options()
    }

    fn select(&mut self) {
        self.workspace.select();
    }

    fn unselect(&mut self) {
        self.workspace.unselect();
    }

    fn title(&self) -> &str {
        ""
    }

    fn needs_redraw(&self) -> bool {
        self.workspace.needs_redraw()
    }

    fn draw(&mut self) {
        self.draw_chrome();
        self.draw_children();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Command { id, data } = event {
            let r = self.handle_command(*id, data);
            if r == HandleResult::Consumed {
                return r;
            }
            if is_workspace_command(*id) {
                let r = self.workspace.handle_command_event(*id, data);
                if r == HandleResult::Consumed {
                    return r;
                }
            }
        }
        if matches!(event, Event::Tick) {
            for i in 0..PANEL_COUNT {
                if let Some(child) = self.workspace.child_mut(i) {
                    child.handle(event);
                }
            }
            return HandleResult::Ignored;
        }
        self.workspace.dispatch(event)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn buffer(&self) -> &Buffer {
        self.workspace.buffer()
    }
}

impl Desktop {
    pub(super) fn draw_children(&mut self) {
        let my_bounds = self.workspace.bounds();
        for i in 0..PANEL_COUNT {
            if let Some(child) = self.workspace.child_mut(i) {
                if child.bounds().w > 0 && child.bounds().h > 0 {
                    child.draw();
                }
            }
        }
        let buf_ptr = self.workspace.buffer_mut() as *mut Buffer;
        for i in 0..PANEL_COUNT {
            if let Some(child) = self.workspace.child(i) {
                let cb = child.bounds();
                if cb.w > 0 && cb.h > 0 {
                    let dx = cb.x.saturating_sub(my_bounds.x);
                    let dy = cb.y.saturating_sub(my_bounds.y);
                    unsafe { (*buf_ptr).blit(child.buffer(), dx, dy) };
                }
            }
        }
    }
}
