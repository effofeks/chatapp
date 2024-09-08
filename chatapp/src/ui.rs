use crossbeam::channel::{Receiver, Sender};
use crossterm::{execute, terminal};
use std::io::{Error, Stdout};
use tui::{
    backend::{self, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    text::{Span, Spans},
    widgets::Paragraph,
    Frame, Terminal,
};

use crate::app::Message;

type UiTerminal = Terminal<CrosstermBackend<Stdout>>;
type UiFrame<'a> = Frame<'a, CrosstermBackend<Stdout>>;

type UiResult = Result<(), Error>;

pub enum UiEventIn {
    Messages { messages: Vec<Message> },
    ShutDown {},
}

pub enum UiEventOut {
    Message { from: String, body: String },
    ShutDown {},
}

pub fn run_ui(to_app: Sender<UiEventOut>, from_app: Receiver<UiEventIn>) -> UiResult {
    let mut ui = UiManager::new(to_app, from_app);
    ui.init()?;
    ui.run()
}

struct UiState {
    messages: Vec<Message>,
    should_run: bool,
}

impl UiState {
    fn default() -> UiState {
        UiState {
            messages: vec![],
            should_run: true,
        }
    }

    fn set_messages(&mut self, messages: Vec<Message>) {
        self.messages = messages;
    }

    fn shutdown(&mut self) {
        self.should_run = false;
    }
}

struct UiManager {
    to_app: Sender<UiEventOut>,
    from_app: Receiver<UiEventIn>,
    state: UiState,
    terminal: UiTerminal,
}

impl UiManager {
    fn new(to_app: Sender<UiEventOut>, from_app: Receiver<UiEventIn>) -> UiManager {
        let stdout = std::io::stdout();
        let backend = backend::CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();

        let state = UiState::default();

        UiManager {
            to_app,
            from_app,
            state,
            terminal,
        }
    }

    fn init(&mut self) -> UiResult {
        terminal::enable_raw_mode()?;
        execute!(self.terminal.backend_mut(), terminal::EnterAlternateScreen)?;
        Ok(())
    }

    fn run(&mut self) -> UiResult {
        while self.state.should_run {
            let event = self.from_app.recv().unwrap();
            match event {
                UiEventIn::Messages { messages } => self.handle_messages(messages)?,
                UiEventIn::ShutDown {} => self.handle_shutdown()?,
            };
            self.render()?;
        }

        Ok(())
    }

    fn handle_messages(&mut self, messages: Vec<Message>) -> UiResult {
        self.state.set_messages(messages);
        Ok(())
    }

    fn handle_shutdown(&mut self) -> UiResult {
        terminal::disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;

        self.state.shutdown();
        Ok(())
    }

    fn render(&mut self) -> UiResult {
        self.terminal.draw(|f: &mut UiFrame| {
            let size = f.size();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_slice());

            let chunks = layout.split(size);
            render_messages(&self.state, f, chunks[0]).unwrap();
        })?;
        Ok(())
    }
}

fn render_messages(state: &UiState, frame: &mut UiFrame, chunk: Rect) -> UiResult {
    let messages: Vec<Spans> = state
        .messages
        .iter()
        .map(|m| Spans::from(Span::raw(format!("{}: {}", m.from, m.body))))
        .collect();

    let paragraph = Paragraph::new(messages);
    frame.render_widget(paragraph, chunk);
    Ok(())
}
