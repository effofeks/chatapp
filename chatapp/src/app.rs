use std::io::Error;

use crossbeam::channel::{Receiver, Sender};

use crate::{net::NetEvent, ui::UiEvent};

pub fn run_app(
    from_ui: Receiver<UiEvent>,
    to_ui: Sender<UiEvent>,
    from_net: Receiver<NetEvent>,
    to_net: Sender<NetEvent>,
) -> Result<(), Error> {
    loop {
        let event = from_ui.recv().unwrap();
        match event {
            UiEvent::Message { from, body } => handle_message(from, body, &to_net),
            UiEvent::ShutDown {} => {
                handle_shut_down(&to_net);
                break;
            }
        }
    }
    Ok(())
}

fn handle_message(from: String, body: String, to_net: &Sender<NetEvent>) {
    to_net
        .send(NetEvent::Message {
            from: from,
            body: body,
        })
        .unwrap();
}

fn handle_shut_down(to_net: &Sender<NetEvent>) {
    to_net.send(NetEvent::ShutDown {}).unwrap();
}
