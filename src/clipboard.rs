//! System clipboard integration via OSC 52 (copy) and platform commands (paste).

use std::io::Write;

use base64::Engine;

/// Copy text to system clipboard via OSC 52 escape sequence.
/// Works through SSH, tmux, and iTerm2/kitty/etc.
pub fn copy_to_clipboard(text: &str) {
    let encoded = base64::engine::general_purpose::STANDARD.encode(text);
    // OSC 52: \x1b]52;c;{base64}\x07
    let seq = format!("\x1b]52;c;{encoded}\x07");
    let _ = std::io::stdout().write_all(seq.as_bytes());
    let _ = std::io::stdout().flush();
}

/// Paste from system clipboard using platform-specific command.
/// Returns Err with reason on failure.
pub fn paste_from_clipboard() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("pbpaste").output();

    #[cfg(not(target_os = "macos"))]
    let result = std::process::Command::new("xclip")
        .args(["-selection", "clipboard", "-o"])
        .output();

    match result {
        Ok(output) if output.status.success() => Ok(String::from_utf8_lossy(&output.stdout).to_string()),
        Ok(output) => Err(format!(
            "Clipboard command failed (exit {})",
            output.status.code().unwrap_or(-1)
        )),
        Err(e) => Err(format!("Clipboard unavailable: {e}")),
    }
}
