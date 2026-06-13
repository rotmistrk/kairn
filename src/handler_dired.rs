//! Dired command handlers — file/directory create, delete, rename, copy.

use std::fs;
use std::path::Path;

use txv_core::message::Message;
use txv_core::program::CommandContext;
use txv_widgets::CM_STATUS_MESSAGE;

use crate::app_state::AppState;
use crate::commands::CM_FS_CHANGED;

fn msg(ctx: &mut CommandContext, text: String) {
    ctx.sink()
        .push_command(CM_STATUS_MESSAGE, Some(Box::new(Message::info("file", text))));
}

fn err(ctx: &mut CommandContext, text: String) {
    ctx.sink()
        .push_command(CM_STATUS_MESSAGE, Some(Box::new(Message::error("file", text))));
}

fn refresh_tree(ctx: &mut CommandContext) {
    ctx.sink().push_broadcast(CM_FS_CHANGED, None);
}

pub(crate) fn cmd_new_file(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    let path = Path::new(arg.trim());
    if path.as_os_str().is_empty() {
        err(ctx, "Usage: new-file <path>".into());
        return;
    }
    if path.exists() {
        err(ctx, format!("Already exists: {}", path.display()));
        return;
    }
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                err(ctx, format!("Cannot create dirs: {e}"));
                return;
            }
        }
    }
    if let Err(e) = fs::write(path, "") {
        err(ctx, format!("Cannot create file: {e}"));
        return;
    }
    msg(ctx, format!("Created: {}", path.display()));
    refresh_tree(ctx);
}

pub(crate) fn cmd_new_dir(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    let path = Path::new(arg.trim());
    if path.as_os_str().is_empty() {
        err(ctx, "Usage: new-dir <path>".into());
        return;
    }
    if path.exists() {
        err(ctx, format!("Already exists: {}", path.display()));
        return;
    }
    if let Err(e) = fs::create_dir_all(path) {
        err(ctx, format!("Cannot create directory: {e}"));
        return;
    }
    msg(ctx, format!("Created dir: {}", path.display()));
    refresh_tree(ctx);
}

pub(crate) fn cmd_delete_file(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    let path = Path::new(arg.trim());
    if path.as_os_str().is_empty() {
        err(ctx, "Usage: delete-file <path>".into());
        return;
    }
    if !path.exists() {
        err(ctx, format!("Not found: {}", path.display()));
        return;
    }
    let result = if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };
    if let Err(e) = result {
        err(ctx, format!("Cannot delete: {e}"));
        return;
    }
    msg(ctx, format!("Deleted: {}", path.display()));
    refresh_tree(ctx);
}

pub(crate) fn cmd_rename_file(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    // arg format: "<old_path> <new_path>" (split on last space? no — use " -> " separator)
    // Actually: prefill is "rename-file <old>", user appends " <new>"
    let parts: Vec<&str> = arg.trim().splitn(2, ' ').collect();
    if parts.len() < 2 {
        err(ctx, "Usage: rename-file <old> <new>".into());
        return;
    }
    let old = Path::new(parts[0]);
    let new = Path::new(parts[1]);
    if !old.exists() {
        err(ctx, format!("Not found: {}", old.display()));
        return;
    }
    if new.exists() {
        err(ctx, format!("Already exists: {}", new.display()));
        return;
    }
    if let Some(parent) = new.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                err(ctx, format!("Cannot create dirs: {e}"));
                return;
            }
        }
    }
    if let Err(e) = fs::rename(old, new) {
        err(ctx, format!("Cannot rename: {e}"));
        return;
    }
    msg(ctx, format!("Renamed: {} → {}", old.display(), new.display()));
    refresh_tree(ctx);
}

pub(crate) fn cmd_copy_file(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    let parts: Vec<&str> = arg.trim().splitn(2, ' ').collect();
    if parts.len() < 2 {
        err(ctx, "Usage: copy-file <src> <dest>".into());
        return;
    }
    let src = Path::new(parts[0]);
    let dest = Path::new(parts[1]);
    if !src.exists() {
        err(ctx, format!("Not found: {}", src.display()));
        return;
    }
    if dest.exists() {
        err(ctx, format!("Already exists: {}", dest.display()));
        return;
    }
    if let Some(parent) = dest.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                err(ctx, format!("Cannot create dirs: {e}"));
                return;
            }
        }
    }
    let result = if src.is_dir() {
        copy_dir_recursive(src, dest)
    } else {
        fs::copy(src, dest).map(|_| ())
    };
    if let Err(e) = result {
        err(ctx, format!("Cannot copy: {e}"));
        return;
    }
    msg(ctx, format!("Copied: {} → {}", src.display(), dest.display()));
    refresh_tree(ctx);
}

pub(crate) fn copy_dir_recursive(src: &Path, dest: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        let ft = entry.file_type()?;
        if ft.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
