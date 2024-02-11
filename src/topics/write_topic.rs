use apache_avro::{
    to_value,
    types::{Record, Value},
    Schema,
};

use crate::{
    domain::Domain,
    error::errors::SalObjError,
    sal_info::SalInfo,
    topics::{base_sal_topic::BaseSALTopic, base_topic::BaseTopic},
    utils::types::WriteTopicResult,
};
use chrono::Utc;
use kafka::{error::Result as KafkaResult, producer};
use rand::Rng;
use schema_registry_converter::{
    async_impl::avro::AvroEncoder, schema_registry_common::SubjectNameStrategy,
};
use serde::Serialize;
use std::{fmt::Debug, time::Duration};

/// Base struct for writing a topic.
pub struct WriteTopic<'a> {
    /// The name of the topic.
    topic_name: String,
    /// Index of the CSC, must be 0 if CSC is not indexed.
    index: i32,
    /// Is the CSC indexed?
    indexed: bool,
    /// An identifier of the current process.
    origin: i32,
    /// A string identifying the instance.
    identity: String,
    /// Data producer.
    producer: KafkaResult<producer::Producer>,
    /// Sequence number of the written samples. This number is incremented
    /// every time a sample is published.
    seq_num: i32,
    encoder: AvroEncoder<'a>,
    schema_registry_topic_name: String,
    schema: Schema,
}

impl<'a> BaseTopic for WriteTopic<'a> {}

impl<'a> WriteTopic<'a> {
    pub fn new(topic_name: &str, sal_info: &SalInfo, domain: &Domain) -> WriteTopic<'a> {
        let mut rng = rand::thread_rng();
        // FIXME: This needs to be properly handled!
        let schema = sal_info
            .get_topic_info(topic_name)
            .unwrap()
            .get_schema()
            .unwrap();

