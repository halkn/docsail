#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Focus {
    #[default]
    FileTree,
    Preview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overlay {
    Toc,
    FileSearch,
    PageSearch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppEvent {
    MoveDown,
    MoveUp,
    Activate,
    ToggleFocus,
    Reload,
    Resize,
    Tick,
    Quit,
    Back,
    Forward,
    OpenToc,
    OpenFileSearch,
    OpenPageSearch,
    NextResult,
    PreviousResult,
    Input(char),
    Backspace,
    CursorStart,
    CursorEnd,
    CursorLeft,
    CursorRight,
    DeleteForward,
    KillToEnd,
    KillToStart,
    ToggleTreeNode,
    Escape,
}

#[derive(Debug)]
pub struct App {
    focus: Focus,
    is_running: bool,
    selected_file_index: usize,
    file_count: usize,
    tree_cursor: usize,
    tree_rows: Vec<Option<usize>>,
    preview_scroll: usize,
    activation_requested: bool,
    reload_requested: bool,
    back_requested: bool,
    forward_requested: bool,
    overlay: Option<Overlay>,
    query: String,
    query_cursor: usize,
    selected_result: usize,
    result_count: usize,
    overlay_submission: Option<Overlay>,
    next_page_match_requested: bool,
    previous_page_match_requested: bool,
    tree_toggle_requested: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            focus: Focus::default(),
            is_running: false,
            selected_file_index: 0,
            file_count: usize::MAX,
            tree_cursor: 0,
            tree_rows: Vec::new(),
            preview_scroll: 0,
            activation_requested: false,
            reload_requested: false,
            back_requested: false,
            forward_requested: false,
            overlay: None,
            query: String::new(),
            query_cursor: 0,
            selected_result: 0,
            result_count: 0,
            overlay_submission: None,
            next_page_match_requested: false,
            previous_page_match_requested: false,
            tree_toggle_requested: false,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            is_running: true,
            ..Self::default()
        }
    }
    pub fn focus(&self) -> Focus {
        self.focus
    }
    pub fn is_running(&self) -> bool {
        self.is_running
    }
    pub fn selected_file_index(&self) -> usize {
        self.selected_file_index
    }
    pub fn tree_cursor(&self) -> usize {
        self.tree_cursor
    }
    pub fn set_tree_cursor(&mut self, cursor: usize) {
        self.tree_cursor = cursor.min(self.tree_rows.len().saturating_sub(1));
    }
    pub fn preview_scroll(&self) -> usize {
        self.preview_scroll
    }
    pub fn overlay(&self) -> Option<Overlay> {
        self.overlay
    }
    pub fn query(&self) -> &str {
        &self.query
    }
    pub fn query_cursor(&self) -> usize {
        self.query_cursor
    }
    pub fn selected_result(&self) -> usize {
        self.selected_result
    }
    pub fn set_file_count(&mut self, file_count: usize) {
        self.file_count = file_count;
        self.selected_file_index = self.selected_file_index.min(file_count.saturating_sub(1));
    }
    pub fn set_tree_rows(&mut self, rows: Vec<Option<usize>>) {
        self.tree_rows = rows;
        self.tree_cursor = self
            .tree_rows
            .iter()
            .position(|file_index| *file_index == Some(self.selected_file_index))
            .unwrap_or_else(|| self.tree_cursor.min(self.tree_rows.len().saturating_sub(1)));
    }
    pub fn set_result_count(&mut self, count: usize) {
        self.result_count = count;
        self.selected_result = self.selected_result.min(count.saturating_sub(1));
    }
    pub fn navigate_to(&mut self, file_index: usize, scroll: usize) {
        self.selected_file_index = file_index.min(self.file_count.saturating_sub(1));
        self.tree_cursor = self
            .tree_rows
            .iter()
            .position(|tree_file_index| *tree_file_index == Some(self.selected_file_index))
            .unwrap_or(self.tree_cursor);
        self.preview_scroll = scroll;
        self.focus = Focus::Preview;
    }
    pub fn take_activation_request(&mut self) -> bool {
        std::mem::take(&mut self.activation_requested)
    }
    pub fn take_reload_request(&mut self) -> bool {
        std::mem::take(&mut self.reload_requested)
    }
    pub fn take_back_request(&mut self) -> bool {
        std::mem::take(&mut self.back_requested)
    }
    pub fn take_forward_request(&mut self) -> bool {
        std::mem::take(&mut self.forward_requested)
    }
    pub fn take_overlay_submission(&mut self) -> Option<Overlay> {
        self.overlay_submission.take()
    }
    pub fn take_next_page_match_request(&mut self) -> bool {
        std::mem::take(&mut self.next_page_match_requested)
    }
    pub fn take_previous_page_match_request(&mut self) -> bool {
        std::mem::take(&mut self.previous_page_match_requested)
    }
    pub fn take_tree_toggle_request(&mut self) -> bool {
        std::mem::take(&mut self.tree_toggle_requested)
    }

    pub fn update(&mut self, event: AppEvent) {
        if self.overlay.is_some() {
            self.update_overlay(event);
            return;
        }
        match event {
            AppEvent::MoveDown => self.move_down(),
            AppEvent::MoveUp => self.move_up(),
            AppEvent::Activate => self.activate_tree_item(),
            AppEvent::ToggleFocus => self.toggle_focus(),
            AppEvent::Reload => self.reload_requested = true,
            AppEvent::Quit => self.is_running = false,
            AppEvent::Back => self.back_requested = true,
            AppEvent::Forward => self.forward_requested = true,
            AppEvent::OpenToc => self.open_overlay(Overlay::Toc),
            AppEvent::OpenFileSearch => self.open_overlay(Overlay::FileSearch),
            AppEvent::OpenPageSearch => self.open_overlay(Overlay::PageSearch),
            AppEvent::Input('j') => self.move_down(),
            AppEvent::Input('k') => self.move_up(),
            AppEvent::Input('q') => self.is_running = false,
            AppEvent::Input('[') => self.back_requested = true,
            AppEvent::Input(']') => self.forward_requested = true,
            AppEvent::Input('t') => self.open_overlay(Overlay::Toc),
            AppEvent::Input('f') => self.open_overlay(Overlay::FileSearch),
            AppEvent::Input('/') => self.open_overlay(Overlay::PageSearch),
            AppEvent::Input('n') => self.next_page_match_requested = true,
            AppEvent::Input('N') => self.previous_page_match_requested = true,
            AppEvent::Input(' ') | AppEvent::ToggleTreeNode if self.focus == Focus::FileTree => {
                self.tree_toggle_requested = true;
            }
            _ => {}
        }
    }

    fn update_overlay(&mut self, event: AppEvent) {
        match event {
            AppEvent::Escape => self.close_overlay(),
            AppEvent::Activate => {
                self.overlay_submission = self.overlay;
                self.close_overlay();
            }
            AppEvent::MoveDown | AppEvent::NextResult => {
                self.selected_result = self
                    .selected_result
                    .saturating_add(1)
                    .min(self.result_count.saturating_sub(1))
            }
            AppEvent::MoveUp | AppEvent::PreviousResult => {
                self.selected_result = self.selected_result.saturating_sub(1)
            }
            AppEvent::Input(character) => self.insert_query_character(character),
            AppEvent::Backspace => self.delete_query_character_before_cursor(),
            AppEvent::DeleteForward => self.delete_query_character_at_cursor(),
            AppEvent::CursorStart => self.query_cursor = 0,
            AppEvent::CursorEnd => self.query_cursor = self.query.len(),
            AppEvent::CursorLeft => {
                self.query_cursor = previous_char_boundary(&self.query, self.query_cursor)
            }
            AppEvent::CursorRight => {
                self.query_cursor = next_char_boundary(&self.query, self.query_cursor)
            }
            AppEvent::KillToEnd => self.query.truncate(self.query_cursor),
            AppEvent::KillToStart => {
                self.query.drain(..self.query_cursor);
                self.query_cursor = 0;
            }
            _ => {}
        }
    }

    fn open_overlay(&mut self, overlay: Overlay) {
        self.overlay = Some(overlay);
        self.query.clear();
        self.query_cursor = 0;
        self.selected_result = 0;
    }
    fn close_overlay(&mut self) {
        self.overlay = None;
        self.result_count = 0;
    }
    fn insert_query_character(&mut self, character: char) {
        self.query.insert(self.query_cursor, character);
        self.query_cursor += character.len_utf8();
        self.selected_result = 0;
    }
    fn delete_query_character_before_cursor(&mut self) {
        let previous = previous_char_boundary(&self.query, self.query_cursor);
        if previous != self.query_cursor {
            self.query.drain(previous..self.query_cursor);
            self.query_cursor = previous;
            self.selected_result = 0;
        }
    }
    fn delete_query_character_at_cursor(&mut self) {
        let next = next_char_boundary(&self.query, self.query_cursor);
        if next != self.query_cursor {
            self.query.drain(self.query_cursor..next);
            self.selected_result = 0;
        }
    }
    fn move_down(&mut self) {
        match self.focus {
            Focus::FileTree => self.move_tree_cursor(1),
            Focus::Preview => self.preview_scroll = self.preview_scroll.saturating_add(1),
        }
    }
    fn move_up(&mut self) {
        match self.focus {
            Focus::FileTree => self.move_tree_cursor(-1),
            Focus::Preview => self.preview_scroll = self.preview_scroll.saturating_sub(1),
        }
    }
    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::FileTree => Focus::Preview,
            Focus::Preview => Focus::FileTree,
        };
    }
    fn activate_tree_item(&mut self) {
        if self.focus == Focus::FileTree && self.tree_rows.get(self.tree_cursor) == Some(&None) {
            self.tree_toggle_requested = true;
        } else {
            self.activation_requested = true;
        }
    }
    fn move_tree_cursor(&mut self, offset: isize) {
        let previous_file_index = self.selected_file_index;
        if self.tree_rows.is_empty() {
            self.selected_file_index = if offset.is_negative() {
                self.selected_file_index
                    .saturating_sub(offset.unsigned_abs())
            } else {
                self.selected_file_index
                    .saturating_add(offset as usize)
                    .min(self.file_count.saturating_sub(1))
            };
            if self.selected_file_index != previous_file_index {
                self.preview_scroll = 0;
            }
            return;
        }
        let max = self.tree_rows.len().saturating_sub(1);
        self.tree_cursor = if offset.is_negative() {
            self.tree_cursor.saturating_sub(offset.unsigned_abs())
        } else {
            self.tree_cursor.saturating_add(offset as usize).min(max)
        };
        if let Some(file_index) = self
            .tree_rows
            .get(self.tree_cursor)
            .and_then(|index| *index)
        {
            self.selected_file_index = file_index;
        }
        if self.selected_file_index != previous_file_index {
            self.preview_scroll = 0;
        }
    }
}

