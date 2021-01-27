use crate::mqtt::packets::QoS;
use getset::Getters;

#[derive(Getters, Debug, PartialEq, Clone, Default)]
pub struct Subscription {
    topic: String,
    qos: QoS,
}

impl Subscription {
    pub fn new(topic: String, qos: QoS) -> Subscription {
        Subscription { topic, qos }
    }
}
