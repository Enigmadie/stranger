use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use ratatui::prelude::*;
use std::io::{self, stdout, Stdout};

use stranger::{
    app::{cleanup_terminal, utils::config_parser::load_config},
    App,
};

type TuiTerminal = Terminal<CrosstermBackend<Stdout>>;

trait TerminalOps {
    type Terminal;

    fn enable_raw_mode(&mut self) -> io::Result<()>;
    fn enter_screen(&mut self) -> io::Result<()>;
    fn create_terminal(&mut self) -> io::Result<Self::Terminal>;
    fn cleanup(&mut self) -> io::Result<()>;
}

struct CrosstermTerminalOps;

impl TerminalOps for CrosstermTerminalOps {
    type Terminal = TuiTerminal;

    fn enable_raw_mode(&mut self) -> io::Result<()> {
        enable_raw_mode()
    }

    fn enter_screen(&mut self) -> io::Result<()> {
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
    }

    fn create_terminal(&mut self) -> io::Result<Self::Terminal> {
        Terminal::new(CrosstermBackend::new(stdout()))
    }

    fn cleanup(&mut self) -> io::Result<()> {
        cleanup_terminal()
    }
}

struct TerminalGuard<O: TerminalOps> {
    ops: O,
    terminal: Option<O::Terminal>,
}

impl<O: TerminalOps> TerminalGuard<O> {
    fn new(mut ops: O) -> io::Result<Self> {
        ops.enable_raw_mode()?;
        let mut guard = Self {
            ops,
            terminal: None,
        };
        guard.ops.enter_screen()?;
        guard.terminal = Some(guard.ops.create_terminal()?);
        Ok(guard)
    }

    fn terminal_mut(&mut self) -> &mut O::Terminal {
        self.terminal
            .as_mut()
            .expect("terminal guard must contain an initialized terminal")
    }
}

impl<O: TerminalOps> Drop for TerminalGuard<O> {
    fn drop(&mut self) {
        if let Err(error) = self.ops.cleanup() {
            eprintln!("Failed to cleanup terminal: {error}");
        }
    }
}

fn main() -> io::Result<()> {
    let config = load_config();
    let mut app = App::new(config)?;
    let mut terminal = TerminalGuard::new(CrosstermTerminalOps)?;

    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = cleanup_terminal();
        default_panic_hook(info);
    }));

    app.run(terminal.terminal_mut())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::RefCell,
        panic::{catch_unwind, AssertUnwindSafe},
        rc::Rc,
    };

    #[derive(Clone, Copy)]
    enum Failure {
        EnableRaw,
        EnterScreen,
        CreateTerminal,
    }

    struct TestTerminalOps {
        events: Rc<RefCell<Vec<&'static str>>>,
        failure: Option<Failure>,
    }

    impl TerminalOps for TestTerminalOps {
        type Terminal = ();

        fn enable_raw_mode(&mut self) -> io::Result<()> {
            self.events.borrow_mut().push("enable_raw");
            match self.failure {
                Some(Failure::EnableRaw) => Err(io::Error::other("enable raw failed")),
                _ => Ok(()),
            }
        }

        fn enter_screen(&mut self) -> io::Result<()> {
            self.events.borrow_mut().push("enter_screen");
            match self.failure {
                Some(Failure::EnterScreen) => Err(io::Error::other("enter screen failed")),
                _ => Ok(()),
            }
        }

        fn create_terminal(&mut self) -> io::Result<Self::Terminal> {
            self.events.borrow_mut().push("create_terminal");
            match self.failure {
                Some(Failure::CreateTerminal) => Err(io::Error::other("create terminal failed")),
                _ => Ok(()),
            }
        }

        fn cleanup(&mut self) -> io::Result<()> {
            self.events.borrow_mut().push("cleanup");
            Ok(())
        }
    }

    fn ops(failure: Option<Failure>) -> (TestTerminalOps, Rc<RefCell<Vec<&'static str>>>) {
        let events = Rc::new(RefCell::new(Vec::new()));
        (
            TestTerminalOps {
                events: events.clone(),
                failure,
            },
            events,
        )
    }

    #[test]
    fn setup_failure_restores_terminal_after_raw_mode_is_enabled() {
        for failure in [Failure::EnterScreen, Failure::CreateTerminal] {
            let (ops, events) = ops(Some(failure));

            assert!(TerminalGuard::new(ops).is_err());
            assert_eq!(events.borrow().last(), Some(&"cleanup"));
        }

        let (ops, events) = ops(Some(Failure::EnableRaw));
        assert!(TerminalGuard::new(ops).is_err());
        assert_eq!(&*events.borrow(), &["enable_raw"]);
    }

    #[test]
    fn normal_exit_restores_terminal() {
        let (ops, events) = ops(None);
        {
            let _guard = TerminalGuard::new(ops).unwrap();
        }

        assert_eq!(events.borrow().last(), Some(&"cleanup"));
    }

    #[test]
    fn panic_restores_terminal() {
        let (ops, events) = ops(None);

        let result = catch_unwind(AssertUnwindSafe(|| {
            let _guard = TerminalGuard::new(ops).unwrap();
            panic!("test panic");
        }));

        assert!(result.is_err());
        assert_eq!(events.borrow().last(), Some(&"cleanup"));
    }
}
