//! High-level representation of topics.
//!
//! This module contains utilities to represent a component topic structure
//! in sensible way as well as mechanisms to convert topic definition into
//! avro schema.
//!

use apache_avro::Schema;

use crate::error::errors::{SalObjError, SalObjResult};

/// Information about one topic.
pub struct TopicInfo {
    component_name: String,
    topic_subname: String,
    topic_name: String,
    indexed: bool,
    schema: Option<apache_avro::Schema>,
    rev_code: Option<String>,
    description: String,
    partitions: usize,
}

impl Default for TopicInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl TopicInfo {
    /// Create a new empty instance of `TopicInfo`.
    pub fn new() -> TopicInfo {
        TopicInfo {
            component_name: String::new(),
            topic_subname: String::new(),
            topic_name: String::new(),
            indexed: false,
            schema: None,
            rev_code: None,
            description: String::new(),
            partitions: 0,
        }
    }

    pub fn with_component(mut self, component_name: &str) -> Self {
        self.component_name = component_name.to_owned();
        self
    }

    pub fn with_topic_subname(mut self, topic_subname: &str) -> Self {
        self.topic_subname = topic_subname.to_owned();
        self
    }

    pub fn with_topic_name(mut self, topic_name: &str) -> Self {
        self.topic_name = topic_name.to_owned();
        self
    }

    pub fn with_schema(mut self, schema: apache_avro::Schema) -> Self {
        self.schema = Some(schema);
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_owned();
        self
    }

    pub fn with_partitions(mut self, partitions: usize) -> Self {
        self.partitions = partitions;
        self
    }

    pub fn with_indexed(mut self, indexed: bool) -> Self {
        self.indexed = indexed;
        self
    }

    pub fn with_rev_code(mut self, rev_code: Option<&str>) -> Self {
        if let Some(rev_code) = rev_code {
            self.rev_code = Some(rev_code.to_owned());
        }
        self
    }

    /// Get topic name.
    ///
    /// See (SalInfo)[crate::sal_info] for more information on topic naming
    /// conventions.
    pub fn get_topic_name(&self) -> &str {
        &self.topic_name
    }

    /// Get topic subname.
    ///
    /// See (SalInfo)[crate::sal_info] for more information on topic naming
    /// conventions.
    pub fn get_topic_subname(&self) -> &str {
        &self.topic_subname
    }

    /// Get a topic sal name.
    ///
    /// See (SalInfo)[crate::sal_info] for more information on topic naming
    /// conventions.
    pub fn get_sal_name(&self) -> String {
        format!("{}_{}", self.component_name, self.topic_name)
    }

    pub fn get_schema(&self) -> Option<Schema> {
        self.schema.clone()
    }

    /// Get number of partitions in this topic.
    ///
    /// This is a Kafka QoS property.
    pub fn get_partitions(&self) -> usize {
        self.partitions
    }

    /// Get revision code for the topic avro schema.
    pub fn get_rev_code(&self) -> SalObjResult<String> {
        if let Some(rev_code) = &self.rev_code {
            Ok(rev_code.to_owned())
        } else {
            Err(SalObjError::new(&format!(
                "Rev code not set for topic {} for {} component.",
                self.topic_name, self.component_name,
            )))
        }
    }

    /// Make schema for the topic.
    pub fn make_schema(&self) -> SalObjResult<apache_avro::Schema> {
        if let Some(schema) = &self.schema {
            Ok(schema.to_owned())
        } else {
            Err(SalObjError::new(&format!(
                "Schema not set for topic {} for {} component.",
                self.topic_name, self.component_name
            )))
        }
    }
}
