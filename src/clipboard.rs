//! System clipboard integration via platform commands (macOS) or OSC 52.

#[cfg(not(target_os = "macos"))]
use base64::Engine;
use std::io::{self, Write};
use std::process::Command;
#[cfg(target_os = "macos")]
use std::process::Stdio;

#[cfg(not(target_os = "macos"))]
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

/// Copy text to system clipboard.
/// On macOS, uses pbcopy for reliable single-copy behavior.
/// On other platforms, uses OSC 52 escape sequence.
/// Trailing whitespace is stripped from each line before copying.
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let trimmed: String = text.lines().map(|l| l.trim_end()).collect::<Vec<_>>().join("\n");
    let cleaned = if text.ends_with('\n') {
        format!("{trimmed}\n")
    } else {
        trimmed
    };
    copy_raw(&cleaned)
}

fn copy_raw(text: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let mut child = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| format!("pbcopy: {e}"))?;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| format!("pbcopy write: {e}"))?;
        }
        child.wait().map_err(|e| format!("pbcopy: {e}"))?;
        Ok(())
    }
    // Fallback: OSC 52
    #[cfg(not(target_os = "macos"))]
    {
        let encoded = BASE64_STANDARD.encode(text);
        let seq = format!("\x1b]52;c;{encoded}\x07");
        io::stdout()
            .write_all(seq.as_bytes())
            .map_err(|e| format!("clipboard: {e}"))?;
        io::stdout().flush().map_err(|e| format!("clipboard: {e}"))?;
        Ok(())
    }
}

/// Paste from system clipboard using platform-specific command.
/// Returns Err with reason on failure.
pub fn paste_from_clipboard() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    let result = Command::new("pbpaste").output();

    #[cfg(not(target_os = "macos"))]
    let result = Command::new("xclip").args(["-selection", "clipboard", "-o"]).output();

    match result {
        Ok(output) if output.status.success() => Ok(String::from_utf8_lossy(&output.stdout).to_string()),
        Ok(output) => Err(format!(
            "Clipboard command failed (exit {})",
            output.status.code().unwrap_or(-1)
        )),
        Err(e) => Err(format!("Clipboard unavailable: {e}")),
    }
}
