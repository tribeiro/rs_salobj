use avro_rs::types::Record;

use crate::{
    domain::Domain,
    sal_info::SalInfo,
    topics::{base_topic::BaseTopic, topic_info::TopicInfo},
};
use std::sync::{Arc, Mutex};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

// Default value for the ``queue_len`` constructor argument.
const DEFAULT_QUEUE_LEN: usize = 100;

// Minimum value for the ``queue_len`` constructor argument.
const MIN_QUEUE_LEN: usize = 10;

/// Base struct for reading a topic.
pub struct ReadTopic<'a> {
    /// SAL component information.
    sal_info: Rc<SalInfo>,
    /// The name of the topic.
    sal_name: String,
    /// Is this read topic open? `True` until `close` or `basic_close` is called.
    open: Arc<Mutex<bool>>,
    /// Maximum number of historical items to read when starting up.
    ///
    /// * 0 is required for commands, events, and the ackcmd topic.
    //  * 1 is recommended for telemetry. For an indexed component
    //    it is possible for data from one index to push data for another
    //    index off the DDS queue, so historical data is not guaranteed.
    //   * For the special case of reading an indexed SAL component
    //     with index=0 (read all indices) the only allowed values are 0 or 1.
    //     If 1 then retrieve the most recent sample for each index
    //     that is still in the read queue, in the order received.
    //     max_history > 1 is forbidden, because it is difficult to implement.
    max_history: usize,
    /// The maximum number of messages that can be read and not dealt with
    /// by a callback function or `next` before older messages will be dropped.
    queue_len: usize,
    data_queue: Arc<Mutex<VecDeque<avro_rs::types::Record<'a>>>>,
    current_data: Arc<Mutex<Option<Record<'a>>>>,
}

impl<'a> ReadTopic<'a> {
    pub fn new(
        sal_info: Rc<SalInfo>,
        sal_name: &str,
        max_history: usize,
        queue_len: usize,
    ) -> ReadTopic {
        sal_info.assert_is_valid_topic(sal_name);

        if sal_info.is_indexed() && sal_info.get_index() == 0 && max_history > 1 {
            panic!(
                "max_history={max_history} must be 0 or 1 for an indexed component with index=0."
            )
        }

        if queue_len < MIN_QUEUE_LEN {
            panic!("queue_len={queue_len} must be >= MIN_QUEUE_LEN={MIN_QUEUE_LEN}.");
        }

        if max_history > queue_len {
            panic!("max_history={max_history} must be <= queue_len={queue_len}.")
        }

        ReadTopic {
            sal_info: sal_info,
            sal_name: sal_name.to_owned(),
            open: Arc::new(Mutex::new(true)),
            max_history: max_history,
            queue_len: queue_len,
            data_queue: Arc::new(Mutex::new(VecDeque::with_capacity(queue_len))),
            current_data: Arc::new(Mutex::new(None)),
        }
    }

    /// Returns an owned copy of the value of the internal flag that tracks if reader is open or close.
    pub fn is_open(&self) -> bool {
        self.open.lock().unwrap().to_owned()
    }

    /// Has any data ever been seen for this topic?
    ///
    /// # Panic
    ///
    /// If sal_info was not started.
    pub fn has_data(&self) -> bool {
        self.sal_info.assert_started();

        match self.current_data.lock().unwrap().to_owned() {
            Some(_) => true,
            None => false,
        }
    }

    /// Return the number of messages in the data queue.
    pub fn get_data_queue_len(&self) -> usize {
        self.data_queue.lock().unwrap().len()
    }

    /// Get value of max history.
    pub fn get_max_history(&self) -> usize {
        self.max_history
    }

    /// A synchronous and possibly less thorough version of `close`.
    ///
    /// Intended for exit handlers and constructor error handlers.
    pub fn basic_close(&self) {
        *self.open.lock().unwrap() = false.to_owned();
    }

    /// Flush the queue used by `get_oldest` and `next`.
    ///
    /// This makes `get_oldest` return `None` and `next` wait,
    /// until a new message arrives.
    /// It does not change which message will be returned by `aget` or `get`.
    pub fn flush(&self) {
        self.data_queue.lock().unwrap().clear();
    }

