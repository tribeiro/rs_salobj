use apache_avro::types::Value;
use kafka::client::KafkaClient;
use std::collections::HashMap;

use salobj::{
    domain,
    sal_info::SalInfo,
    topics::{base_topic::BaseTopic, read_topic::ReadTopic},
};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let mut client = KafkaClient::new(vec!["localhost:9092".to_owned()]);
    client.set_client_id("kafka-rust-console-producer".into());
    client.load_metadata_all().unwrap();

    let domain = domain::Domain::new();
    let component = "Test";
    let topic = "logevent_heartbeat";

    let _ = client.load_metadata(&[topic]);

    assert!(client.topics().contains(&topic));

    let sal_info = SalInfo::new(component, 1).unwrap();
    let max_history: usize = 10;

    let mut topic_reader = ReadTopic::new(topic, &sal_info, &domain, max_history);

    println!(
        "Reading topic: {} group: {}",
        topic_reader.get_topic_publish_name(),
        topic_reader.get_record_type()
    );

    println!("Reader heartbeats...");
    for i in 0..10 {
        println!("Iteration {i}...");
        println!("==> reading data queue!");
        loop {
            if let Some(Value::Record(new_data)) =
                topic_reader.pop_back(false, Duration::from_secs(1)).await
            {
                let data_dict: HashMap<String, Value> = new_data
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
