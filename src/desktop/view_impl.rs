//! View trait implementation for Desktop.

use txv_core::event::CommandId;
use txv_core::prelude::*;

use super::Desktop;

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
        // Chrome is drawn first as background, then children blit on top
        // (TabPanel's row 0 is transparent, showing chrome through)
        self.draw_chrome();
        self.draw_children();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // Handle kairn-specific commands first
        if let Event::Command { id, data } = event {
            let r = self.handle_command(*id, data);
            if r == HandleResult::Consumed {
                return r;
            }
            // Forward workspace subpanel/layout commands to TiledWorkspace
            if is_workspace_command(*id) {
                let r = self.workspace.handle_command_event(*id, data);
                if r == HandleResult::Consumed {
                    return r;
                }
            }
        }
        // Tick goes to ALL panels (background tabs need it for PTY poll)
        if matches!(event, Event::Tick) {
            for i in 0..super::PANEL_COUNT {
                if let Some(child) = self.workspace.child_mut(i) {
                    child.handle(event);
                }
            }
            return HandleResult::Ignored;
        }
        // All other events: delegate to focused child
        self.workspace.dispatch(event)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn buffer(&self) -> &Buffer {
        self.workspace.buffer()
    }
}
