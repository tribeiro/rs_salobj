use std::time::{Duration, Instant};

use crate::{
    domain::Domain,
    error::errors::{SalObjError, SalObjResult},
    sal_info::SalInfo,
    topics::base_topic::BaseTopic,
};
use apache_avro::types::Value;
use kafka::{
    consumer::{Consumer, FetchOffset, GroupOffsetStorage},
    error::Result as KafkaResult,
};
use schema_registry_converter::async_impl::avro::AvroDecoder;
use std::collections::VecDeque;
use tokio::time::sleep;

// Default value for the ``queue_len`` constructor argument.
const DEFAULT_QUEUE_LEN: usize = 100;

const POOL_WAIT_TIME: std::time::Duration = std::time::Duration::from_micros(50);

/// Base struct for reading a topic.
pub struct ReadTopic<'a> {
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
    consumer: KafkaResult<Consumer>,
    decoder: AvroDecoder<'a>,
    sal_index: Option<i32>,
}

impl<'a> BaseTopic for ReadTopic<'a> {}

impl<'a> ReadTopic<'a> {
    pub fn new(
        topic_name: &str,
        sal_info: &SalInfo,
        domain: &Domain,
        max_history: usize,
    ) -> ReadTopic<'a> {
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

        let sal_index = sal_info.get_optional_index();

        ReadTopic {
            topic_name: topic_name.to_owned(),
            topic_publish_name: sal_info.make_schema_registry_topic_name(topic_name),
            max_history,
            data_queue: VecDeque::with_capacity(DEFAULT_QUEUE_LEN),
            consumer: Consumer::from_hosts(Domain::get_client_hosts())
                .with_topic(sal_info.make_schema_registry_topic_name(topic_name))
                .with_fallback_offset(fetch_offset)
                .with_group(format!("{}", domain.get_origin()))
                .with_fetch_max_wait_time(Duration::from_millis(1))
                .with_offset_storage(GroupOffsetStorage::Kafka)
                .create(),
            current_data: None,
            decoder: SalInfo::make_decoder(),
            sal_index,
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
    pub async fn pop_back(&mut self, flush: bool, timeout: std::time::Duration) -> Option<Value> {
        if flush {
            self.flush();
        }
        let start = Instant::now();
        match self.pool(timeout).await {
            Ok(n_messages) => {
                let duration = start.elapsed();
                log::trace!(
                    "pop_back {} took {duration:?} to pool data. Got {n_messages} messages.",
                    self.topic_name
                );
            }
            Err(error) => {
                log::warn!("Error pooling new data: {error}.");
            }
        }
        self.data_queue.pop_back()
    }

    /// Pop and return the oldest message from the queue, waiting for data
    /// if the queue is empty (or if flush=True). If data does not arrive in
    /// the specified `timeout` time return `None`.
    ///
    /// This method affects the data returned by `get_oldest`, but not the data
    /// returned by `aget` or `get`.
    pub async fn pop_front(&mut self, flush: bool, timeout: std::time::Duration) -> Option<Value> {
        if flush {
            self.flush();
        }
        if self.data_queue.is_empty() {
            let start = Instant::now();
            match self.pool(timeout).await {
                Ok(n_messages) => {
                    let duration = start.elapsed();
                    log::trace!(
                        "pop_front {} took {duration:?} to pool data. Got {n_messages} messages.",
                        self.topic_name
                    );
                }
                Err(error) => {
                    log::warn!("Error pooling new data: {error}.");
                }
            }
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
    async fn pool(&mut self, timeout: std::time::Duration) -> SalObjResult<usize> {
        match &mut self.consumer {
            Ok(consumer) => {
                let timer_task = tokio::spawn(async move {
                    sleep(timeout).await;
                });

                let mut n_messages = 0;

                while !timer_task.is_finished() {
                    let start = Instant::now();
                    match consumer.poll() {
                        Ok(messages) => {
                            let duration = start.elapsed();

                            let no_data = messages.is_empty();
                            log::trace!(
                                "pool {} took {duration:?} to consume data, is empty? {no_data}.",
                                self.topic_name
                            );
                            for ms in messages.iter() {
                                for m in ms.messages() {
                                    let start = Instant::now();
                                    match self.decoder.decode(Some(m.value)).await {
                                        Ok(data) => {
                                            let duration = start.elapsed();
                                            log::trace!(
                                                "pool {} took {duration:?} to decode data.",
                                                self.topic_name
                                            );
                                            let data_value = data.value;
                                            if !ReadTopic::same_index(&self.sal_index, &data_value)
                                            {
                                                continue;
                                            }
                                            self.current_data = Some(data_value.clone());
                                            self.data_queue.push_back(data_value);
                                            n_messages += 1;
                                        }
                                        Err(error) => return Err(SalObjError::from_error(error)),
                                    };
                                }
                                if let Err(error) = consumer.consume_messageset(ms) {
                                    return Err(SalObjError::from_error(error));
                                }
                            }
                            if n_messages > 0 && no_data {
                                timer_task.abort();
                                return Ok(n_messages);
                            }
                            sleep(POOL_WAIT_TIME).await;
                        }
                        Err(error) => {
                            return Err(SalObjError::from_error(error));
                        }
                    }
                }
                Ok(0)
            }
            Err(error) => Err(SalObjError::new(&error.to_string())),
        }
    }

    fn same_index(sal_index: &Option<i32>, data_value: &Value) -> bool {
        if let Some(sal_index) = sal_index {
            if let Value::Record(data_record) = &data_value {
                let data_sal_index: Vec<&Value> = data_record
                    .iter()
                    .filter_map(|(field, value)| {
                        if field == "salIndex" {
                            Some(value)
                        } else {
                            None
                        }
                    })
                    .collect();
                if let Some(Value::Int(data_sal_index)) = data_sal_index.get(0) {
                    sal_index == data_sal_index
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
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
        let sal_info = SalInfo::new("Test", 0).unwrap();

        ReadTopic::new("scalars", &sal_info, &domain, 2);
    }

    #[tokio::test]
    async fn get_no_data() {
        let mut domain = Domain::new();
        let sal_info = SalInfo::new("Test", 1).unwrap();

        let topics: Vec<String> = sal_info
            .get_telemetry_names()
            .into_iter()
            .map(|topic_name| {
                sal_info
                    .make_schema_registry_topic_name(&topic_name)
                    .to_owned()
            })
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
