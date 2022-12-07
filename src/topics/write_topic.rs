use apache_avro::types::{Record, Value};

use crate::{domain::Domain, sal_info::SalInfo, topics::base_topic::BaseTopic};
use chrono::Utc;
use kafka::producer;
use schema_registry_converter::schema_registry_common::SubjectNameStrategy;
use std::time::Duration;

/// Maximum value for the ``private_seqNum`` field of each topic,
/// a 4 byte signed integer.
/// For command topics this field is the command ID, and it must be unique
/// for each command in order to avoid collisions (since there is only one
/// ``ackcmd`` topic that is shared by all commands).
/// For other topics its use is unspecified but it may prove handy to
/// increment it (with wraparound) for each data point.
const MAX_SEQ_NUM: i64 = i64::MAX;

/// Base struct for writing a topic.
pub struct WriteTopic {
    /// The name of the topic.
    topic_name: String,
    /// Index of the CSC, must be 0 if CSC is not indexed.
    index: i64,
    /// Is the CSC indexed?
    indexed: bool,
    /// An identifier of the current process.
    origin: i64,
    /// A string identifying the instance.
    identity: String,
    /// Data producer.
    producer: producer::Producer,
    /// Sequence number of the written samples. This number is incremented
    /// every time a sample is published.
    seq_num: i64,
}

impl BaseTopic for WriteTopic {}

impl WriteTopic {
    pub fn new(topic_name: &str, sal_info: &SalInfo, domain: &Domain) -> WriteTopic {
        sal_info.assert_is_valid_topic(topic_name);

        WriteTopic {
            topic_name: topic_name.to_owned(),
            index: sal_info.get_index() as i64,
            indexed: sal_info.is_indexed(),
            origin: domain.get_origin() as i64,
            identity: domain.get_identity().to_owned(),
            producer: producer::Producer::from_hosts(vec!["localhost:9092".to_owned()])
                .with_ack_timeout(Duration::from_secs(1))
                .with_required_acks(producer::RequiredAcks::One)
                .create()
                .unwrap(),
            seq_num: 1,
        }
    }

    /// Get value of the origin identifier.
    ///
    /// This identifies the process running the current application.
    pub fn get_origin(&self) -> i64 {
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
    pub fn get_index(&self) -> i64 {
        self.index
    }

    /// Get topic name.
    pub fn get_topic_name(&self) -> String {
        self.topic_name.to_owned()
    }

    /// Get current sequence number.
    pub fn get_seq_num(&self) -> i64 {
        self.seq_num
    }

    /// Write the data.
    ///
    /// # Notes
    ///
    /// Originally the `private_sndStamp` has to be tai but this is writing it
    /// as utc. The precision is going to be microseconds.
    pub async fn write<'r, 'si>(&mut self, data: &mut Record<'r>, sal_info: &SalInfo<'si>) -> i64 {
        // read current time in microseconds, as int, convert to f32 then
        // convert to seconds.
        self.seq_num += 1;
        data.put(
            "private_sndStamp",
            Value::Double(Utc::now().timestamp_micros() as f64 * 1e-6),
        );
        data.put("private_origin", Value::Long(self.get_origin()));
        data.put("private_identity", Value::String(self.get_identity()));
        data.put(
            "private_seqNum",
            Value::Long(self.seq_num), // FIXME: This is supposed to be an increasing number
        );
        data.put("private_rcvStamp", Value::Double(0.0));

        if self.is_indexed() {
            data.put("salIndex", Value::Long(self.get_index()));
        }

        for (key, value) in &data.fields {
            match value {
                Value::Null => println!("Attribute {key} not set."),
                _ => continue,
            }
        }
        let topic_name = sal_info.make_topic_name(&self.get_topic_name());
        let record_type = self.get_record_type();

        let key_strategy =
            SubjectNameStrategy::TopicRecordNameStrategy(topic_name.clone(), record_type);
        let data_fields: Vec<(&str, Value)> =
            data.fields.iter().map(|(k, v)| (&**k, v.clone())).collect();

        let bytes = sal_info.encode(data_fields, key_strategy).await.unwrap();

        self.producer
            .send(&producer::Record::from_value(&topic_name, bytes))
            .unwrap();

        self.seq_num
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::domain::Domain;

    #[test]
    fn test_basics() {
        let domain = Domain::new();
        let sal_info = SalInfo::new("Test", 1);
        let write_topic = WriteTopic::new("scalars", &sal_info, &domain);

        assert_eq!(write_topic.is_indexed(), true);
        assert_eq!(write_topic.get_index(), 1);
        assert_eq!(write_topic.get_topic_name(), "scalars");
    }

    #[test]
    #[should_panic]
    fn new_with_bad_topic_name() {
        let domain = Domain::new();
        let sal_info = SalInfo::new("Test", 1);

        WriteTopic::new("inexistentTopic", &sal_info, &domain);
    }
}