        WriteTopic {
            topic_name: topic_name.to_owned(),
            index: sal_info.get_index() as i32,
            indexed: sal_info.is_indexed(),
            origin: domain.get_origin() as i32,
            identity: domain.get_identity(),
            producer: producer::Producer::from_hosts(Domain::get_client_hosts())
                .with_ack_timeout(Duration::from_secs(1))
                .with_required_acks(producer::RequiredAcks::One)
                .create(),
            seq_num: rng.gen(),
            encoder: SalInfo::make_encoder(),
            schema_registry_topic_name: sal_info.make_schema_registry_topic_name(topic_name),
            schema,
        }
    }
    /// Get value of the origin identifier.
    ///
    /// This identifies the process running the current application.
    pub fn get_origin(&self) -> i32 {
        self.origin
    }

    /// Get instance identity.
    pub fn get_identity(&self) -> String {
        self.identity.to_owned()
    }

    /// Is the component indexed?
    ///
    /// This information is defined in the component interface.
    pub fn is_indexed(&self) -> bool {
        self.indexed
    }

    /// Component index.
    pub fn get_index(&self) -> i32 {
        self.index
    }

    /// Get Schema
    pub fn get_schema(&self) -> &Schema {
        &self.schema
    }

    /// Get topic name.
    pub fn get_topic_name(&self) -> String {
        self.topic_name.to_owned()
    }

    /// Get current sequence number.
    pub fn get_seq_num(&self) -> i32 {
        self.seq_num
    }

    /// Write the data.
    ///
    /// # Notes
    ///
    /// Originally the `private_sndStamp` has to be tai but this is writing it
    /// as utc. The precision is going to be microseconds.
    pub async fn write<'r>(&mut self, data: &mut Record<'r>) -> WriteTopicResult {
        // read current time in microseconds, as int, convert to f32 then
        // convert to seconds.
        self.seq_num += 1;
        let timestamp = Value::Union(
            0,
            Box::new(Value::Double(Utc::now().timestamp_micros() as f64 * 1e-6)),
        );
        data.put("private_sndStamp", timestamp.clone());
        data.put("private_efdStamp", timestamp.clone());
        data.put("private_kafkaStamp", timestamp);
        data.put("private_origin", Value::Int(self.get_origin()));
        data.put("private_identity", Value::String(self.get_identity()));
        data.put("private_revCode", Value::String("Not Set".to_owned()));
        data.put("private_seqNum", Value::Int(self.seq_num));
        data.put(
            "private_rcvStamp",
            Value::Union(0, Box::new(Value::Double(0.0))),
        );

        if self.is_indexed() {
            data.put("salIndex", Value::Int(self.get_index()));
        }

        for (key, value) in &data.fields {
            match value {
                Value::Null => println!("Attribute {key} not set."),
                _ => continue,
            }
        }
        let record_type = self.get_record_type();

        let key_strategy = SubjectNameStrategy::TopicRecordNameStrategy(
            self.schema_registry_topic_name.clone(),
            record_type,
        );
        let data_fields: Vec<(&str, Value)> =
            data.fields.iter().map(|(k, v)| (&**k, v.clone())).collect();

        match self.encoder.encode(data_fields, key_strategy).await {
            Ok(bytes) => match &mut self.producer {
                Ok(producer) => {
                    match producer.send(&producer::Record::from_key_value(
                        &self.schema_registry_topic_name,
                        format!("{{ \"name\": \"{}\" }}", self.schema_registry_topic_name),
                        bytes,
                    )) {
                        Ok(_) => Ok(self.seq_num),
                        Err(error) => Err(SalObjError::from_error(error)),
                    }
                }
                Err(error) => Err(SalObjError::new(&error.to_string())),
            },
            Err(error) => Err(SalObjError::from_error(error)),
        }
    }

    pub fn set_seq_num(&mut self, seq_num: i32) {
        self.seq_num = seq_num
    }

    /// Write the data.
    ///
    /// # Notes
    ///
    /// Originally the `private_sndStamp` has to be tai but this is writing it
    /// as utc. The precision is going to be microseconds.
    pub async fn write_typed<T>(&mut self, data: &T) -> WriteTopicResult
    where
        T: BaseSALTopic + Serialize + Debug,
    {
        // read current time in microseconds, as int, convert to f32 then
        // convert to seconds.
        if data.get_private_seq_num() != self.seq_num {
            return Err(SalObjError::new(&format!(
                "Input data has wrong sequence number. Must be {}, got {}.",
                self.seq_num,
                data.get_private_seq_num(),
            )));
        }
        self.seq_num += 1;

        if let Ok(data_value) = to_value(data) {
            if let Value::Record(data_record) = data_value {
                let record_type = self.get_record_type();

                let key_strategy = SubjectNameStrategy::TopicRecordNameStrategy(
                    self.schema_registry_topic_name.clone(),
                    record_type,
                );

                let data_fields: Vec<(&str, Value)> =
                    data_record.iter().map(|(k, v)| (&**k, v.clone())).collect();

                match self.encoder.encode(data_fields, key_strategy).await {
                    Ok(bytes) => match &mut self.producer {
                        Ok(producer) => {
                            match producer.send(&producer::Record::from_key_value(
                                &self.schema_registry_topic_name,
                                format!("{{ \"name\": \"{}\" }}", self.schema_registry_topic_name),
                                bytes,
                            )) {
                                Ok(_) => Ok(data.get_private_seq_num()),
                                Err(error) => Err(SalObjError::from_error(error)),
                            }
                        }
                        Err(error) => Err(SalObjError::new(&error.to_string())),
                    },
                    Err(error) => Err(SalObjError::from_error(error)),
                }
            } else {
                Err(SalObjError::new("Failed to convert value to record."))
            }
        } else {
            Err(SalObjError::new("Failed to serialize data."))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::domain::Domain;

    #[test]
    fn test_basics() {
        let domain = Domain::new();
        let sal_info = SalInfo::new("Test", 1).unwrap();
        let write_topic = WriteTopic::new("scalars", &sal_info, &domain);

        assert_eq!(write_topic.is_indexed(), true);
        assert_eq!(write_topic.get_index(), 1);
        assert_eq!(write_topic.get_topic_name(), "scalars");
    }

    #[test]
    #[should_panic]
    fn new_with_bad_topic_name() {
        let domain = Domain::new();
        let sal_info = SalInfo::new("Test", 1).unwrap();

        WriteTopic::new("inexistentTopic", &sal_info, &domain);
    }
}
