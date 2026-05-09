//! StatusBar — bottom status line with key→command shortcuts.
//! preprocess:true so it intercepts keys before focused views.

use txv_core::prelude::*;

pub struct StatusItem {
    pub key: KeyEvent,
    pub command: CommandId,
    pub label: String,
}

pub struct StatusBar {
    state: ViewState,
    pub items: Vec<StatusItem>,
    pub context: String,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                preprocess: true,
                focusable: false,
                ..ViewOptions::default()
            }),
            items: Vec::new(),
            context: String::new(),
        }
    }

    pub fn add_item(
        &mut self,
        key: KeyEvent,
        command: CommandId,
        label: impl Into<String>,
    ) {
        self.items.push(StatusItem { key, command, label: label.into() });
        self.state.dirty = true;
    }

    pub fn set_context(&mut self, ctx: impl Into<String>) {
        self.context = ctx.into();
        self.state.dirty = true;
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for StatusBar {
    delegate_view_state!(state);

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        let style = Style {
            attrs: Attrs { reverse: true, ..Attrs::default() },
            ..Style::default()
        };
        surface.hline(b.x, b.y, b.w, ' ', style);
        // Labels left-aligned
        let mut x = b.x;
        for item in &self.items {
            if x >= b.x + b.w {
                break;
            }
            let text = format!(" {} ", item.label);
            surface.print(x, b.y, &text, style);
            x += text.len() as u16;
        }
        // Context right-aligned
        if !self.context.is_empty() {
            let ctx_len = self.context.len() as u16;
            let rx = (b.x + b.w).saturating_sub(ctx_len + 1);
            if rx > x {
                surface.print(rx, b.y, &self.context, style);
            }
        }
    }

    fn handle(
        &mut self,
        event: &Event,
        queue: &mut EventQueue,
    ) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        for item in &self.items {
            if *key == item.key {
                queue.put_command(item.command, None);
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }
}
