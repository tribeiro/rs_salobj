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

    let sal_info = SalInfo::new(component, 1).unwrap();

    let mut topic_writer = WriteTopic::new(topic, &sal_info, &domain);

    let sal_name = sal_info.get_sal_name(&topic);
    let schema = sal_info.get_topic_schema(&sal_name).unwrap().clone();

    println!("Writing heartbeat...");
    for i in 0..10 {
        println!("Sending hearbeat {i}...");
        let mut test_scalars_record = WriteTopic::make_data_type(&schema).unwrap();

        test_scalars_record.put("heartbeat", Value::Boolean(false));

        topic_writer
            .write(&mut test_scalars_record, &sal_info)
            .await
            .unwrap();
        time::sleep(time::Duration::from_secs(1)).await;
    }
    println!("Done...");
}
