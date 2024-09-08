use std::{io::Error, net::UdpSocket, time::Duration};

use crossbeam::channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};

const BIND_ADDRESS: &str = "0.0.0.0:2510";
const DESTINATION_ADDRESS: &str = "127.0.0.1:2510";
const READ_TIMEOUT_MS: u64 = 100;
const RECV_TIMEOUT_MS: u64 = 100;
const UDP_BUFFER_SIZE: usize = 65507;

pub enum NetEventIn {
    Message { from: String, body: String },
    ShutDown {},
}

pub enum NetEventOut {
    Message { from: String, body: String },
}

#[derive(Serialize, Deserialize)]
enum NetPacket {
    Message { from: String, body: String },
}

pub fn run_net(to_app: Sender<NetEventOut>, from_app: Receiver<NetEventIn>) -> Result<(), Error> {
    let net = NetManager::new(to_app, from_app, BIND_ADDRESS);
    net.run()
}
struct NetManager {
    to_app: Sender<NetEventOut>,
    from_app: Receiver<NetEventIn>,
    socket: UdpSocket,
}

impl NetManager {
    fn new(to_app: Sender<NetEventOut>, from_app: Receiver<NetEventIn>, addr: &str) -> NetManager {
        let socket = UdpSocket::bind(addr).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_millis(READ_TIMEOUT_MS)))
            .unwrap();
        NetManager {
            to_app,
            from_app,
            socket,
        }
    }

    fn run(&self) -> Result<(), Error> {
        self.handle_recv_message("Robyn".to_string(), "I am stinky".to_string());
        loop {
            let result = self
                .from_app
                .recv_timeout(Duration::from_millis(RECV_TIMEOUT_MS));
            match result {
                Ok(event) => match event {
                    NetEventIn::Message { from, body } => self.handle_send_message(from, body),
                    NetEventIn::ShutDown {} => break,
                },
                Err(_) => (),
            };
            let mut buf = vec![0u8; UDP_BUFFER_SIZE];
            let result2 = self.socket.recv_from(&mut buf);
            match result2 {
                Ok((length, _)) => {
                    let payload = &buf[..length];
                    let packet = bincode::deserialize(payload).unwrap();
                    match packet {
                        NetPacket::Message { from, body } => self.handle_recv_message(from, body),
                    }
                }
                Err(_) => (),
            }
        }
        Ok(())
    }

    fn handle_send_message(&self, from: String, body: String) {
        let message = NetPacket::Message { from, body };
        let payload = bincode::serialize(&message).unwrap();
        self.socket.send_to(&payload, DESTINATION_ADDRESS).unwrap();
    }

    fn handle_recv_message(&self, from: String, body: String) {
        let message = NetEventOut::Message { from, body };
        self.to_app.send(message).unwrap();
    }
}
