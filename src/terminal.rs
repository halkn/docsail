use std::io::{self, Stdout};

use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

pub trait TerminalControl {
    fn enable_raw_mode(&mut self) -> io::Result<()>;
    fn disable_raw_mode(&mut self) -> io::Result<()>;
    fn enter_alternate_screen(&mut self) -> io::Result<()>;
    fn leave_alternate_screen(&mut self) -> io::Result<()>;
    fn hide_cursor(&mut self) -> io::Result<()>;
    fn show_cursor(&mut self) -> io::Result<()>;
}

pub struct TerminalMode<C: TerminalControl> {
    control: C,
    is_active: bool,
}

impl<C: TerminalControl> TerminalMode<C> {
    pub fn activate(mut control: C) -> io::Result<Self> {
        control.enable_raw_mode()?;

        if let Err(error) = control.enter_alternate_screen() {
            let _ = control.disable_raw_mode();
            return Err(error);
        }

        if let Err(error) = control.hide_cursor() {
            let _ = control.leave_alternate_screen();
            let _ = control.disable_raw_mode();
            return Err(error);
        }

        Ok(Self {
            control,
            is_active: true,
        })
    }

    pub fn restore(&mut self) -> io::Result<()> {
        if !self.is_active {
            return Ok(());
        }

        let cursor_result = self.control.show_cursor();
        let screen_result = self.control.leave_alternate_screen();
        let raw_mode_result = self.control.disable_raw_mode();
        let result = cursor_result.and(screen_result).and(raw_mode_result);

        if result.is_ok() {
            self.is_active = false;
        }

        result
    }
}

impl<C: TerminalControl> Drop for TerminalMode<C> {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

pub struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    mode: TerminalMode<CrosstermControl>,
}

impl TerminalSession {
    pub fn enter() -> io::Result<Self> {
        let mode = TerminalMode::activate(CrosstermControl)?;
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

        Ok(Self { terminal, mode })
    }

    pub fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }

    pub fn restore(&mut self) -> io::Result<()> {
        self.mode.restore()
    }
}

struct CrosstermControl;

impl TerminalControl for CrosstermControl {
    fn enable_raw_mode(&mut self) -> io::Result<()> {
        enable_raw_mode()
    }

    fn disable_raw_mode(&mut self) -> io::Result<()> {
        disable_raw_mode()
    }

    fn enter_alternate_screen(&mut self) -> io::Result<()> {
        execute!(io::stdout(), EnterAlternateScreen)
    }

    fn leave_alternate_screen(&mut self) -> io::Result<()> {
        execute!(io::stdout(), LeaveAlternateScreen)
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        execute!(io::stdout(), Hide)
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        execute!(io::stdout(), Show)
    }
}

#[cfg(test)]
mod tests {
    use super::{TerminalControl, TerminalMode};
    use std::{cell::RefCell, io, rc::Rc};

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Operation {
        EnableRawMode,
        DisableRawMode,
        EnterAlternateScreen,
        LeaveAlternateScreen,
        HideCursor,
        ShowCursor,
    }

    struct FakeTerminalControl {
        operations: Rc<RefCell<Vec<Operation>>>,
        enter_error: bool,
        leave_error: bool,
    }

    impl FakeTerminalControl {
        fn new() -> (Self, Rc<RefCell<Vec<Operation>>>) {
            let operations = Rc::new(RefCell::new(Vec::new()));

            (
                Self {
                    operations: Rc::clone(&operations),
                    enter_error: false,
                    leave_error: false,
                },
                operations,
            )
        }

        fn with_enter_error() -> (Self, Rc<RefCell<Vec<Operation>>>) {
            let (mut control, operations) = Self::new();
            control.enter_error = true;
            (control, operations)
        }

        fn with_leave_error() -> (Self, Rc<RefCell<Vec<Operation>>>) {
            let (mut control, operations) = Self::new();
            control.leave_error = true;
            (control, operations)
        }
    }

    impl TerminalControl for FakeTerminalControl {
        fn enable_raw_mode(&mut self) -> io::Result<()> {
            self.operations.borrow_mut().push(Operation::EnableRawMode);
            Ok(())
        }

        fn disable_raw_mode(&mut self) -> io::Result<()> {
            self.operations.borrow_mut().push(Operation::DisableRawMode);
            Ok(())
        }

        fn enter_alternate_screen(&mut self) -> io::Result<()> {
            self.operations
                .borrow_mut()
                .push(Operation::EnterAlternateScreen);

            if self.enter_error {
                Err(io::Error::other("could not enter alternate screen"))
            } else {
                Ok(())
            }
        }

        fn leave_alternate_screen(&mut self) -> io::Result<()> {
            self.operations
                .borrow_mut()
                .push(Operation::LeaveAlternateScreen);

            if self.leave_error {
                Err(io::Error::other("could not leave alternate screen"))
            } else {
                Ok(())
            }
        }

        fn hide_cursor(&mut self) -> io::Result<()> {
            self.operations.borrow_mut().push(Operation::HideCursor);
            Ok(())
        }

        fn show_cursor(&mut self) -> io::Result<()> {
            self.operations.borrow_mut().push(Operation::ShowCursor);
            Ok(())
        }
    }

    #[test]
    fn restores_terminal_mode_after_activation() {
        let (control, operations) = FakeTerminalControl::new();
        let mut mode = TerminalMode::activate(control).unwrap();

        mode.restore().unwrap();

        assert_eq!(
            *operations.borrow(),
            vec![
                Operation::EnableRawMode,
                Operation::EnterAlternateScreen,
                Operation::HideCursor,
                Operation::ShowCursor,
                Operation::LeaveAlternateScreen,
                Operation::DisableRawMode,
            ]
        );
    }

    #[test]
    fn disables_raw_mode_when_entering_alternate_screen_fails() {
        let (control, operations) = FakeTerminalControl::with_enter_error();

        assert!(TerminalMode::activate(control).is_err());
        assert_eq!(
            *operations.borrow(),
            vec![
                Operation::EnableRawMode,
                Operation::EnterAlternateScreen,
                Operation::DisableRawMode,
            ]
        );
    }

    #[test]
    fn retries_restoration_after_leaving_alternate_screen_fails() {
        let (control, operations) = FakeTerminalControl::with_leave_error();
        let mut mode = TerminalMode::activate(control).unwrap();

        assert!(mode.restore().is_err());
        mode.control.leave_error = false;
        mode.restore().unwrap();

        assert_eq!(
            *operations.borrow(),
            vec![
                Operation::EnableRawMode,
                Operation::EnterAlternateScreen,
                Operation::HideCursor,
                Operation::ShowCursor,
                Operation::LeaveAlternateScreen,
                Operation::DisableRawMode,
                Operation::ShowCursor,
                Operation::LeaveAlternateScreen,
                Operation::DisableRawMode,
            ]
        );
    }
}
