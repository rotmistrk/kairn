//! kairn — TUI IDE entry point.
use txv_core::view::View;

use std::path::PathBuf;

use clap::Parser;
use txv_core::program::Program;
use txv_render::backend::CrosstermBackend;
use txv_render::color::detect_color_mode;

use kairn::commands::*;
use kairn::desktop::{SlotId, SlottedDesktop};
use kairn::status::KairnStatusBar;
use kairn::views::editor::EditorView;
use kairn::views::terminal::TerminalView;
use kairn::views::tree::FileTreeView;
use kairn::broker::{FileBroker, OpenResult};
use kairn::completer::CommandCompleter;

#[derive(Parser)]
#[command(name = "kairn", about = "TUI IDE")]
struct Cli {
    /// Directory to open
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let root_dir = std::fs::canonicalize(&cli.path)?;

    // Build desktop
    let mut desktop = SlottedDesktop::new();
    let tree = FileTreeView::new(root_dir.clone());
    desktop.insert_tab(SlotId::Left, "Files", Box::new(tree));
    let term = TerminalView::new("Shell");
    desktop.insert_tab(SlotId::Right, "Shell", Box::new(term));

    // Build status bar
    let mut status = KairnStatusBar::new();
    status.set_completer(Box::new(CommandCompleter));

    // Build program
    let mut program = Program::new(Box::new(status), Box::new(desktop));

    // App state
    let mut broker = FileBroker::new();

    // Run
    let color_mode = detect_color_mode();
    let mut backend = CrosstermBackend::new(color_mode);

    program.run(&mut backend, |ctx| {
        match ctx.command {
            CM_OPEN_FILE => {
                let Some(data) = ctx.data.as_ref() else { return };
                let Some(path) = data.downcast_ref::<PathBuf>() else { return };
                let path_str = path.to_string_lossy().to_string();

                match broker.open(&path_str, SlotId::Center, 0) {
                    OpenResult::AlreadyOpen { .. } => {
                        // TODO: focus existing tab
                    }
                    OpenResult::Opened => {
                        if let Ok(editor) = EditorView::open(path) {
                            let title = editor.title().to_string();
                            // Downcast desktop to insert tab
                            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                                desktop.insert_tab(
                                    SlotId::Center,
                                    title,
                                    Box::new(editor),
                                );
                            }
                        }
                    }
                }
            }
            CM_NEW_SHELL => {
                let term = TerminalView::new("Shell");
                if let Some(desktop) = downcast_desktop(ctx.desktop) {
                    desktop.insert_tab(SlotId::Right, "Shell", Box::new(term));
                }
            }
            _ => {}
        }
    });

    Ok(())
}

fn downcast_desktop(view: &mut dyn txv_core::view::View) -> Option<&mut SlottedDesktop> {
    let ptr = view as *mut dyn txv_core::view::View;
    // SAFETY: we know the desktop is a SlottedDesktop (we created it above)
    unsafe { (ptr as *mut SlottedDesktop).as_mut() }
}
