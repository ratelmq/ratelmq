use std::collections::HashMap;

use crate::mqtt::packets::suback::SubAckReturnCode;
use crate::mqtt::packets::ClientId;
use crate::mqtt::subscription::Subscription;

pub struct SubscriptionsRepository {
    root: SubscriptionNode,
}

impl SubscriptionsRepository {
    pub fn new() -> SubscriptionsRepository {
        SubscriptionsRepository {
            root: SubscriptionNode::new(),
        }
    }

    pub fn subscribe(
        &mut self,
        client_id: &ClientId,
        subscription: &Subscription,
    ) -> SubAckReturnCode {
        let mut node = &mut self.root;

        // subscription.topic().split("/").for_each(|x| {
        //     node = node.children.entry(x.to_string()).or_insert(SubscriptionNode::new());
        // });

        let segments: Vec<&str> = subscription.topic().split("/").collect();
        for segment in segments {
            node = node
                .children
                .entry(segment.to_string())
                .or_insert(SubscriptionNode::new());
        }
        node.clients.push(client_id.clone());
        // println!("Nodes: {:?}", &self.root);

        SubAckReturnCode::SuccessQoS0
    }

    pub fn unsubscribe(&mut self, client_id: &ClientId, topics: &Vec<String>) {
        for topic in topics {
            // if let Some(client_ids) = self.subscriptions.get_mut(topic) {
            // client_ids.remove(client_id);
            // }
            let mut node = &mut self.root;

            // subscription.topic().split("/").for_each(|x| {
            //     node = node.children.entry(x.to_string()).or_insert(SubscriptionNode::new());
            // });

            let segments: Vec<&str> = topic.split("/").collect();
            for segment in segments {
                node = node
                    .children
                    .entry(segment.to_string())
                    .or_insert(SubscriptionNode::new());
            }

            node.clients
                .iter()
                .position(|c| c == client_id)
                .map(|p| node.clients.remove(p));
            // node.clients.(client_id.clone());
            // println!("Nodes: {:?}", &self.root);
        }
    }

    pub fn disconnected(&mut self, client_id: &ClientId) {}

    pub fn connections_lost(&mut self, client_id: &ClientId) {}

    pub fn subscribed_clients(&self, topic: &String) -> Option<Vec<ClientId>> {
        let mut client_ids = Vec::<ClientId>::new();

        let mut nodes = vec![&self.root];
        let segments: Vec<&str> = topic.split("/").collect();
        for segment in segments {
            // nodes.iter().or itera
            let mut descendant_nodes = Vec::new();
            for node in nodes {
                if let Some(direct) = node.children.get(segment) {
                    descendant_nodes.push(direct);
                };

                if let Some(plus) = node.children.get("+") {
                    descendant_nodes.push(plus);
                };

                if let Some(node) = node.children.get("#") {
                    node.clients
                        .iter()
                        .for_each(|client_id| client_ids.push(client_id.clone()))
                }
            }
            nodes = descendant_nodes;
        }

        nodes
            .iter()
            .flat_map(|&node| &node.clients)
            .for_each(|client_id| client_ids.push(client_id.clone()));

        Some(client_ids)
        // None
    }
}

#[derive(Debug)]
struct SubscriptionNode {
    pub children: HashMap<String, SubscriptionNode>,
    pub clients: Vec<ClientId>,
}

