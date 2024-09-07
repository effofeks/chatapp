use crossbeam::channel::{Receiver, Sender};
use crossterm::{execute, terminal};
use std::{io::Error, thread, time::Duration};
use tui::{backend, widgets::Block, Terminal};

pub enum UiEvent {
    Message { from: String, body: String },
    ShutDown {},
}

pub fn run_ui(tx: Sender<UiEvent>, rx: Receiver<UiEvent>) -> Result<(), Error> {
    terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    let backend = backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| {
        let block = Block::default().title("Hello, world!");
        f.render_widget(block, f.size());
    })?;
    tx.send(UiEvent::Message {
        from: "Robyn".to_string(),
        body: "WTF is going on".to_string(),
    }).unwrap();
    thread::sleep(Duration::from_secs(5));
    tx.send(UiEvent::ShutDown {}).unwrap();
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
