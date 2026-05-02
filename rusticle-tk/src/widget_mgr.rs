//! Widget instance registry — tracks widgets by string ID.

use std::collections::HashMap;
use std::path::Path;

use crossterm::event::KeyEvent;
use txv::surface::Surface;
use txv_widgets::widget::{EventResult, Widget};
use txv_widgets::{
    FileTreeData, FuzzySelect, InputLine, ListView, Menu, ProgressBar, StatusBar, TabBar, Table,
    TextArea, TreeView,
};

/// Simple string-backed list data for `ListView`.
pub struct StringListData {
    /// The string items.
    pub items: Vec<String>,
}

impl StringListData {
    /// Create from a vec of strings.
    pub fn new(items: Vec<String>) -> Self {
        Self { items }
    }
}

impl txv_widgets::ListData for StringListData {
    fn len(&self) -> usize {
        self.items.len()
    }

    fn render_item(&self, index: usize, surface: &mut Surface<'_>, selected: bool) {
        let style = if selected {
            txv::cell::Style {
                attrs: txv::cell::Attrs {
                    reverse: true,
                    ..txv::cell::Attrs::default()
                },
                ..txv::cell::Style::default()
            }
        } else {
            txv::cell::Style::default()
        };
        if selected {
            surface.fill(' ', style);
        }
        if let Some(text) = self.items.get(index) {
            surface.print(0, 0, text, style);
        }
    }
}

/// A widget instance stored in the registry.
pub enum WidgetEntry {
    /// Multi-line text viewer.
    Text(TextArea),
    /// String list.
    List(ListView<StringListData>),
    /// File tree.
    Tree(TreeView<FileTreeData>),
    /// Single-line input.
    Input(InputLine),
    /// Tab strip.
    TabBar(TabBar),
    /// Status bar.
    StatusBar(StatusBar),
    /// Table with columns.
    Table(Table),
    /// Progress bar.
    Progress(ProgressBar),
    /// Popup menu.
    Menu(Menu),
    /// Fuzzy select.
    FuzzySelect(FuzzySelect),
}

impl WidgetEntry {
    /// Render this widget entry to a surface.
    pub fn render(&self, surface: &mut Surface<'_>, focused: bool) {
        match self {
            Self::Text(w) => w.render(surface, focused),
            Self::List(w) => w.render(surface, focused),
            Self::Tree(w) => w.render(surface, focused),
            Self::Input(w) => w.render(surface, focused),
            Self::TabBar(w) => w.render(surface, focused),
            Self::StatusBar(w) => w.render(surface, focused),
            Self::Table(w) => w.render(surface, focused),
            Self::Progress(w) => w.render(&mut surface.sub(0, 0, surface.width(), 1)),
            Self::Menu(w) => w.render(surface, focused),
            Self::FuzzySelect(w) => w.render(surface, focused),
        }
    }

    /// Handle a key event.
    pub fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        match self {
            Self::Text(w) => w.handle_key(key),
            Self::List(w) => w.handle_key(key),
            Self::Tree(w) => w.handle_key(key),
            Self::Input(w) => w.handle_key(key),
            Self::TabBar(w) => w.handle_key(key),
            Self::StatusBar(w) => w.handle_key(key),
            Self::Table(w) => w.handle_key(key),
            Self::Menu(w) => w.handle_key(key),
            Self::FuzzySelect(w) => w.handle_key(key),
            Self::Progress(_) => EventResult::Ignored,
        }
    }
}

/// Widget kind for creation.
pub enum WidgetKind {
    /// Text area.
    Text,
    /// String list.
    List,
    /// File tree rooted at a path.
    Tree(String),
    /// Input line with prompt.
    Input(String),
    /// Tab bar.
    TabBar,
    /// Status bar.
    StatusBar,
    /// Table with column definitions.
    Table(Vec<txv_widgets::table::Column>),
    /// Progress bar.
    Progress,
    /// Menu with items.
    Menu(Vec<txv_widgets::MenuItem>),
    /// Fuzzy select with items.
    #[allow(dead_code)]
    FuzzySelect(Vec<String>),
}

/// Registry of widget instances keyed by string IDs.
pub struct WidgetManager {
    widgets: HashMap<String, WidgetEntry>,
    next_id: u64,
}