    /// Get the most recent message, or `None` if no data has ever been seen
    /// (`has_data` False).
    ///
    /// This method does not change which message will be returned by `aget`,
    /// `get_oldest`, and `next`.
    pub fn get(&self) -> Option<Record> {
        self.sal_info.assert_started();

        self.current_data.lock().unwrap().to_owned()
    }

    /// Pop and return the oldest message from the queue, or `None` if the
    /// queue is empty.
    ///
    /// This is a synchronous variant of `pop_next` that does not wait for a new
    /// message. This method affects which message will be returned by `next`,
    /// but not which message will be returned by `aget` or `get`.
    pub fn pop_oldest(&self) -> Option<Record> {
        self.sal_info.assert_started();

        self.data_queue.lock().unwrap().pop_front()
    }

    /// Pop and return the oldest message from the queue, waiting for data
    /// if the queue is empty (or if flush=True). If data does not arrive in
    /// the specified `timeout` time return `None`.
    ///
    /// This method affects the data returned by `get_oldest`, but not the data
    /// returned by `aget` or `get`.
    pub async fn pop_next(&self, flush: bool, timeout: std::time::Duration) -> Option<Record> {
        self.sal_info.assert_started();
        if flush {
            self.flush();
        }

        self.wait_next(timeout).await
    }

    /// Implement waiting for new messages to arrive.
    ///
    /// If the data queue has data available, return the oldest message, if the
    /// queue is empty wait up to timeout for new data to arrive and return it.
    /// If no data is received in time, return `None`.
    async fn wait_next(&self, timeout: std::time::Duration) -> Option<Record> {
        // TODO: Finish implementation.
        self.data_queue.lock().unwrap().pop_back()
    }

    /// Add data to the back of the data queue.
    ///
    /// If the queue is full, older data will be dropped.
    fn push_back(&self, data: Record<'a>) {
        *self.current_data.lock().unwrap() = Some(data.to_owned());
        self.data_queue.lock().unwrap().push_back(data.to_owned());
    }
}

impl<'a> BaseTopic for ReadTopic<'a> {
    fn get_topic_info(&self) -> &TopicInfo {
        self.sal_info.get_topic_info(&self.sal_name).unwrap()
    }

    fn get_avro_schema(&self) -> &avro_rs::Schema {
        &self.sal_info.get_topic_schema(&self.sal_name).unwrap()
    }

