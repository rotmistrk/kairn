//! Flat file listing data provider using the `ignore` crate.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use txv::cell::{Attrs, Color, Style};
use txv::surface::Surface;

use crate::list_view::ListData;

/// Icon for a file based on extension.
fn file_icon(path: &Path) -> &'static str {
    if path.is_dir() {
        return "📁";
    }
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
    {
        "rs" => "🦀",
        "java" => "☕",
        "go" => "🐹",
        "ts" | "tsx" => "🔷",
        "js" | "jsx" => "📜",
        "toml" | "yaml" | "yml" | "json" => "⚙",
        "md" => "📝",
        _ => "📄",
    }
}

/// Format a byte size as a human-readable string.
fn human_size(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    const GIB: u64 = 1024 * MIB;
    if bytes >= GIB {
        format!("{:.1}G", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1}M", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1}K", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes}B")
    }
}

/// Format a system time as a short date string.
fn format_time(time: SystemTime) -> String {
    let secs = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86400;
    let y = 1970 + (days * 400 / 146097); // approximate year
    let rem = secs % 86400;
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    // Simple month/day approximation
    let day_of_year = days - (y - 1970) * 365 - ((y - 1969) / 4);
    let mon = (day_of_year / 30).min(11) + 1;
    let day = (day_of_year % 30) + 1;
    format!("{y:04}-{mon:02}-{day:02} {h:02}:{m:02}")
}

/// A single file entry in the list.
struct FileEntry {
    path: PathBuf,
    name: String,
    size: u64,
    modified: Option<SystemTime>,
    is_dir: bool,
}

/// Flat file listing data source implementing [`ListData`].
///
/// Lists files in a directory using the `ignore` crate (respects `.gitignore`).
/// Each item shows icon, name, size, and modification date.
pub struct FileListData {
    entries: Vec<FileEntry>,
    root: PathBuf,
}

impl FileListData {
    /// Build a flat file list for the given directory.
    ///
    /// Only lists immediate children (depth 1). Sorted alphabetically.
    pub fn new(root: &Path) -> Result<Self, ignore::Error> {
        let mut entries = Vec::new();

        let walker = ignore::WalkBuilder::new(root)
            .max_depth(Some(1))
            .sort_by_file_name(|a, b| a.cmp(b))
            .build();

        for result in walker {
            let entry = result?;
            let path = entry.path().to_path_buf();
            if path == root {
                continue;
            }
            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();

            let (size, modified) = entry
                .metadata()
                .ok()
                .map(|m| (m.len(), m.modified().ok()))
                .unwrap_or((0, None));

            entries.push(FileEntry {
                path,
                name,
                size,
                modified,
                is_dir,
            });
        }

        // Sort: dirs first, then alphabetical
        entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));

        Ok(Self {
            entries,
            root: root.to_path_buf(),
        })
    }

    /// The root directory this list was built from.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get the path of an entry by index.
    pub fn path_at(&self, index: usize) -> Option<&Path> {
        self.entries.get(index).map(|e| e.path.as_path())
    }
}

impl ListData for FileListData {
    fn len(&self) -> usize {
        self.entries.len()
    }

    fn render_item(&self, index: usize, surface: &mut Surface<'_>, selected: bool) {
        let Some(entry) = self.entries.get(index) else {
            return;
        };

        let style = Style {
            fg: if entry.is_dir {
                Color::Ansi(4)
            } else {
                Color::Reset
            },
            bg: Color::Reset,
            attrs: Attrs {
                reverse: selected,
                bold: entry.is_dir,
                ..Attrs::default()
            },
        };

        if selected {
            surface.fill(' ', style);
        }

        let icon = if entry.is_dir {
            "📁"
        } else {
            file_icon(&entry.path)
        };
        let size_str = if entry.is_dir {
            "<DIR>".to_string()
        } else {
            human_size(entry.size)
        };
        let date_str = entry.modified.map(format_time).unwrap_or_default();

        let w = surface.width() as usize;
        // Layout: icon name  [right-aligned: size  date]
        let right = format!("{size_str:>8}  {date_str}");
        let name_max = w.saturating_sub(right.len() + 5); // 3 for icon+space, 2 padding
        let name = if entry.name.len() > name_max {
            format!("{}…", &entry.name[..name_max.saturating_sub(1)])
        } else {
            entry.name.clone()
        };

        surface.print(0, 0, &format!("{icon} {name}"), style);
        let right_col = w.saturating_sub(right.len());
        surface.print(right_col as u16, 0, &right, style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use txv::cell::ColorMode;
    use txv::screen::Screen;

    fn make_temp_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("subdir")).unwrap();
        fs::write(root.join("hello.rs"), "fn main() {}").unwrap();
        fs::write(root.join("readme.md"), "# hi").unwrap();
        dir
    }

    #[test]
    fn loads_entries() {
        let tmp = make_temp_dir();
        let data = FileListData::new(tmp.path()).unwrap();
        assert_eq!(data.len(), 3); // subdir, hello.rs, readme.md
    }

    #[test]
    fn dirs_sort_first() {
        let tmp = make_temp_dir();
        let data = FileListData::new(tmp.path()).unwrap();
        assert!(data.entries[0].is_dir);
    }

    #[test]
    fn path_at_returns_correct() {
        let tmp = make_temp_dir();
        let data = FileListData::new(tmp.path()).unwrap();
        let p = data.path_at(0);
        assert!(p.is_some());
    }

    #[test]
    fn path_at_out_of_bounds() {
        let tmp = make_temp_dir();
        let data = FileListData::new(tmp.path()).unwrap();
        assert!(data.path_at(999).is_none());
    }

    #[test]
    fn is_empty_false() {
        let tmp = make_temp_dir();
        let data = FileListData::new(tmp.path()).unwrap();
        assert!(!data.is_empty());
    }

    #[test]
    fn render_item_shows_name() {
        let tmp = make_temp_dir();
        let data = FileListData::new(tmp.path()).unwrap();
        let mut screen = Screen::with_color_mode(60, 1, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            data.render_item(1, &mut s, false); // first file after dir
        }
        let text = screen.to_text();
        assert!(
            text.contains("hello.rs") || text.contains("readme.md"),
            "should show a filename: {text}"
        );
    }

    #[test]
    fn render_selected_has_reverse() {
        let tmp = make_temp_dir();
        let data = FileListData::new(tmp.path()).unwrap();
        let mut screen = Screen::with_color_mode(60, 1, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            data.render_item(0, &mut s, true);
        }
        assert!(screen.cell(0, 0).style.attrs.reverse);
    }

    #[test]
    fn root_accessor() {
        let tmp = make_temp_dir();
        let data = FileListData::new(tmp.path()).unwrap();
        assert_eq!(data.root(), tmp.path());
    }

    #[test]
    fn human_size_formatting() {
        assert_eq!(human_size(0), "0B");
        assert_eq!(human_size(512), "512B");
        assert_eq!(human_size(1024), "1.0K");
        assert_eq!(human_size(1_048_576), "1.0M");
        assert_eq!(human_size(1_073_741_824), "1.0G");
    }
}