impl SubscriptionNode {
    pub fn new() -> SubscriptionNode {
        SubscriptionNode {
            children: HashMap::new(),
            clients: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mqtt::packets::QoS;

    use super::*;

    #[test]
    fn test_subscribe_no_wildcard() {
        let mut repo = SubscriptionsRepository::new();
        let subscription = Subscription::new("asd/zxc/qwe".to_string(), QoS::AtMostOnce);
        let client_id = ClientId::from("client 1");

        let result = repo.subscribe(&client_id, &subscription);

        assert_eq!(result, SubAckReturnCode::SuccessQoS0)
    }

    #[test]
    fn test_subscribed_clients_no_wildcards_matching() {
        let mut repo = SubscriptionsRepository::new();

        subscribe(&mut repo, "a/b/c", "c1");
        subscribe(&mut repo, "a/b/c", "c2");

        assert(
            repo.subscribed_clients(&"a/b/c".to_string()),
            Some(vec!["c1".to_string(), "c2".to_string()]),
        )
    }

    #[test]
    fn test_subscribed_clients_no_wildcards_not_matching() {
        let mut repo = SubscriptionsRepository::new();

        subscribe(&mut repo, "a", "c1");
        subscribe(&mut repo, "a/b", "c2");
        subscribe(&mut repo, "a/b/c/d", "c3");

        assert(repo.subscribed_clients(&"a/b/c".to_string()), Some(vec![]))
    }

    #[test]
    fn test_subscribed_clients_no_wildcards_combined() {
        let mut repo = SubscriptionsRepository::new();

        subscribe(&mut repo, "a/b/c", "c1");
        subscribe(&mut repo, "a/b/d", "c2");
        subscribe(&mut repo, "a/b/c/d", "c3");
        subscribe(&mut repo, "a/b", "c4");

        assert(
            repo.subscribed_clients(&"a/b/c".to_string()),
            Some(vec!["c1".to_string()]),
        )
    }

    #[test]
    fn test_subscribed_clients_wildcard_plus_matching() {
        let mut repo = SubscriptionsRepository::new();

        subscribe(&mut repo, "+/b/c", "c1");
        subscribe(&mut repo, "a/+/c", "c2");
        subscribe(&mut repo, "a/b/+", "c3");
        subscribe(&mut repo, "+/b/+", "c4");
        subscribe(&mut repo, "+/+/+", "c5");

        assert(
            repo.subscribed_clients(&"a/b/c".to_string()),
            Some(vec![
                "c1".to_string(),
                "c2".to_string(),
                "c3".to_string(),
                "c4".to_string(),
                "c5".to_string(),
            ]),
        );
    }

    #[test]
    fn test_subscribed_clients_wildcard_plus_not_matching() {
        let mut repo = SubscriptionsRepository::new();

        subscribe(&mut repo, "+/+/+/+", "c1");
        subscribe(&mut repo, "+/+", "c2");
        subscribe(&mut repo, "+", "c3");
        subscribe(&mut repo, "a/+", "c4");
        subscribe(&mut repo, "a/+/d", "c5");

        assert(repo.subscribed_clients(&"a/b/c".to_string()), Some(vec![]));
    }

    #[test]
    fn test_subscribed_clients_wildcard_plus_combined() {
        let mut repo = SubscriptionsRepository::new();

        subscribe(&mut repo, "a/b/+", "c1");
        subscribe(&mut repo, "+/b/+", "c2");
        subscribe(&mut repo, "+/+/+", "c3");
        subscribe(&mut repo, "+/+/+/+", "cx");
        subscribe(&mut repo, "+/+", "cx");

        assert(
            repo.subscribed_clients(&"a/b/c".to_string()),
            Some(vec!["c1".to_string(), "c2".to_string(), "c3".to_string()]),
        );
    }

    #[test]
    fn test_subscribed_clients_wildcard_hash_matching() {
        let mut repo = SubscriptionsRepository::new();

        subscribe(&mut repo, "a/#", "c1");
        subscribe(&mut repo, "a/b/#", "c2");

        assert(
            repo.subscribed_clients(&"a/b/c".to_string()),
            Some(vec!["c1".to_string(), "c2".to_string()]),
        );
    }

    #[test]
    fn test_subscribed_clients_wildcard_hash_not_matching() {
        let mut repo = SubscriptionsRepository::new();

        subscribe(&mut repo, "a/b/c/#", "c1");
        subscribe(&mut repo, "a/d/#", "c2");
        subscribe(&mut repo, "b/#", "c3");

        assert(repo.subscribed_clients(&"a/b/c".to_string()), Some(vec![]));
    }

    #[test]
    fn test_subscribed_clients_wildcard_hash_combined() {
        let mut repo = SubscriptionsRepository::new();

        subscribe(&mut repo, "a/#", "c1");
        subscribe(&mut repo, "a/d/#", "c2");
        subscribe(&mut repo, "b/#", "c3");

        assert(
            repo.subscribed_clients(&"a/b/c".to_string()),
            Some(vec!["c1".to_string()]),
        );
    }

    #[test]
    fn test_subscribed_clients_combined() {
        let mut repo = SubscriptionsRepository::new();

        subscribe(&mut repo, "a/b/c", "c1");
        subscribe(&mut repo, "a/b/d", "cx");

        subscribe(&mut repo, "a/+/+", "c2");
        subscribe(&mut repo, "a/+/d", "cy");

        subscribe(&mut repo, "a/#", "c3");
        subscribe(&mut repo, "a/d/#", "cz");

        assert(
            repo.subscribed_clients(&"a/b/c".to_string()),
            Some(vec!["c1".to_string(), "c2".to_string(), "c3".to_string()]),
        );
    }

    fn assert(actual: Option<Vec<ClientId>>, expected: Option<Vec<ClientId>>) {
        let a = actual.map(|mut v| {
            v.sort();
            v
        });
        let e = expected.map(|mut v| {
            v.sort();
            v
        });
        println!("Actual: {:?}; Expected: {:?}", &a, &e);
        assert_eq!(a, e);
    }

    fn subscribe(repo: &mut SubscriptionsRepository, topic: &str, client_id: &str) {
        let subscription = Subscription::new(topic.to_string(), QoS::AtMostOnce);
        let client_id = ClientId::from(client_id);

        let result = repo.subscribe(&client_id, &subscription);
        assert_eq!(result, SubAckReturnCode::SuccessQoS0)
    }
}
