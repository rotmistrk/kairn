//! Immediate title sync for file open.

use std::path::MAIN_SEPARATOR;

use txv_core::disambiguate::{disambiguate, Side};
use txv_widgets::tab_panel::TabPanel;

use crate::views::editor::EditorView;

pub(crate) fn sync_titles_immediate(
    desktop: &mut txv_widgets::tiled_workspace::TiledWorkspace,
    root: &std::path::Path,
) {
    use crate::slots::SlotId;
    let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) else {
        return;
    };
    for child_idx in 0..sp.child_count() {
        let Some(child) = sp.child_mut(child_idx) else {
            continue;
        };
        let Some(panel) = child.as_any_mut().and_then(|a| a.downcast_mut::<TabPanel>()) else {
            continue;
        };
        let paths: Vec<String> = (0..panel.tab_count())
            .filter_map(|i| {
                let view = panel.view_at_mut(i)?;
                let any = view.as_any_mut()?;
                any.downcast_ref::<EditorView>().map(|ev| {
                    ev.path()
                        .strip_prefix(root)
                        .unwrap_or(ev.path())
                        .to_string_lossy()
                        .to_string()
                })
            })
            .collect();
        let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let labels = disambiguate(&refs, MAIN_SEPARATOR, Side::Right);
        for (i, label) in labels.iter().enumerate() {
            if !label.is_empty() {
                panel.set_title(i, label);
            }
        }
    }
}
