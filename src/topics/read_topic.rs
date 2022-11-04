use apache_avro::types::Record;

use crate::{
    sal_info::SalInfo,
    topics::{base_topic::BaseTopic, topic_info::TopicInfo},
};

// Default value for the ``queue_len`` constructor argument.
const DEFAULT_QUEUE_LEN: usize = 100;

// Minimum value for the ``queue_len`` constructor argument.
const MIN_QUEUE_LEN: usize = 10;

/// Base struct for reading a topic.
pub struct ReadTopic<'a> {
    /// SAL component information.
    sal_info: &'a SalInfo,
    /// The name of the topic.
    sal_name: String,
    /// Is this read topic open? `True` until `close` or `basic_close` is called.
    open: bool,
    flushed: bool,
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
    current_data: Option<Record<'a>>,
}

impl<'a> ReadTopic<'a> {
    pub fn new(sal_info: &'a SalInfo, sal_name: &str, max_history: usize) -> ReadTopic<'a> {
        sal_info.assert_is_valid_topic(sal_name);

        if sal_info.is_indexed() && sal_info.get_index() == 0 && max_history > 1 {
            panic!(
                "max_history={max_history} must be 0 or 1 for an indexed component with index=0."
            )
        }

        ReadTopic {
            sal_info: sal_info,
            sal_name: sal_name.to_owned(),
            open: false,
            flushed: false,
            max_history: max_history,
            current_data: None,
        }
    }

    pub fn get_sal_name(&self) -> String {
        self.sal_name.to_owned()
    }

    pub fn get_max_history(&self) -> usize {
        self.max_history
    }

    /// Returns an owned copy of the value of the internal flag that tracks if
    /// reader is open or close.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Has any data ever been seen for this topic?
    ///
    /// # Panic
    ///
    /// If sal_info was not started.
    pub fn has_data(&self) -> bool {
        self.current_data.is_some()
    }

    /// A synchronous and possibly less thorough version of `close`.
    ///
    /// Intended for exit handlers and constructor error handlers.
    pub fn basic_close(&mut self) {
        self.open = false;
    }

    /// Flush the queue used by `get_oldest` and `next`.
    ///
    /// This makes `get_oldest` return `None` and `next` wait,
    /// until a new message arrives.
    /// It does not change which message will be returned by `aget` or `get`.
    pub fn flush(&mut self) {
        self.flushed = true;
    }

    /// Get the most recent message, or `None` if no data has ever been seen
    /// (`has_data` False).
    ///
    /// This method does not change which message will be returned by `aget`,
    /// `get_oldest`, and `next`.
    pub fn get(&self) -> Option<Record> {
        self.current_data.to_owned()
    }

    /// Pop and return the oldest message from the queue, or `None` if the
    /// queue is empty.
    ///
    /// This is a synchronous variant of `pop_next` that does not wait for a new
    /// message. This method affects which message will be returned by `next`,
    /// but not which message will be returned by `aget` or `get`.
    pub fn pop_oldest(&self) -> Option<Record> {
        self.current_data.to_owned()
    }

    /// Pop and return the oldest message from the queue, waiting for data
    /// if the queue is empty (or if flush=True). If data does not arrive in
    /// the specified `timeout` time return `None`.
    ///
    /// This method affects the data returned by `get_oldest`, but not the data
    /// returned by `aget` or `get`.
    pub async fn pop_next(&mut self, flush: bool, timeout: std::time::Duration) -> Option<Record> {
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
    async fn wait_next(&mut self, timeout: std::time::Duration) -> Option<Record> {
        // TODO: Finish implementation.
        if self.flushed {
            // TODO: Get the most recent data that was published after flush
            // was called.
            return None;
        } else {
            // TODO: Get the next data in the queue
            return None;
        }
    }
}

impl<'a> BaseTopic for ReadTopic<'a> {
    fn get_topic_info(&self) -> &TopicInfo {
        self.sal_info.get_topic_info(&self.sal_name).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(
        expected = "max_history=2 must be 0 or 1 for an indexed component with index=0."
    )]
    fn read_topic_new_indexed_0_with_max_history() {
        let sal_info = SalInfo::new("Test", 0);

        ReadTopic::new(&sal_info, "scalars", 2);
    }

    #[test]
    fn read_topic_new_is_open() {
        let sal_info = SalInfo::new("Test", 1);

        let read_topic = ReadTopic::new(&sal_info, "scalars", 0);

        assert!(!read_topic.is_open())
    }

    #[test]
    fn basic_close() {
        let sal_info = SalInfo::new("Test", 1);

        let mut read_topic = ReadTopic::new(&sal_info, "scalars", 0);

        read_topic.basic_close();

        assert!(!read_topic.is_open())
    }

    #[test]
    fn get_no_data() {
        let sal_info = SalInfo::new("Test", 1);

        // Need mutable instance for pushback to work
        let read_topic = ReadTopic::new(&sal_info, "scalars", 0);

        let data = read_topic.get();

        // No data in the queue, so should be none.
        assert!(data.is_none());
        // There's no data in the queue.
        assert!(!read_topic.has_data());
    }
}
