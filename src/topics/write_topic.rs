use apache_avro::types::{Record, Value};

use crate::{
    base_topic,
    domain::Domain,
    sal_info::SalInfo,
    topics::{base_topic::BaseTopic, topic_info::TopicInfo},
};
use chrono::Utc;
use kafka::producer;
use schema_registry_converter::async_impl::avro::AvroEncoder;
use schema_registry_converter::schema_registry_common::SubjectNameStrategy;

/// Maximum value for the ``private_seqNum`` field of each topic,
/// a 4 byte signed integer.
/// For command topics this field is the command ID, and it must be unique
/// for each command in order to avoid collisions (since there is only one
/// ``ackcmd`` topic that is shared by all commands).
/// For other topics its use is unspecified but it may prove handy to
/// increment it (with wraparound) for each data point.
const MAX_SEQ_NUM: i64 = i64::MAX;

/// Base struct for writing a topic.
pub struct WriteTopic<'a> {
    /// Domain information
    domain: Domain,
    /// SAL component information.
    sal_info: &'a SalInfo,
    /// The name of the topic.
    topic_name: String,
    /// Is this instance open? `True` until `close` or `basic_close` is called.
    data_changed: bool,
    data: Option<Record<'a>>,
    producer: Option<producer::Producer>,
}

base_topic!(WriteTopic);

