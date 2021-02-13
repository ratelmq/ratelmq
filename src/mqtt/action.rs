use crate::mqtt::packets::ControlPacket;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Action {
    pub packet: ControlPacket,
    pub response: Sender<ControlPacket>,
}

impl Action {
    pub fn new(packet: ControlPacket, response: Sender<ControlPacket>) -> Self {
        Action { packet, response }
    }
}
