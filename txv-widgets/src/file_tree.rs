//! Filesystem tree data provider using the `ignore` crate.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use txv::cell::{Attrs, Color, Style};
use txv::surface::Surface;

use crate::tree_view::TreeData;

/// Icon for a file based on extension.
fn file_icon(path: &Path) -> &'static str {
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

/// A directory entry with cached children.
struct DirEntry {
    is_dir: bool,
    children: Vec<PathBuf>,
}

/// Filesystem tree data source implementing [`TreeData`].
///
/// Walks directories using the `ignore` crate, respecting `.gitignore`.
/// Directories sort before files; entries are alphabetical within each group.
pub struct FileTreeData {
    root: PathBuf,
    entries: HashMap<PathBuf, DirEntry>,
    roots: Vec<PathBuf>,
}

impl FileTreeData {
    /// Build a tree rooted at `root`, scanning up to `max_depth` levels.
    ///
    /// Returns an error if the root path cannot be read.
    pub fn new(root: &Path, max_depth: usize) -> Result<Self, ignore::Error> {
        let mut entries = HashMap::new();
        let mut child_map: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

        let walker = ignore::WalkBuilder::new(root)
            .max_depth(Some(max_depth))
            .sort_by_file_name(|a, b| a.cmp(b))
            .build();

        for result in walker {
            let entry = result?;
            let path = entry.path().to_path_buf();
            if path == root {
                continue;
            }
            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

            let parent = match path.parent() {
                Some(p) => p.to_path_buf(),
                None => continue,
            };

            child_map.entry(parent).or_default().push(path.clone());

            entries.insert(
                path,
                DirEntry {
                    is_dir,
                    children: Vec::new(),
                },
            );
        }

        // Sort children: dirs first, then alphabetical by filename
        for children in child_map.values_mut() {
            children.sort_by(|a, b| {
                let a_dir = entries.get(a).map(|e| e.is_dir).unwrap_or(false);
                let b_dir = entries.get(b).map(|e| e.is_dir).unwrap_or(false);
                b_dir
                    .cmp(&a_dir)
                    .then_with(|| a.file_name().cmp(&b.file_name()))
            });
        }

        // Assign children to entries, and ensure dirs exist in entries map
        let roots = child_map
            .get(&root.to_path_buf())
            .cloned()
            .unwrap_or_default();

        for (parent, children) in &child_map {
            if let Some(entry) = entries.get_mut(parent) {
                entry.children = children.clone();
            }
        }

        Ok(Self {
            root: root.to_path_buf(),
            entries,
            roots,
        })
    }

    /// The root directory this tree was built from.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Whether a path is a directory in this tree.
    pub fn is_dir(&self, path: &Path) -> bool {
        self.entries.get(path).map(|e| e.is_dir).unwrap_or(false)
    }
}

impl TreeData for FileTreeData {
    type NodeId = PathBuf;

    fn root_nodes(&self) -> Vec<PathBuf> {
        self.roots.clone()
    }

    fn children(&self, id: &PathBuf) -> Vec<PathBuf> {
        self.entries
            .get(id)
            .map(|e| e.children.clone())
            .unwrap_or_default()
    }

    fn has_children(&self, id: &PathBuf) -> bool {
        self.entries.get(id).map(|e| e.is_dir).unwrap_or(false)
    }

    fn render_node(
        &self,
        id: &PathBuf,
        surface: &mut Surface<'_>,
        depth: usize,
        expanded: bool,
        selected: bool,
    ) {
        let is_dir = self.is_dir(id);
        let icon = if is_dir {
            if expanded {
                "📂"
            } else {
                "📁"
            }
        } else {
            file_icon(id)
        };

        let name = id.file_name().and_then(|n| n.to_str()).unwrap_or("?");

        let indent = depth as u16 * 2;
        let text = format!("{icon} {name}");

        let fg = if is_dir {
            Color::Ansi(4) // blue
        } else {
            Color::Reset
        };

        let style = Style {
            fg,
            bg: if selected {
                Color::Ansi(0)
            } else {
                Color::Reset
            },
            attrs: Attrs {
                reverse: selected,
                bold: is_dir,
                ..Attrs::default()
            },
        };

        if selected {
            surface.fill(' ', style);
        }
        surface.print(indent, 0, &text, style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use txv::cell::ColorMode;
    use txv::screen::Screen;

    fn make_temp_tree() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("Cargo.toml"), "[package]").unwrap();
        fs::write(root.join("README.md"), "# hi").unwrap();
        dir
    }

    #[test]
    fn loads_directory() {
        let tmp = make_temp_tree();
        let data = FileTreeData::new(tmp.path(), 10).unwrap();
        let roots = data.root_nodes();
        assert!(!roots.is_empty());
    }

    #[test]
    fn dirs_sort_first() {
        let tmp = make_temp_tree();
        let data = FileTreeData::new(tmp.path(), 10).unwrap();
        let roots = data.root_nodes();
        // "src" dir should come before files
        let first_is_dir = data.is_dir(&roots[0]);
        assert!(first_is_dir, "directories should sort before files");
    }

    #[test]
    fn has_children_for_dirs() {
        let tmp = make_temp_tree();
        let data = FileTreeData::new(tmp.path(), 10).unwrap();
        let src = tmp.path().join("src");
        assert!(data.has_children(&src));
    }

    #[test]
    fn no_children_for_files() {
        let tmp = make_temp_tree();
        let data = FileTreeData::new(tmp.path(), 10).unwrap();
        let toml = tmp.path().join("Cargo.toml");
        assert!(!data.has_children(&toml));
    }

    #[test]
    fn children_of_src() {
        let tmp = make_temp_tree();
        let data = FileTreeData::new(tmp.path(), 10).unwrap();
        let children = data.children(&tmp.path().join("src"));
        assert_eq!(children.len(), 1);
        assert!(children[0].ends_with("main.rs"));
    }

    #[test]
    fn render_node_shows_name() {
        let tmp = make_temp_tree();
        let data = FileTreeData::new(tmp.path(), 10).unwrap();
        let toml = tmp.path().join("Cargo.toml");
        let mut screen = Screen::with_color_mode(40, 1, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            data.render_node(&toml, &mut s, 0, false, false);
        }
        let text = screen.to_text();
        assert!(text.contains("Cargo.toml"));
    }

    #[test]
    fn render_selected_has_reverse() {
        let tmp = make_temp_tree();
        let data = FileTreeData::new(tmp.path(), 10).unwrap();
        let toml = tmp.path().join("Cargo.toml");
        let mut screen = Screen::with_color_mode(40, 1, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            data.render_node(&toml, &mut s, 0, false, true);
        }
        assert!(screen.cell(0, 0).style.attrs.reverse);
    }

    #[test]
    fn root_accessor() {
        let tmp = make_temp_tree();
        let data = FileTreeData::new(tmp.path(), 10).unwrap();
        assert_eq!(data.root(), tmp.path());
    }

    #[test]
    fn file_icon_mapping() {
        assert_eq!(file_icon(Path::new("main.rs")), "🦀");
        assert_eq!(file_icon(Path::new("App.tsx")), "🔷");
        assert_eq!(file_icon(Path::new("Main.java")), "☕");
        assert_eq!(file_icon(Path::new("main.go")), "🐹");
        assert_eq!(file_icon(Path::new("unknown.xyz")), "📄");
    }
}
