//! Stores information and behavior shared by all clients.
//!
//! The [Domain] struct and its implementation stores information about the
//! client identity and origin and holds an instance to the kafka client.
//!
//! When developing applications with salobj you will want to reduce the number
//! of [Domain] instances, ideally having only one per application.

use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic},
    client::DefaultClientContext,
    error::KafkaError,
    ClientConfig,
};
use std::env;
use std::process;
use whoami::{self, fallible};

const DEFAULT_LSST_KAFKA_CLIENT_ADDR: &str = "localhost:9092";
const DEFAULT_LSST_SCHEMA_REGISTRY_URL: &str = "http://127.0.0.1:8081";

pub struct Domain {
    origin: u32,
    identity: Option<String>,
}

impl Default for Domain {
    fn default() -> Self {
        Self::new()
    }
}

impl Domain {
    /// Create a new instance of Domain,
    pub fn new() -> Domain {
        Domain {
            origin: process::id(),
            identity: None,
        }
    }

    /// Return the default identify.
    pub fn get_default_identity(&self) -> String {
        let username = whoami::username();
        let hostname = fallible::hostname().unwrap_or("unknown".to_string());
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
    pub async fn register_topics<T: AsRef<str>>(&self, topics: &[T]) -> Result<(), KafkaError> {
        let timeout = std::time::Duration::from_secs(5);
        let client_config = Domain::get_admin_client_configuration();
        let admin_client: AdminClient<DefaultClientContext> = client_config.create()?;
        let admin_options = AdminOptions::new();

        let missing_topics: Vec<String> = topics
            .iter()
            .filter_map(|topic| {
                if let Ok(metadata) = admin_client
                    .inner()
                    .fetch_metadata(Some(topic.as_ref()), timeout)
                {
                    if metadata.topics()[0].error().is_some() {
                        Some(topic.as_ref().to_string())
                    } else {
                        None
                    }
                } else {
                    Some(topic.as_ref().to_string())
                }
            })
            .collect();

        for topic_name in missing_topics.into_iter() {
            match admin_client
                .create_topics(
                    &[NewTopic::new(
                        &topic_name,
                        1,
                        rdkafka::admin::TopicReplication::Fixed(1),
                    )],
                    &admin_options,
                )
                .await
            {
                Ok(topic_result) => {
                    log::info!("Topic {topic_name} created successfully: {topic_result:?}");
                }
                Err(error) => {
                    log::error!("Failed to create topic {topic_name}: {error:?}");
                }
            }
        }
        Ok(())
    }

    /// Get client host address.
    ///
    /// This method will look for the LSST_KAFKA_CLIENT_ADDR environment
    /// variable and return a default value if it is not set. Usually the
    /// default is only good enough for local testing. For production this
    /// environment variable should be set.
    pub fn get_client_hosts() -> Vec<String> {
        match env::var("LSST_KAFKA_BROKER_ADDR") {
            Ok(kafka_client_addr) => kafka_client_addr
                .split(',')
                .map(|addr| addr.to_owned())
                .collect(),
            Err(_) => vec![DEFAULT_LSST_KAFKA_CLIENT_ADDR.to_owned()],
        }
    }

    /// Get schema registry url.
    ///
    /// This method will look for the LSST_SCHEMA_REGISTRY_URL environment
    /// variable and return a default value if it is not set. Usually the
    /// default is only good enough for local testing. For production this
    /// environment variable should be set.
    pub fn get_schema_registry_url() -> String {
        match env::var("LSST_SCHEMA_REGISTRY_URL") {
            Ok(schema_registry_url) => schema_registry_url,
            Err(_) => DEFAULT_LSST_SCHEMA_REGISTRY_URL.to_owned(),
        }
    }

    pub fn get_producer_configuration() -> ClientConfig {
        ClientConfig::new()
            .set("bootstrap.servers", Domain::get_client_hosts()[0].clone())
            .clone()
    }

    pub fn get_consumer_configuration(&self) -> ClientConfig {
        ClientConfig::new()
            .set("bootstrap.servers", Domain::get_client_hosts()[0].clone())
            .set("allow.auto.create.topics", "false")
            .set("auto.offset.reset", "earliest")
            .clone()
    }

    pub fn get_admin_client_configuration() -> ClientConfig {
        ClientConfig::new()
            .set("bootstrap.servers", Domain::get_client_hosts()[0].clone())
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{Domain, DEFAULT_LSST_KAFKA_CLIENT_ADDR, DEFAULT_LSST_SCHEMA_REGISTRY_URL};
    use std::env;

    #[test]
    fn get_default_identity() {
        let domain = Domain::new();

        let default_identity = domain.get_default_identity();

        assert!(default_identity.contains("@"))
    }

    #[test]
    fn get_client_hosts_env_not_set() {
        let current_lsst_kafka_client_addr = env::var("LSST_KAFKA_CLIENT_ADDR");

        if current_lsst_kafka_client_addr.is_ok() {
            env::remove_var("LSST_KAFKA_CLIENT_ADDR");
        }

        let default_value = DEFAULT_LSST_KAFKA_CLIENT_ADDR.to_owned();
        let value = Domain::get_client_hosts()[0].to_owned();
        if let Ok(current_lsst_kafka_client_addr) = current_lsst_kafka_client_addr {
            env::set_var("LSST_KAFKA_CLIENT_ADDR", current_lsst_kafka_client_addr);
        }
        assert_eq!(value, default_value)
    }

    #[test]
    fn get_client_hosts_env_set() {
        env::set_var(
            "LSST_KAFKA_BROKER_ADDR",
            "kafka_client_1:9092,kafka_client_2:9092",
        );

        let client_hosts = Domain::get_client_hosts();

        assert_eq!(client_hosts.len(), 2);
        assert!(client_hosts.contains(&"kafka_client_1:9092".to_owned()));
        assert!(client_hosts.contains(&"kafka_client_2:9092".to_owned()));
    }

    #[test]
    fn get_schema_registry_url_env_not_set() {
        if env::var("LSST_SCHEMA_REGISTRY_URL").is_ok() {
            env::remove_var("LSST_SCHEMA_REGISTRY_URL");
        }

        let default_value = DEFAULT_LSST_SCHEMA_REGISTRY_URL.to_owned();

        env::remove_var("LSST_KAFKA_BROKER_ADDR");

        assert_eq!(Domain::get_schema_registry_url(), default_value)
    }

    #[test]
    fn get_schema_registry_url_env_set() {
        env::set_var(
            "LSST_SCHEMA_REGISTRY_URL",
            "http://lsst-schema-registry.lsst.codes:8081",
        );

        let schema_registry_url = Domain::get_schema_registry_url();

        env::remove_var("LSST_SCHEMA_REGISTRY_URL");

        assert_eq!(
            schema_registry_url,
            "http://lsst-schema-registry.lsst.codes:8081"
        )
    }
}