fn previous_char_boundary(value: &str, cursor: usize) -> usize {
    value[..cursor]
        .char_indices()
        .last()
        .map_or(0, |(index, _)| index)
}

fn next_char_boundary(value: &str, cursor: usize) -> usize {
    value[cursor..]
        .chars()
        .next()
        .map_or(cursor, |character| cursor + character.len_utf8())
}

#[cfg(test)]
mod tests {
    use super::{App, AppEvent, Focus, Overlay};
    #[test]
    fn navigates_and_restores_a_location() {
        let mut app = App::new();
        app.set_file_count(3);
        app.navigate_to(2, 8);
        assert_eq!(app.selected_file_index(), 2);
        assert_eq!(app.preview_scroll(), 8);
        assert_eq!(app.focus(), Focus::Preview);
    }
    #[test]
    fn navigates_tree_rows_without_changing_the_preview_on_a_directory() {
        let mut app = App::new();
        app.set_file_count(2);
        app.set_tree_rows(vec![Some(0), None, Some(1)]);

        app.update(AppEvent::MoveDown);

        assert_eq!(app.tree_cursor(), 1);
        assert_eq!(app.selected_file_index(), 0);
        app.update(AppEvent::Activate);
        assert!(app.take_tree_toggle_request());
    }
    #[test]
    fn accepts_a_search_query_and_submits_its_selected_result() {
        let mut app = App::new();
        app.update(AppEvent::OpenFileSearch);
        app.set_result_count(2);
        app.update(AppEvent::Input('g'));
        app.update(AppEvent::MoveDown);
        app.update(AppEvent::Activate);
        assert_eq!(app.take_overlay_submission(), Some(Overlay::FileSearch));
        assert_eq!(app.overlay(), None);
    }
    #[test]
    fn records_navigation_requests() {
        let mut app = App::new();
        app.update(AppEvent::Back);
        app.update(AppEvent::Forward);
        assert!(app.take_back_request());
        assert!(app.take_forward_request());
    }

    #[test]
    fn edits_search_input_at_unicode_character_boundaries() {
        let mut app = App::new();
        app.update(AppEvent::OpenFileSearch);
        app.update(AppEvent::Input('a'));
        app.update(AppEvent::Input('日'));
        app.update(AppEvent::Input('b'));
        app.update(AppEvent::CursorLeft);
        app.update(AppEvent::Backspace);
        app.update(AppEvent::Input('本'));
        app.update(AppEvent::CursorStart);
        app.update(AppEvent::DeleteForward);

        assert_eq!(app.query(), "本b");
        assert_eq!(app.query_cursor(), 0);
    }

    #[test]
    fn accepts_j_and_k_as_search_input() {
        let mut app = App::new();
        app.update(AppEvent::OpenFileSearch);
        app.update(AppEvent::Input('j'));
        app.update(AppEvent::Input('k'));

        assert_eq!(app.query(), "jk");
    }
}