impl WidgetManager {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a widget of the given kind. Returns its ID.
    pub fn create(&mut self, kind: WidgetKind) -> Result<String, String> {
        let id = format!("widget_{}", self.next_id);
        self.next_id += 1;
        let entry = match kind {
            WidgetKind::Text => WidgetEntry::Text(TextArea::new()),
            WidgetKind::List => WidgetEntry::List(ListView::new(StringListData::new(Vec::new()))),
            WidgetKind::Tree(path) => {
                let data = FileTreeData::new(Path::new(&path), 10)
                    .map_err(|e| format!("cannot scan {path}: {e}"))?;
                WidgetEntry::Tree(TreeView::new(data))
            }
            WidgetKind::Input(prompt) => WidgetEntry::Input(InputLine::new(&prompt)),
            WidgetKind::TabBar => WidgetEntry::TabBar(TabBar::new()),
            WidgetKind::StatusBar => WidgetEntry::StatusBar(StatusBar::new()),
            WidgetKind::Table(cols) => WidgetEntry::Table(Table::new(cols)),
            WidgetKind::Progress => WidgetEntry::Progress(ProgressBar::new()),
            WidgetKind::Menu(items) => WidgetEntry::Menu(Menu::new(items)),
            WidgetKind::FuzzySelect(items) => WidgetEntry::FuzzySelect(FuzzySelect::new(items)),
        };
        self.widgets.insert(id.clone(), entry);
        Ok(id)
    }

    /// Get a widget by ID.
    pub fn get(&self, id: &str) -> Option<&WidgetEntry> {
        self.widgets.get(id)
    }

    /// Get a mutable widget by ID.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut WidgetEntry> {
        self.widgets.get_mut(id)
    }

    /// Remove a widget by ID.
    #[allow(dead_code)]
    pub fn remove(&mut self, id: &str) {
        self.widgets.remove(id);
    }

    /// Iterate over all widget IDs.
    pub fn ids(&self) -> impl Iterator<Item = &String> {
        self.widgets.keys()
    }
}

impl Default for WidgetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_text_returns_id() {
        let mut mgr = WidgetManager::new();
        let id = mgr.create(WidgetKind::Text).unwrap_or_default();
        assert_eq!(id, "widget_1");
        assert!(mgr.get(&id).is_some());
    }

    #[test]
    fn ids_increment() {
        let mut mgr = WidgetManager::new();
        let id1 = mgr.create(WidgetKind::Text).unwrap_or_default();
        let id2 = mgr.create(WidgetKind::StatusBar).unwrap_or_default();
        assert_eq!(id1, "widget_1");
        assert_eq!(id2, "widget_2");
    }

    #[test]
    fn remove_widget() {
        let mut mgr = WidgetManager::new();
        let id = mgr.create(WidgetKind::Text).unwrap_or_default();
        mgr.remove(&id);
        assert!(mgr.get(&id).is_none());
    }

    #[test]
    fn create_list() {
        let mut mgr = WidgetManager::new();
        let id = mgr.create(WidgetKind::List).unwrap_or_default();
        assert!(matches!(mgr.get(&id), Some(WidgetEntry::List(_))));
    }

    #[test]
    fn create_input() {
        let mut mgr = WidgetManager::new();
        let id = mgr
            .create(WidgetKind::Input(">> ".into()))
            .unwrap_or_default();
        assert!(matches!(mgr.get(&id), Some(WidgetEntry::Input(_))));
    }

    #[test]
    fn create_tabbar() {
        let mut mgr = WidgetManager::new();
        let id = mgr.create(WidgetKind::TabBar).unwrap_or_default();
        assert!(matches!(mgr.get(&id), Some(WidgetEntry::TabBar(_))));
    }

    #[test]
    fn create_statusbar() {
        let mut mgr = WidgetManager::new();
        let id = mgr.create(WidgetKind::StatusBar).unwrap_or_default();
        assert!(matches!(mgr.get(&id), Some(WidgetEntry::StatusBar(_))));
    }

    #[test]
    fn create_progress() {
        let mut mgr = WidgetManager::new();
        let id = mgr.create(WidgetKind::Progress).unwrap_or_default();
        assert!(matches!(mgr.get(&id), Some(WidgetEntry::Progress(_))));
    }

    #[test]
    fn get_mut_works() {
        let mut mgr = WidgetManager::new();
        let id = mgr.create(WidgetKind::Text).unwrap_or_default();
        if let Some(WidgetEntry::Text(ta)) = mgr.get_mut(&id) {
            ta.set_text("hello");
            assert_eq!(ta.line_count(), 1);
        }
    }
}
