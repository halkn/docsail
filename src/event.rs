use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use crate::app::{App, AppEvent};

pub trait EventSource {
    fn next_event(&mut self) -> io::Result<AppEvent>;
}

pub struct CrosstermEventSource;

impl EventSource for CrosstermEventSource {
    fn next_event(&mut self) -> io::Result<AppEvent> {
        loop {
            if let Some(event) = translate_event(event::read()?) {
                return Ok(event);
            }
        }
    }
}

pub fn run<S, D>(app: &mut App, event_source: &mut S, mut draw: D) -> io::Result<()>
where
    S: EventSource,
    D: FnMut(&App) -> io::Result<()>,
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
    match key_event.code {
        KeyCode::Char('j') | KeyCode::Down => Some(AppEvent::MoveDown),
        KeyCode::Char('k') | KeyCode::Up => Some(AppEvent::MoveUp),
        KeyCode::Enter => Some(AppEvent::Activate),
        KeyCode::Tab => Some(AppEvent::ToggleFocus),
        KeyCode::Char('r') => Some(AppEvent::Reload),
        KeyCode::Char('q') => Some(AppEvent::Quit),
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
            (KeyCode::Char('j'), AppEvent::MoveDown),
            (KeyCode::Down, AppEvent::MoveDown),
            (KeyCode::Char('k'), AppEvent::MoveUp),
            (KeyCode::Up, AppEvent::MoveUp),
            (KeyCode::Enter, AppEvent::Activate),
            (KeyCode::Tab, AppEvent::ToggleFocus),
            (KeyCode::Char('r'), AppEvent::Reload),
            (KeyCode::Char('q'), AppEvent::Quit),
        ];

        for (key_code, expected_event) in cases {
            let key_event = KeyEvent::new(key_code, KeyModifiers::NONE);

            assert_eq!(translate_event(Event::Key(key_event)), Some(expected_event));
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
}
