use kafka::client::KafkaClient;
use kafka::error::Error as KafkaError;
use std::{process, thread, time::Duration};
use whoami;

const MAX_ITER_LOAD_METADATA: u8 = 5;
const POOL_CLIENT_WAIT_TIME: Duration = Duration::from_millis(500);

pub struct Domain {
    origin: u32,
    identity: Option<String>,
    kafka_client: KafkaClient,
}

impl Domain {
    /// Create a new instance of Domain,
    pub fn new() -> Domain {
        Domain {
            origin: process::id(),
            identity: None,
            kafka_client: KafkaClient::new(Domain::get_client_hosts()),
        }
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

    /// Register topics.
    pub fn register_topics<T: AsRef<str>>(&mut self, topics: &[T]) -> Result<(), KafkaError> {
        for _ in 0..MAX_ITER_LOAD_METADATA {
            let result = self.kafka_client.load_metadata(topics);
            match result {
                Ok(_) => {
                    let partitions_unloaded: Vec<bool> = topics
                        .iter()
                        .filter_map(|topic| {
                            if self
                                .kafka_client
                                .topics()
                                .partitions(topic.as_ref().into())
                                .map(|p| p.len())
                                .unwrap_or(0)
                                > 0
                            {
                                None
                            } else {
                                Some(true)
                            }
                        })
                        .collect();
                    if partitions_unloaded.is_empty() {
                        return Ok(());
                    }
                }
                Err(err) => return Err(err),
            }
            thread::sleep(POOL_CLIENT_WAIT_TIME);
        }
        Ok(())
    }

    /// Get client host address.
    pub fn get_client_hosts() -> Vec<String> {
        // FIXME: Handle general case (tribeiro/rs_salobj#42).
        vec!["localhost:9092".to_owned()]
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
}
