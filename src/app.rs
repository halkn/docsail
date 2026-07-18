#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Focus {
    #[default]
    FileTree,
    Preview,
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
}

#[derive(Debug)]
pub struct App {
    focus: Focus,
    is_running: bool,
    selected_file_index: usize,
    file_count: usize,
    preview_scroll: usize,
    activation_requested: bool,
    reload_requested: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            focus: Focus::default(),
            is_running: false,
            selected_file_index: 0,
            file_count: usize::MAX,
            preview_scroll: 0,
            activation_requested: false,
            reload_requested: false,
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

    pub fn set_file_count(&mut self, file_count: usize) {
        self.file_count = file_count;
        self.selected_file_index = self.selected_file_index.min(file_count.saturating_sub(1));
    }

    pub fn preview_scroll(&self) -> usize {
        self.preview_scroll
    }

    pub fn take_activation_request(&mut self) -> bool {
        std::mem::take(&mut self.activation_requested)
    }

    pub fn take_reload_request(&mut self) -> bool {
        std::mem::take(&mut self.reload_requested)
    }

    pub fn update(&mut self, event: AppEvent) {
        match event {
            AppEvent::MoveDown => self.move_down(),
            AppEvent::MoveUp => self.move_up(),
            AppEvent::Activate => self.activation_requested = true,
            AppEvent::ToggleFocus => self.toggle_focus(),
            AppEvent::Reload => self.reload_requested = true,
            AppEvent::Resize => {}
            AppEvent::Tick => {}
            AppEvent::Quit => self.is_running = false,
        }
    }

    fn move_down(&mut self) {
        match self.focus {
            Focus::FileTree => {
                let previous_selection = self.selected_file_index;
                if self.file_count > 0 {
                    self.selected_file_index = self
                        .selected_file_index
                        .saturating_add(1)
                        .min(self.file_count.saturating_sub(1));
                }
                if self.selected_file_index != previous_selection {
                    self.preview_scroll = 0;
                }
            }
            Focus::Preview => self.preview_scroll = self.preview_scroll.saturating_add(1),
        }
    }

    fn move_up(&mut self) {
        match self.focus {
            Focus::FileTree => {
                let previous_selection = self.selected_file_index;
                self.selected_file_index = self.selected_file_index.saturating_sub(1);
                if self.selected_file_index != previous_selection {
                    self.preview_scroll = 0;
                }
            }
            Focus::Preview => self.preview_scroll = self.preview_scroll.saturating_sub(1),
        }
    }

    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::FileTree => Focus::Preview,
            Focus::Preview => Focus::FileTree,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{App, AppEvent, Focus};

    #[test]
    fn moves_the_file_selection_when_the_file_tree_is_focused() {
        let mut app = App::new();
        app.set_file_count(3);

        app.update(AppEvent::MoveDown);
        app.update(AppEvent::MoveDown);
        app.update(AppEvent::MoveUp);

        assert_eq!(app.selected_file_index(), 1);
        assert_eq!(app.preview_scroll(), 0);
    }

    #[test]
    fn keeps_file_selection_within_the_available_files() {
        let mut app = App::new();
        app.set_file_count(2);

        app.update(AppEvent::MoveDown);
        app.update(AppEvent::MoveDown);

        assert_eq!(app.selected_file_index(), 1);
    }

    #[test]
    fn keeps_the_file_selection_at_zero_when_the_tree_is_empty() {
        let mut app = App::new();
        app.set_file_count(0);

        app.update(AppEvent::MoveDown);

        assert_eq!(app.selected_file_index(), 0);
    }

    #[test]
    fn resets_preview_scroll_after_selecting_another_file() {
        let mut app = App::new();
        app.set_file_count(2);
        app.update(AppEvent::ToggleFocus);
        app.update(AppEvent::MoveDown);
        app.update(AppEvent::ToggleFocus);

        app.update(AppEvent::MoveDown);

        assert_eq!(app.selected_file_index(), 1);
        assert_eq!(app.preview_scroll(), 0);
    }

    #[test]
    fn scrolls_the_preview_when_the_preview_is_focused() {
        let mut app = App::new();
        app.update(AppEvent::ToggleFocus);
        app.update(AppEvent::MoveDown);
        app.update(AppEvent::MoveDown);
        app.update(AppEvent::MoveUp);

        assert_eq!(app.focus(), Focus::Preview);
        assert_eq!(app.selected_file_index(), 0);
        assert_eq!(app.preview_scroll(), 1);
    }

    #[test]
    fn records_activation_and_reload_requests_once() {
        let mut app = App::new();

        app.update(AppEvent::Activate);
        app.update(AppEvent::Reload);

        assert!(app.take_activation_request());
        assert!(!app.take_activation_request());
        assert!(app.take_reload_request());
        assert!(!app.take_reload_request());
    }

    #[test]
    fn quits_the_event_loop() {
        let mut app = App::new();

        app.update(AppEvent::Quit);

        assert!(!app.is_running());
    }
}
