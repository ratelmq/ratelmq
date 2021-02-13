use crate::mqtt::packets::QoS;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Subscription {
    topic: String,
    qos: QoS,
}

impl Display for Subscription {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "topic: '{}' QoS: {}", self.topic, self.qos)
    }
}

impl Subscription {
    pub fn new(topic: String, qos: QoS) -> Subscription {
        Subscription { topic, qos }
    }

    pub fn topic(&self) -> &str {
        self.topic.as_str()
    }
}
