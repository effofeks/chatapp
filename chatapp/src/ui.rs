use crossbeam::channel::{Receiver, Sender, TryRecvError};
use crossterm::{
    event::{self, KeyEvent, KeyModifiers},
    execute, terminal,
};
use std::{
    io::{Error, Stdout},
    time::Duration,
};
use tui::{
    backend::{self, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use crate::app::Message;

type UiTerminal = Terminal<CrosstermBackend<Stdout>>;
type UiFrame<'a> = Frame<'a, CrosstermBackend<Stdout>>;

type UiResult = Result<(), Error>;

const INPUT_POLL_TIMEOUT: u64 = 1;

pub enum UiEventIn {
    Messages { messages: Vec<Message> },
    #[allow(dead_code)]
    ShutDown {},
}

pub enum UiEventOut {
    Message { body: String },
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
    input: String,
}

impl UiState {
    fn default() -> UiState {
        UiState {
            messages: vec![],
            should_run: true,
            input: "".to_string(),
        }
    }

    fn set_messages(&mut self, messages: Vec<Message>) {
        self.messages = messages;
    }

    fn shutdown(&mut self) {
        self.should_run = false;
    }

    fn append_input(&mut self, c: char) {
        self.input.push(c);
    }

    fn chop_input(&mut self) {
        self.input.pop();
    }

    fn truncate_input(&mut self) {
        self.input.clear();
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
            self.render()?;
            self.handle_input()?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> UiResult {
        let result = self
            .from_app
            .try_recv();

        match result {
            Ok(event) => {
                match event {
                    UiEventIn::Messages { messages } => self.handle_messages(messages)?,
                    UiEventIn::ShutDown {} => self.handle_shutdown()?,
                };
            }
            Err(TryRecvError::Empty) => (),
            Err(e) => panic!("{}", e.to_string()),
        }

        Ok(())
    }

    fn handle_messages(&mut self, messages: Vec<Message>) -> UiResult {
        self.state.set_messages(messages);
        Ok(())
    }

    fn handle_input(&mut self) -> UiResult {
        if event::poll(Duration::from_millis(INPUT_POLL_TIMEOUT))? {
            let e = event::read()?;
            match e {
                event::Event::Key(key_event) => {
                    self.handle_key_input(key_event)?;
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn handle_key_input(&mut self, key_event: KeyEvent) -> UiResult {
        match key_event.code {
            event::KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.shutdown()?;
            }
            event::KeyCode::Char(c) => self.state.append_input(c),
            event::KeyCode::Backspace => self.state.chop_input(),
            event::KeyCode::Enter => self.send_message(),
            _ => (),
        }
        Ok(())
    }

    fn shutdown(&mut self) -> UiResult {
        self.handle_shutdown()?;
        self.to_app.send(UiEventOut::ShutDown {}).unwrap();
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
            render_input(&self.state, f, chunks[1]).unwrap();
        })?;
        Ok(())
    }

    fn send_message(&mut self) {
        let body = self.state.input.clone();
        self.state.truncate_input();
        self.to_app.send(UiEventOut::Message { body }).unwrap();
    }
}

fn render_messages(state: &UiState, frame: &mut UiFrame, chunk: Rect) -> UiResult {
    let messages: Vec<Spans> = state
        .messages
        .iter()
        .map(|m| Spans::from(Span::raw(format!("{}: {}", m.from, m.body))))
        .collect();

    let block = Block::default().title("Messages").borders(Borders::all());
    let paragraph = Paragraph::new(messages).block(block);
    frame.render_widget(paragraph, chunk);
    Ok(())
}

fn render_input(state: &UiState, frame: &mut UiFrame, chunk: Rect) -> UiResult {
    let block = Block::default()
        .title("Compose message")
        .borders(Borders::all());
    let paragraph = Paragraph::new(state.input.clone()).block(block);
    frame.render_widget(paragraph, chunk);
    Ok(())
}
