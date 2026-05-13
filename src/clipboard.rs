//! System clipboard integration via platform commands (macOS) or OSC 52.

use std::io::Write;

use base64::Engine;

/// Copy text to system clipboard.
/// On macOS, uses pbcopy for reliable single-copy behavior.
/// On other platforms, uses OSC 52 escape sequence.
pub fn copy_to_clipboard(text: &str) {
    #[cfg(target_os = "macos")]
    {
        if let Ok(mut child) = std::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
            return;
        }
    }
    // Fallback: OSC 52
    let encoded = base64::engine::general_purpose::STANDARD.encode(text);
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
