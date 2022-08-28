use avro_rs::types::{Record, Value};

use crate::{
    sal_info::SalInfo,
    topics::{base_topic::BaseTopic, topic_info::TopicInfo},
};
use chrono::Utc;
use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

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
    /// SAL component information.
    sal_info: Rc<SalInfo>,
    /// The name of the topic.
    sal_name: String,
    /// Is this instance open? `True` until `close` or `basic_close` is called.
    open: Arc<Mutex<bool>>,
    data: Arc<Mutex<Option<Record<'a>>>>,
}

impl<'a> BaseTopic for WriteTopic<'a> {
    fn get_topic_info(&self) -> &TopicInfo {
        self.sal_info.get_topic_info(&self.sal_name).unwrap()
    }

    fn get_avro_schema(&self) -> &avro_rs::Schema {
        &self.sal_info.get_topic_schema(&self.sal_name).unwrap()
    }

    fn get_data_type(&self) -> Record {
        self.make_data_type()
    }
}

impl<'a> WriteTopic<'a> {
    pub fn new(sal_info: Rc<SalInfo>, sal_name: &str) -> WriteTopic {
        sal_info.assert_is_valid_topic(sal_name);

        WriteTopic {
            sal_info: sal_info,
            sal_name: sal_name.to_owned(),
            open: Arc::new(Mutex::new(true)),
            data: Arc::new(Mutex::new(None)),
        }
    }

    /// Returns an owned copy of the value of the internal flag that tracks if
    /// writer is open or close.
    pub fn is_open(&self) -> bool {
        self.open.lock().unwrap().to_owned()
    }

    /// Has `data` ever been set?
    ///
    /// A value of true means at least one field has been set, not that all
    /// fields have been set or data has actually been sent.
    pub fn has_data(&self) -> bool {
        self.sal_info.assert_started();

        self.data.lock().unwrap().is_some()
    }

    /// A synchronous and possibly less thorough version of `close`.
    ///
    /// Intended for exit handlers and constructor error handlers.
    pub fn basic_close(&self) {
        if self.is_open() {
            *self.open.lock().unwrap() = false;
        }
    }

    /// Shut down and release resources.
    ///
    /// Intended to be called by SalInfo.close(), since that tracks all topics.
    pub async fn close(&self) {
        self.basic_close();
    }

    /// Set data but do not write it.
    pub fn set(&self, data: Record<'a>) -> bool {
        // Need to check if data changed or not and return false if not
        *self.data.lock().unwrap() = Some(data.clone());
        true
    }

    /// Write the data.
    ///
    /// # Notes
    ///
    /// Originally the `private_sndStamp` has to be tai but this is writting it
    /// as utc. The precision is going to be microseconds.
    pub async fn write(&self) -> Record {
        self.sal_info.assert_started();
        let mut data = self.make_data_type();

        let field_values = self.data.lock().unwrap().as_ref().unwrap().clone().fields;

        for (field, value) in field_values {
            data.put(&field, value);
        }

        // read current time in microseconds, as int, convert to f32 then
        // convert to seconds.
        data.put(
            "private_sndStamp",
            Value::Float(Utc::now().timestamp_micros() as f32 * 1e-6),
        );
        data.put(
            "private_origin",
            Value::Int(self.sal_info.get_origin().try_into().unwrap()),
        );
        data.put(
            "private_identity",
            Value::String(self.sal_info.get_identity()),
        );
        data.put(
            "private_seqNum",
            Value::Int(0), // FIXME: This is supposed to be an increasing number
        );

        if self.sal_info.is_indexed() {
            data.put(
                "salIndex",
                Value::Int(self.sal_info.get_index().try_into().unwrap()),
            );
        }
        self.sal_info.write_data(&data).await;
        data
    }

    /// Set and write data.
    ///
    /// Data resets after sent.
    pub async fn set_write(&self, data: Record<'a>) {
        self.set(data);
        self.write().await;
        *self.data.lock().unwrap() = None;
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::domain::Domain;
    use avro_rs::types::Value;

    #[test]
    #[should_panic]
    fn new_with_bad_topic_name() {
        let domain = Domain::new_rc();
        let sal_info = SalInfo::new(domain, "Test", 1);

        WriteTopic::new(sal_info, "Test_inexistentTopic");
    }

    #[test]
    fn new() {
        let domain = Domain::new_rc();
        let sal_info = SalInfo::new(domain, "Test", 1);

        let topic_writer = WriteTopic::new(sal_info, "Test_scalars");

        assert!(topic_writer.is_open());
        assert!(!topic_writer.has_data());
    }

    #[test]
    fn set_scalars() {
        let domain = Domain::new_rc();
        let sal_info = SalInfo::new(domain, "Test", 1);

        let topic_writer = WriteTopic::new(sal_info, "Test_scalars");

        let mut data = topic_writer.make_data_type();

        data.put("boolean0", Value::Boolean(true));
        data.put("int0", Value::Int(1));
        data.put("float0", Value::Float(1.0));
        data.put("string0", Value::String("This is a test!".to_owned()));

        assert!(topic_writer.set(data))
    }

    #[test]
    fn set_arrays() {
        let domain = Domain::new_rc();
        let sal_info = SalInfo::new(domain, "Test", 1);

        let topic_writer = WriteTopic::new(sal_info, "Test_arrays");

        let mut data = topic_writer.make_data_type();

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
