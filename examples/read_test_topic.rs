use apache_avro::types::Value;
use kafka::{
    client::KafkaClient,
    consumer::{Consumer, FetchOffset, GroupOffsetStorage},
};
use std::collections::HashMap;

use salobj::{
    domain,
    sal_info::SalInfo,
    topics::{base_topic::BaseTopic, read_topic::ReadTopic},
};
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() {
    let mut client = KafkaClient::new(vec!["localhost:9092".to_owned()]);
    client.set_client_id("kafka-rust-console-producer".into());
    client.load_metadata_all().unwrap();

    let domain = domain::Domain::new();
    let component = "Test";
    let topic = "logevent_heartbeat";

    client.load_metadata(&[topic]);

    assert!(client.topics().contains(&topic));

    let sal_info = SalInfo::new(component, 1);
    let max_history: usize = 10;

    let mut topic_reader = ReadTopic::new(&sal_info, topic, max_history);

    println!(
        "Reading topic: {} group: {}",
        topic_reader.get_topic_publish_name(),
        topic_reader.get_record_type()
    );

    let consumer = Consumer::from_hosts(vec!["localhost:9092".to_owned()])
        .with_topic(topic_reader.get_topic_publish_name())
        .with_fallback_offset(FetchOffset::Latest)
        .with_group(format!("{}", domain.get_origin()))
        .with_offset_storage(GroupOffsetStorage::Kafka)
        .create()
        .unwrap();

    topic_reader.set_consumer(consumer);

    let decoder = SalInfo::make_decoder();

    println!("Reader heartbeats...");
    for i in 0..10 {
        println!("Iteration {i}...");
        topic_reader.pool(&decoder, Duration::from_secs(5)).await;
        println!("==> reading data queue!");
        loop {
            if let Some(new_data) = topic_reader.pop_next(false, Duration::from_secs(1)).await {
                let data_dict: HashMap<String, Value> = new_data
                    .fields
                    .into_iter()
                    .map(|(field, value)| (field, value))
                    .collect();
                let private_snd_stamp = data_dict.get("private_seqNum").unwrap();
                println!("\t{private_snd_stamp:?}");
            } else {
                println!("No new data...");
                break;
            };
        }
        // time::sleep(time::Duration::from_secs(1)).await;
    }
    println!("Done...");
}