impl<'a> WriteTopic<'a> {
    pub fn new(domain: Domain, sal_info: &'a SalInfo, topic_name: &str) -> WriteTopic<'a> {
        sal_info.assert_is_valid_topic(topic_name);

        WriteTopic {
            domain: domain,
            sal_info: sal_info,
            topic_name: topic_name.to_owned(),
            data_changed: false,
            data: None,
            producer: None,
        }
    }

    /// Returns an owned copy of the value of the internal flag that tracks if
    /// writer is open or close.
    pub fn is_open(&self) -> bool {
        self.producer.is_some()
    }

    /// Has `data` ever been set?
    ///
    /// A value of true means at least one field has been set, not that all
    /// fields have been set or data has actually been sent.
    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    pub fn set_producer(&mut self, producer: producer::Producer) {
        self.producer = Some(producer);
    }

    /// A synchronous and possibly less thorough version of `close`.
    ///
    /// Intended for exit handlers and constructor error handlers.
    pub fn basic_close(&mut self) {
        if self.is_open() {
            self.producer = None;
        }
    }

    /// Shut down and release resources.
    ///
    /// Intended to be called by SalInfo.close(), since that tracks all topics.
    pub async fn close(&mut self) {
        self.basic_close();
    }

    /// Set data but do not write it.
    pub fn set(&mut self, data: Record<'a>) -> bool {
        // Need to check if data changed or not and return false if not
        self.data = Some(data.clone());
        self.data_changed = true;
        true
    }

    /// Write the data.
    ///
    /// # Notes
    ///
    /// Originally the `private_sndStamp` has to be tai but this is writing it
    /// as utc. The precision is going to be microseconds.
    pub async fn write(&mut self, encoder: &AvroEncoder<'a>) -> bool {
        if !self.is_open() {
            return false;
        }
        match self.data.clone() {
            Some(mut data) => {
                // read current time in microseconds, as int, convert to f32 then
                // convert to seconds.
                data.put(
                    "private_sndStamp",
                    Value::Double(Utc::now().timestamp_micros() as f64 * 1e-6),
                );
                data.put(
                    "private_origin",
                    Value::Long(self.domain.get_origin().try_into().unwrap()),
                );
                data.put(
                    "private_identity",
                    Value::String(self.domain.get_identity()),
                );
                data.put(
                    "private_seqNum",
                    Value::Long(0), // FIXME: This is supposed to be an increasing number
                );
                data.put("private_rcvStamp", Value::Double(0.0));

                if self.sal_info.is_indexed() {
                    data.put(
                        "salIndex",
                        Value::Long(self.sal_info.get_index().try_into().unwrap()),
                    );
                }

                for (key, value) in &data.fields {
                    match value {
                        Value::Null => println!("Attribute {key} not set."),
                        _ => continue,
                    }
                }
                self.data_changed = false;
                let topic_name = self.get_topic_publish_name();
                let record_type = self.get_record_type();

                let key_strategy =
                    SubjectNameStrategy::TopicRecordNameStrategy(topic_name.clone(), record_type);
                let data_fields: Vec<(&str, Value)> =
                    data.fields.iter().map(|(k, v)| (&**k, v.clone())).collect();

                let bytes = encoder.encode(data_fields, key_strategy).await.unwrap();

                if let Some(producer) = &mut self.producer {
                    producer
                        .send(&producer::Record::from_value(&topic_name, bytes))
                        .unwrap();
                }

                true
            }
            None => false,
        }
    }

    /// Set and write data.
    ///
    /// Data resets after sent.
    pub async fn set_write(&mut self, data: Record<'a>, encoder: &AvroEncoder<'a>) {
        self.set(data);
        self.write(encoder).await;
        self.data_changed = true;
        self.data = None;
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::domain::Domain;
    use apache_avro::{types::Value, Writer};

    #[test]
    #[should_panic]
    fn new_with_bad_topic_name() {
        let domain = Domain::new();
        let sal_info = SalInfo::new("Test", 1);

        WriteTopic::new(domain, &sal_info, "inexistentTopic");
    }

    #[test]
    fn new() {
        let domain = Domain::new();
        let sal_info = SalInfo::new("Test", 1);

        let topic_writer = WriteTopic::new(domain, &sal_info, "scalars");

        assert!(!topic_writer.is_open());
        assert!(!topic_writer.has_data());
    }

    #[test]
    fn set_scalars() {
        let domain = Domain::new();
        let sal_info = SalInfo::new("Test", 1);

        let mut topic_writer = WriteTopic::new(domain, &sal_info, "scalars");

        let schema = WriteTopic::get_avro_schema(&sal_info, &topic_writer.get_sal_name());

        let mut data = WriteTopic::make_data_type(&schema);

        data.put(
            "private_sndStamp",
            Value::Double(Utc::now().timestamp_micros() as f64 * 1e-6),
        );
        data.put("private_origin", Value::Long(101));
        data.put("private_identity", Value::String("myself".to_owned()));
        data.put("private_seqNum", Value::Long(0));
        data.put("private_rcvStamp", Value::Double(0.0));
        data.put(
            "salIndex",
            Value::Long(sal_info.get_index().try_into().unwrap()),
        );

        data.put("boolean0", Value::Boolean(true));
        data.put("byte0", Value::Bytes(vec![1, 2, 3]));
        data.put("short0", Value::Int(1));
        data.put("int0", Value::Int(1));
        data.put("long0", Value::Long(1));
        data.put("longLong0", Value::Long(1));
        data.put("unsignedShort0", Value::Int(1));
        data.put("unsignedInt0", Value::Int(1));
        data.put("unsignedLong0", Value::Long(1));
        data.put("float0", Value::Float(1.0));
        data.put("double0", Value::Double(1.0));
        data.put("string0", Value::String("This is a test!".to_owned()));

        let mut writer = Writer::new(&schema, Vec::new());

        // Schema validation...
        match writer.append(data) {
            Ok(_) => println!("Data passes schema validation!"),
            Err(error) => {
                println!("{:?}", error);
                panic!("Error!");
            }
        }
    }

    #[test]
    fn set_arrays() {
        let domain = Domain::new();
        let sal_info = SalInfo::new("Test", 1);

        let mut topic_writer = WriteTopic::new(domain, &sal_info, "arrays");

        let schema = WriteTopic::get_avro_schema(&sal_info, &topic_writer.get_sal_name());

        let mut data = WriteTopic::make_data_type(&schema);

        data.put(
            "boolean0",
            Value::Array(vec![
                Value::Boolean(true),
                Value::Boolean(true),
                Value::Boolean(true),
                Value::Boolean(true),
                Value::Boolean(true),
            ]),
        );
        data.put(
            "int0",
            Value::Array(vec![
                Value::Int(1),
                Value::Int(1),
                Value::Int(1),
                Value::Int(1),
                Value::Int(1),
            ]),
        );
        data.put(
            "float0",
            Value::Array(vec![
                Value::Float(1.0),
                Value::Float(1.0),
                Value::Float(1.0),
                Value::Float(1.0),
                Value::Float(1.0),
            ]),
        );
        data.put(
            "string0",
            Value::Array(vec![
                Value::String("This is a test!".to_owned()),
                Value::String("This is a test!".to_owned()),
                Value::String("This is a test!".to_owned()),
                Value::String("This is a test!".to_owned()),
                Value::String("This is a test!".to_owned()),
            ]),
        );

        assert!(topic_writer.set(data))
    }
}
