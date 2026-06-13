//! Scenario: pressing Enter in problems panel navigates to the correct line.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_DIAGNOSTIC, CM_OPEN_FILE_FOCUS};
use kairn::lsp::diagnostics::{Diagnostic, Severity};
use txv_core::event::{KeyCode, KeyMod};

fn focus_problems(h: &mut TestHarness) {
    use kairn::handler::downcast_desktop;
    use kairn::slots::{focus_tab_by_title, SlotId};
    let desktop = h.program.desktop_mut();
    if let Some(d) = downcast_desktop(desktop) {
        focus_tab_by_title(d, SlotId::Tools, "Problems");
        d.focus_panel(SlotId::Tools as usize);
    }
    h.run_cycles(2);
}

#[test]
fn problems_enter_goes_to_diagnostic_line() {
    let content: String = (1..=100).map(|i| format!("line{i}\n")).collect();
    let dir = temp_project(&[("big.rs", &content)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(1);

    // Open the file so it's in a tab
    let path = dir.path().join("big.rs");
    let req = OpenFileRequest::new(path.clone());
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(3);

    // Send diagnostic at line 79 (0-indexed) = display line 80
    let uri = path.to_string_lossy().to_string();
    let diags = vec![Diagnostic::new(79, 0, 5, Severity::Error, "error here")];
    // dispatch_command updates problems view via handler
    h.dispatch_command(CM_DIAGNOSTIC, Some(Box::new((uri.clone(), diags.clone()))));
    // push through sink so editor view gets it via on_command
    h.program
        .sink()
        .push_command(CM_DIAGNOSTIC, Some(Box::new((uri, diags))));
    h.run_cycles(3);

    // Focus the Problems tab and press Enter
    focus_problems(&mut h);
    h.inject_key(KeyCode::Enter, KeyMod::NONE);
    h.run_cycles(3);

    // The editor should now show line 80 area
    assert!(
        h.content_contains("line80"),
        "editor should show line80 after problems Enter"
    );
}
