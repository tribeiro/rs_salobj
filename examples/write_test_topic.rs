use apache_avro::types::Value;
use kafka::{
    client::KafkaClient,
    producer::{Producer, RequiredAcks},
};
use salobj::{
    domain,
    sal_info::SalInfo,
    topics::{base_topic::BaseTopic, write_topic::WriteTopic},
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

    let mut topic_writer = WriteTopic::new(domain, &sal_info, topic);

    let producer = Producer::from_hosts(vec!["localhost:9092".to_owned()])
        .with_ack_timeout(Duration::from_secs(1))
        .with_required_acks(RequiredAcks::One)
        .create()
        .unwrap();

    topic_writer.set_producer(producer);

    let test_scalars_schema = WriteTopic::get_avro_schema(&sal_info, &topic_writer.get_sal_name());
    let encoder = SalInfo::make_encoder();

    println!("Writing heartbeat...");
    for i in 0..10 {
        println!("Sending hearbeat {i}...");
        let mut test_scalars_record = WriteTopic::make_data_type(&test_scalars_schema);

        test_scalars_record.put("heartbeat", Value::Boolean(false));

        topic_writer.set_write(test_scalars_record, &encoder).await;
        time::sleep(time::Duration::from_secs(1)).await;
    }
    println!("Done...");
}
