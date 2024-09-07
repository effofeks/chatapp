use std::io::Error;

use crossbeam::channel::{Receiver, Sender};

pub enum NetEvent {
    Message { from: String, body: String },
    ShutDown {},
}

pub fn run_net(tx: Sender<NetEvent>, rx: Receiver<NetEvent>) -> Result<(), Error> {
    loop {
        let event = rx.recv().unwrap();
        match event {
            NetEvent::Message { from, body } => send_message(from, body),
            NetEvent::ShutDown {} => break,
        }
    }
    Ok(())
}

fn send_message(from: String, body: String) {
    // TODO: publish message to network
}
