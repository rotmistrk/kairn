//! View trait implementation for Desktop.

use txv_core::event::CommandId;
use txv_core::prelude::*;
use txv_widgets::tiled_workspace::TiledWorkspace;

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
        <TiledWorkspace as View>::draw(&mut self.workspace);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // Intercept kairn-specific commands before TiledWorkspace
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
        // Everything (including unhandled commands) goes to TiledWorkspace
        <TiledWorkspace as View>::handle(&mut self.workspace, event)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn buffer(&self) -> &Buffer {
        self.workspace.buffer()
    }
}
