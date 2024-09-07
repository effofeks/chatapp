use std::{io::Error, thread};

use crossbeam::channel;
use net::NetEvent;
use ui::UiEvent;

mod net;
mod ui;
mod app;

fn main() -> Result<(), Error> {
    let (from_ui_tx, from_ui_rx) = channel::unbounded::<UiEvent>();
    let (to_ui_tx, to_ui_rx) = channel::unbounded::<UiEvent>();
    let (from_net_tx, from_net_rx) = channel::unbounded::<NetEvent>();
    let (to_net_tx, to_net_rx) = channel::unbounded::<NetEvent>();
    
    let ui_handle = thread::spawn(move || {
        ui::run_ui(from_ui_tx, to_ui_rx).expect("Something went wrong.");
    });

    let net_handle = thread::spawn(move || {
        net::run_net(from_net_tx, to_net_rx).expect("Something went wrong.");
    });

    app::run_app(from_ui_rx, to_ui_tx, from_net_rx, to_net_tx)?;

    ui_handle.join().unwrap();
    net_handle.join().unwrap();
    Ok(())
}
