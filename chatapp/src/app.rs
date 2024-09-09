use std::io::Error;

use crossbeam::{
    channel::{Receiver, Sender},
    select,
};

use crate::{net::NetEventIn, net::NetEventOut, ui::UiEventIn, ui::UiEventOut};

type AppResult = Result<(), Error>;

const USER_NAME: &str = "Robyn";

pub fn run_app(
    from_ui: Receiver<UiEventOut>,
    to_ui: Sender<UiEventIn>,
    from_net: Receiver<NetEventOut>,
    to_net: Sender<NetEventIn>,
) -> Result<(), Error> {
    let mut app = App::new(from_ui, to_ui, from_net, to_net);
    app.run()
}

struct App {
    from_ui: Receiver<UiEventOut>,
    to_ui: Sender<UiEventIn>,
    from_net: Receiver<NetEventOut>,
    to_net: Sender<NetEventIn>,
    state: State,
}

impl App {
    fn new(
        from_ui: Receiver<UiEventOut>,
        to_ui: Sender<UiEventIn>,
        from_net: Receiver<NetEventOut>,
        to_net: Sender<NetEventIn>,
    ) -> App {
        App {
            from_ui: from_ui,
            to_ui: to_ui,
            from_net: from_net,
            to_net: to_net,
            state: State::default(),
        }
    }

    fn run(&mut self) -> AppResult {
        while self.state.should_run {
            select! {
                recv(self.from_ui) -> event => {
                    match event.unwrap() {
                        UiEventOut::Message { body } => self.handle_ui_message(body),
                        UiEventOut::ShutDown {} => self.handle_shutdown(),
                    }
                }
                recv(self.from_net) -> event => {
                    match event.unwrap() {
                        NetEventOut::Message { from, body } => self.handle_net_message(from , body),
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_ui_message(&mut self, body: String) {
        self.handle_message(&USER_NAME.to_string(), &body);
        self.to_net
            .send(NetEventIn::Message {
                from: USER_NAME.to_string(),
                body: body,
            })
            .unwrap();
    }

    fn handle_net_message(&mut self, from: String, body: String) {
        self.handle_message(&from, &body);
    }

    fn handle_message(&mut self, from: &String, body: &String) {
        self.state.add_message(&from, &body);
        let event = UiEventIn::Messages {
            messages: self.state.messages.clone(),
        };
        self.to_ui.send(event).unwrap();
    }

    fn handle_shutdown(&mut self) {
        self.to_net.send(NetEventIn::ShutDown {}).unwrap();
        self.state.shutdown();
    }

}

#[derive(Clone)]
pub struct Message {
    pub from: String,
    pub body: String,
}

struct State {
    should_run: bool,
    messages: Vec<Message>,
}

impl State {
    fn default() -> State {
        State {
            should_run: true,
            messages: vec![],
        }
    }

    fn shutdown(&mut self) {
        self.should_run = false;
    }

    fn add_message(&mut self, from: &String, body: &String) {
        let message = Message {
            from: from.clone(),
            body: body.clone(),
        };
        self.messages.push(message);
    }
}
