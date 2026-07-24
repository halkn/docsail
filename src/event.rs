use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::{App, AppEvent};

pub trait EventSource {
    fn next_event(&mut self) -> io::Result<AppEvent>;
}

pub struct CrosstermEventSource;

impl EventSource for CrosstermEventSource {
    fn next_event(&mut self) -> io::Result<AppEvent> {
        loop {
            if !event::poll(Duration::from_millis(250))? {
                return Ok(AppEvent::Tick);
            }
            if let Some(event) = translate_event(event::read()?) {
                return Ok(event);
            }
        }
    }
}

pub fn run<S, D>(app: &mut App, event_source: &mut S, mut draw: D) -> io::Result<()>
where
    S: EventSource,
    D: FnMut(&mut App) -> io::Result<()>,
{
    draw(app)?;

    while app.is_running() {
        app.update(event_source.next_event()?);

        if app.is_running() {
            draw(app)?;
        }
    }

    Ok(())
}

fn translate_event(event: Event) -> Option<AppEvent> {
    match event {
        Event::Key(key_event)
            if matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
        {
            translate_key(key_event)
        }
        Event::Resize(_, _) => Some(AppEvent::Resize),
        _ => None,
    }
}

fn translate_key(key_event: KeyEvent) -> Option<AppEvent> {
    if key_event.modifiers == KeyModifiers::CONTROL {
        return match key_event.code {
            KeyCode::Char('a') => Some(AppEvent::CursorStart),
            KeyCode::Char('e') => Some(AppEvent::CursorEnd),
            KeyCode::Char('b') => Some(AppEvent::CursorLeft),
            KeyCode::Char('f') => Some(AppEvent::CursorRight),
            KeyCode::Char('d') => Some(AppEvent::DeleteForward),
            KeyCode::Char('h') => Some(AppEvent::Backspace),
            KeyCode::Char('k') => Some(AppEvent::KillToEnd),
            KeyCode::Char('n') => Some(AppEvent::NextResult),
            KeyCode::Char('p') => Some(AppEvent::PreviousResult),
            KeyCode::Char('u') => Some(AppEvent::KillToStart),
            _ => None,
        };
    }
    match key_event.code {
        KeyCode::Down => Some(AppEvent::MoveDown),
        KeyCode::Up => Some(AppEvent::MoveUp),
        KeyCode::Enter => Some(AppEvent::Activate),
        KeyCode::Tab => Some(AppEvent::ToggleFocus),
        KeyCode::Char('r') => Some(AppEvent::Reload),
        KeyCode::Esc => Some(AppEvent::Escape),
        KeyCode::Backspace => Some(AppEvent::Backspace),
        KeyCode::Char(character) => Some(AppEvent::Input(character)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{EventSource, run, translate_event};
    use crate::app::{App, AppEvent};
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use std::{collections::VecDeque, io};

    struct FakeEventSource {
        events: VecDeque<AppEvent>,
    }

    impl EventSource for FakeEventSource {
        fn next_event(&mut self) -> io::Result<AppEvent> {
            self.events
                .pop_front()
                .ok_or_else(|| io::Error::other("no more events"))
        }
    }

    #[test]
    fn translates_initial_keybindings() {
        let cases = [
            (KeyCode::Char('j'), AppEvent::Input('j')),
            (KeyCode::Down, AppEvent::MoveDown),
            (KeyCode::Char('k'), AppEvent::Input('k')),
            (KeyCode::Up, AppEvent::MoveUp),
            (KeyCode::Enter, AppEvent::Activate),
            (KeyCode::Tab, AppEvent::ToggleFocus),
            (KeyCode::Char('r'), AppEvent::Reload),
            (KeyCode::Char('q'), AppEvent::Input('q')),
            (KeyCode::Char('['), AppEvent::Input('[')),
            (KeyCode::Char(']'), AppEvent::Input(']')),
            (KeyCode::Char('t'), AppEvent::Input('t')),
            (KeyCode::Char('f'), AppEvent::Input('f')),
            (KeyCode::Char('/'), AppEvent::Input('/')),
            (KeyCode::Char('m'), AppEvent::Input('m')),
        ];

        for (key_code, expected_event) in cases {
            let key_event = KeyEvent::new(key_code, KeyModifiers::NONE);

            assert_eq!(translate_event(Event::Key(key_event)), Some(expected_event));
        }
    }

    #[test]
    fn translates_emacs_style_search_editing_keys() {
        let cases = [
            ('a', AppEvent::CursorStart),
            ('e', AppEvent::CursorEnd),
            ('b', AppEvent::CursorLeft),
            ('f', AppEvent::CursorRight),
            ('d', AppEvent::DeleteForward),
            ('h', AppEvent::Backspace),
            ('k', AppEvent::KillToEnd),
            ('n', AppEvent::NextResult),
            ('p', AppEvent::PreviousResult),
            ('u', AppEvent::KillToStart),
        ];

        for (character, expected) in cases {
            let key_event = KeyEvent::new(KeyCode::Char(character), KeyModifiers::CONTROL);

            assert_eq!(translate_event(Event::Key(key_event)), Some(expected));
        }
    }

    #[test]
    fn ignores_key_releases() {
        let key_event = KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: KeyEventState::NONE,
        };

        assert_eq!(translate_event(Event::Key(key_event)), None);
    }

    #[test]
    fn translates_terminal_resize_events() {
        assert_eq!(
            translate_event(Event::Resize(120, 40)),
            Some(AppEvent::Resize)
        );
    }

    #[test]
    fn draws_until_the_quit_event() {
        let mut app = App::new();
        let mut event_source = FakeEventSource {
            events: VecDeque::from([AppEvent::Resize, AppEvent::MoveDown, AppEvent::Quit]),
        };
        let mut draw_count = 0;

        run(&mut app, &mut event_source, |_| {
            draw_count += 1;
            Ok(())
        })
        .unwrap();

        assert_eq!(draw_count, 3);
        assert_eq!(app.selected_file_index(), 1);
    }

    #[test]
    fn redraws_after_a_tick() {
        let mut app = App::new();
        let mut event_source = FakeEventSource {
            events: VecDeque::from([AppEvent::Tick, AppEvent::Quit]),
        };
        let mut draw_count = 0;
        run(&mut app, &mut event_source, |_| {
            draw_count += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(draw_count, 2);
    }
}
