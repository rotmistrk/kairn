//! Minimal demo to see TabPanel's native chrome and dropdown behavior.
//! Run: cargo run --example tabgroup_demo

use txv_core::prelude::*;
use txv_render::CrosstermBackend;
use txv_widgets::tab_bar::TabBarMode;
use txv_widgets::tab_panel::TabPanel;

fn main() {
    let mut backend = CrosstermBackend::new(txv_render::ColorMode::TrueColor);
    backend.enter();

    let (w, h) = backend.size();
    let mut tp = TabPanel::new(TabBarMode::Lru);

    for name in &["main.rs", "lib.rs", "Cargo.toml", "README.md"] {
        let mut ta = txv_widgets::TextArea::new();
        ta.set_content(&format!(
            "Content of {name}\n\nThis is a demo tab.\n\
             Press Alt+0 to open dropdown.\n\
             Type to filter, Enter to select.\n\
             Alt+1..9 selects by index.\nCtrl-Q to quit."
        ));
        tp.insert_tab(*name, Box::new(ta));
    }
    tp.set_dirty(1, true);

    tp.set_bounds(Rect::new(0, 0, w, h));
    let sink = EventSink::new();
    tp.set_sink(sink.clone());

    txv_core::run::run(&mut tp, &mut backend);
}
