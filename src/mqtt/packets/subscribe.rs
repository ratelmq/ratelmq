use crate::mqtt::subscription::Subscription;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct SubscribePacket {
    pub packet_id: u16,

    pub subscriptions: Vec<Subscription>,
}

impl SubscribePacket {
    pub fn new(packet_id: u16, subscriptions: Vec<Subscription>) -> Self {
        SubscribePacket {
            packet_id,
            subscriptions,
        }
    }
}
