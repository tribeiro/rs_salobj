use kafka::{client, consumer};
use std::process;
use whoami;

pub struct Domain {
    origin: u32,
    identity: Option<String>,
    client: client::KafkaClient,
}

impl Domain {
    /// Create a new instance of Domain,
    pub fn new() -> Domain {
        let mut client = client::KafkaClient::new(Domain::get_hosts());
        client.load_metadata_all().unwrap();
        Domain {
            origin: process::id(),
            identity: None,
            client: client,
        }
    }

    pub fn get_hosts() -> Vec<String> {
        vec!["localhost:9092".to_owned()]
    }

    /// Return the default identify.
    pub fn get_default_identity(&self) -> String {
        let username = whoami::username();
        let hostname = whoami::hostname();
        format!("{username}@{hostname}")
    }

    pub fn get_origin(&self) -> u32 {
        self.origin
    }

    pub fn get_identity(&self) -> String {
        match &self.identity {
            Some(identity) => identity.to_owned(),
            None => self.get_default_identity(),
        }
    }

    pub fn get_client(&self) -> &client::KafkaClient {
        &self.client
    }

    pub fn get_topics(&self) -> client::metadata::Topics<'_> {
        self.client.topics()
    }

    pub fn make_consumer(
        &self,
        topic_name: String,
        fallback_offset: consumer::FetchOffset,
    ) -> consumer::Consumer {
        consumer::Consumer::from_client(self.client.)
            .with_topic(topic_name)
            .with_fallback_offset(fallback_offset)
            .with_group(format!("{}", self.get_origin()))
            .with_offset_storage(consumer::GroupOffsetStorage::Kafka)
            .create()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::Domain;

    #[test]
    fn get_default_identity() {
        let domain = Domain::new();

        let default_identity = domain.get_default_identity();

        assert!(default_identity.contains("@"))
    }

    #[test]
    fn get_client() {
        let domain = Domain::new();

        let client = domain.get_client();

        let hosts = client.hosts();

        assert_eq!(hosts, Domain::get_hosts());
    }
}
