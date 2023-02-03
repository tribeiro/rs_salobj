use crate::{domain::Domain, sal_info::SalInfo, topics::base_topic::BaseTopic};
use apache_avro::types::Value;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use std::collections::VecDeque;
use tokio::time::sleep;

// Default value for the ``queue_len`` constructor argument.
const DEFAULT_QUEUE_LEN: usize = 100;

// Minimum value for the ``queue_len`` constructor argument.
const MIN_QUEUE_LEN: usize = 10;

const POOL_WAIT_TIME: std::time::Duration = std::time::Duration::from_millis(10);

/// Base struct for reading a topic.
pub struct ReadTopic {
    /// The name of the topic.
    topic_name: String,
    /// The name of the topic in the data cloud.
    topic_publish_name: String,
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
    /// Sample of the last data seen.
    current_data: Option<Value>,
    /// Data queue.
    data_queue: VecDeque<Value>,
    /// Topic consumer.
    consumer: Consumer,
}

impl BaseTopic for ReadTopic {}

impl ReadTopic {
    pub fn new(
        topic_name: &str,
        sal_info: &SalInfo,
        domain: &Domain,
        max_history: usize,
    ) -> ReadTopic {
        sal_info.assert_is_valid_topic(topic_name);

        if sal_info.is_indexed() && sal_info.get_index() == 0 && max_history > 1 {
            panic!(
                "max_history={max_history} must be 0 or 1 for an indexed component with index=0."
            )
        }

        let fetch_offset = if max_history > 0 {
            FetchOffset::Earliest
        } else {
            FetchOffset::Latest
        };

        ReadTopic {
            topic_name: topic_name.to_owned(),
            topic_publish_name: sal_info.make_topic_name(topic_name).to_owned(),
            max_history: max_history,
            data_queue: VecDeque::with_capacity(DEFAULT_QUEUE_LEN),
            consumer: Consumer::from_hosts(vec!["localhost:9092".to_owned()])
                .with_topic(sal_info.make_topic_name(topic_name))
                .with_fallback_offset(fetch_offset)
                .with_group(format!("{}", domain.get_origin()))
                .with_offset_storage(GroupOffsetStorage::Kafka)
                .create()
                .unwrap(),
            current_data: None,
        }
    }

    /// Get the name of the topic.
    pub fn get_topic_name(&self) -> String {
        self.topic_name.to_owned()
    }

    /// Get the name of the topic used to publish data.
    pub fn get_topic_publish_name(&self) -> String {
        self.topic_publish_name.to_owned()
    }

    /// Get the size of the data queue.
    pub fn get_max_history(&self) -> usize {
        self.max_history
    }

    /// Has any data ever been seen for this topic?
    ///
    /// # Panic
    ///
    /// If sal_info was not started.
    pub fn has_data(&self) -> bool {
        self.current_data.is_some()
    }

    /// Flush the queue used by `get_oldest` and `next`.
    ///
    /// This makes `get_oldest` return `None` and `next` wait,
    /// until a new message arrives.
    /// It does not change which message will be returned by `aget` or `get`.
    pub fn flush(&mut self) {
        self.data_queue.clear();
    }

    /// Get the most recent message, or `None` if no data has ever been seen
    /// (`has_data` False).
    ///
    /// This method does not change which message will be returned by `aget`,
    /// `get_oldest`, and `next`.
    pub fn get(&self) -> Option<Value> {
        self.current_data.to_owned()
    }

    /// Pop and return the newest message from the queue, or `None` if the
    /// queue is empty.
    ///
    /// This is a synchronous variant of `pop_next` that does not wait for a new
    /// message. This method affects which message will be returned by `next`,
    /// but not which message will be returned by `aget` or `get`.
    pub async fn pop_back<'b>(
        &mut self,
        flush: bool,
        timeout: std::time::Duration,
        sal_info: &SalInfo<'b>,
    ) -> Option<Value> {
        if flush {
            self.flush();
        }
        self.pool(timeout, sal_info).await;
        self.data_queue.pop_back()
    }

    /// Pop and return the oldest message from the queue, waiting for data
    /// if the queue is empty (or if flush=True). If data does not arrive in
    /// the specified `timeout` time return `None`.
    ///
    /// This method affects the data returned by `get_oldest`, but not the data
    /// returned by `aget` or `get`.
    pub async fn pop_front<'b>(
        &mut self,
        flush: bool,
        timeout: std::time::Duration,
        sal_info: &SalInfo<'b>,
    ) -> Option<Value> {
        if flush {
            self.flush();
        }
        if self.data_queue.is_empty() {
            self.pool(timeout, sal_info).await;
        }
        self.data_queue.pop_front()
    }

    /// Pool for new data until there are no more data to pool.
    ///
    /// The data is pushed to a dequeue with limited size so calling this
    /// method may cause older data to be dropped.
    ///
    /// This current implementation will simply pool for all the data, so if
    /// the backlog is large it may take some tike to finish. A future
    /// implementation will check the message offset and reset it to the head
    /// of the data queue, avoiding unnecessary reads.
    async fn pool<'b>(&mut self, timeout: std::time::Duration, sal_info: &SalInfo<'b>) -> bool {
        let timer_task = tokio::spawn(async move {
            sleep(timeout).await;
        });

        let mut got_any_data = false;

        while !timer_task.is_finished() {
            let mut got_data = false;
            let messages = self.consumer.poll().unwrap();
            for ms in messages.iter() {
                for m in ms.messages() {
                    let data = sal_info.decode(Some(m.value)).await.unwrap().value;
                    self.current_data = Some(data.clone());
                    self.data_queue.push_back(data);
                }
                got_any_data = true;
                got_data = true;
                self.consumer.consume_messageset(ms).unwrap();
            }
            self.consumer.commit_consumed().unwrap();
            if !got_data && got_any_data {
                timer_task.abort();
                return got_any_data;
            }
            sleep(POOL_WAIT_TIME).await;
        }
        return false;
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
        let domain = Domain::new();
        let sal_info = SalInfo::new("Test", 0);

        ReadTopic::new("scalars", &sal_info, &domain, 2);
    }

    #[tokio::test]
    async fn get_no_data() {
        let mut domain = Domain::new();
        let sal_info = SalInfo::new("Test", 1);

        let topics: Vec<String> = sal_info
            .get_telemetry_names()
            .into_iter()
            .map(|topic_name| sal_info.make_topic_name(&topic_name).to_owned())
            .collect();

        println!("Loading metadata for topics: {topics:?}");
        domain.register_topics(&topics).unwrap();
        sal_info.register_schema().await;

        let read_topic = ReadTopic::new("scalars", &sal_info, &domain, 0);

        let data = read_topic.get();

        // No data in the queue, so should be none.
        assert!(data.is_none());
        // There's no data in the queue.
        assert!(!read_topic.has_data());
    }
}