    fn get_data_type(&self) -> avro_rs::types::Record {
        self.make_data_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    #[should_panic(
        expected = "max_history=2 must be 0 or 1 for an indexed component with index=0."
    )]
    fn read_topic_new_indexed_0_with_max_history() {
        let domain = Rc::new(RefCell::new(Domain::new()));
        let sal_info = SalInfo::new(domain, "Test", 0);

        ReadTopic::new(sal_info, "Test_scalars", 2, 10);
    }

    #[test]
    #[should_panic(expected = "queue_len=5 must be >= MIN_QUEUE_LEN")]
    fn read_topic_new_queue_len_too_small() {
        let domain = Rc::new(RefCell::new(Domain::new()));
        let sal_info = SalInfo::new(domain, "Test", 1);

        ReadTopic::new(sal_info, "Test_scalars", 1, 5);
    }

    #[test]
    #[should_panic(expected = "max_history=200 must be <= queue_len=100.")]
    fn read_topic_new_max_history_less_than_queue_len() {
        let domain = Rc::new(RefCell::new(Domain::new()));
        let sal_info = SalInfo::new(domain, "Test", 1);

        ReadTopic::new(sal_info, "Test_scalars", 200, 100);
    }

    #[test]
    fn read_topic_new_is_open() {
        let domain = Rc::new(RefCell::new(Domain::new()));
        let sal_info = SalInfo::new(domain, "Test", 1);

        let read_topic = ReadTopic::new(sal_info, "Test_scalars", 0, 100);

        assert!(read_topic.is_open())
    }

    #[test]
    fn basic_close() {
        let domain = Rc::new(RefCell::new(Domain::new()));
        let sal_info = SalInfo::new(domain, "Test", 1);

        let read_topic = ReadTopic::new(sal_info, "Test_scalars", 0, 100);

        read_topic.basic_close();

        assert!(!read_topic.is_open())
    }

    #[test]
    fn push_back() {
        let domain = Rc::new(RefCell::new(Domain::new()));
        let sal_info = SalInfo::new(domain, "Test", 1);

        // Need mutable instance for pushback to work
        let read_topic = ReadTopic::new(sal_info, "Test_scalars", 0, 100);

        let mut topic_sample = read_topic.get_data_type();

        topic_sample.put("boolean0", true);
        topic_sample.put("byte0", 1);
        topic_sample.put("short0", 1);
        topic_sample.put("int0", 1);
        topic_sample.put("long0", 1);
        topic_sample.put("longLong0", 1);
        topic_sample.put("unsignedShort0", 1);
        topic_sample.put("unsignedInt0", 1);
        topic_sample.put("unsignedLong0", 1);
        topic_sample.put("float0", 1.0);
        topic_sample.put("double0", 1.0);
        topic_sample.put("string0", "one".to_owned());

        read_topic.push_back(topic_sample);

        assert!(read_topic.has_data());
        assert_eq!(read_topic.get_data_queue_len(), 1);
    }

    #[test]
    fn get_no_data() {
        let domain = Rc::new(RefCell::new(Domain::new()));
        let sal_info = SalInfo::new(domain, "Test", 1);

        // Need mutable instance for pushback to work
        let read_topic = ReadTopic::new(sal_info, "Test_scalars", 0, 100);

        let data = read_topic.get();

        // No data in the queue, so should be none.
        assert!(data.is_none());
        // There's no data in the queue.
        assert!(!read_topic.has_data());
        assert_eq!(read_topic.get_data_queue_len(), 0);
    }

    #[test]
    fn get_with_data() {
        let domain = Rc::new(RefCell::new(Domain::new()));
        let sal_info = SalInfo::new(domain, "Test", 1);

        // Need mutable instance for pushback to work
        let read_topic = ReadTopic::new(sal_info, "Test_scalars", 0, 100);

        let mut topic_sample = read_topic.get_data_type();

        topic_sample.put("boolean0", true);
        topic_sample.put("byte0", 1);
        topic_sample.put("short0", 1);
        topic_sample.put("int0", 1);
        topic_sample.put("long0", 1);
        topic_sample.put("longLong0", 1);
        topic_sample.put("unsignedShort0", 1);
        topic_sample.put("unsignedInt0", 1);
        topic_sample.put("unsignedLong0", 1);
        topic_sample.put("float0", 1.0);
        topic_sample.put("double0", 1.0);
        topic_sample.put("string0", "one".to_owned());

        read_topic.push_back(topic_sample);

        let data = read_topic.get();

        // Data must be something.
        assert!(data.is_some());

        // Get does not change the data queue, so there should still be data there.
        assert_eq!(read_topic.get_data_queue_len(), 1)
    }

    #[test]
    fn pop_oldest() {
        let domain = Rc::new(RefCell::new(Domain::new()));
        let sal_info = SalInfo::new(domain, "Test", 1);

        // Need mutable instance for pushback to work
        let read_topic = ReadTopic::new(sal_info, "Test_scalars", 0, 100);

        let mut topic_sample = read_topic.get_data_type();

        topic_sample.put("boolean0", true);
        topic_sample.put("byte0", 1);
        topic_sample.put("short0", 1);
        topic_sample.put("int0", 1);
        topic_sample.put("long0", 1);
        topic_sample.put("longLong0", 1);
        topic_sample.put("unsignedShort0", 1);
        topic_sample.put("unsignedInt0", 1);
        topic_sample.put("unsignedLong0", 1);
        topic_sample.put("float0", 1.0);
        topic_sample.put("double0", 1.0);
        topic_sample.put("string0", "one".to_owned());

        read_topic.push_back(topic_sample);

        let data = read_topic.pop_oldest();

        // Data should be something.
        assert!(data.is_some());
        // There's still data
        assert!(read_topic.has_data());
        // But queue should be empty now.
        assert_eq!(read_topic.get_data_queue_len(), 0);
    }
}
