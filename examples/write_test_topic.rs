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
    let mut domain = domain::Domain::new();
    let component = "Test";
    let topic = "logevent_heartbeat";

    let sal_info = SalInfo::new(component, 1).unwrap();

    if let Err(error) = domain.register_topics(&sal_info.get_topics_name()) {
        log::warn!("Failed to register topics: {error:?}. Continuing...");
    }
    sal_info.register_schema().await;

    let mut topic_writer = WriteTopic::new(topic, &sal_info, &domain);

    let schema = sal_info.get_topic_schema(&topic).unwrap().clone();

    println!("Writing heartbeat...");
    for i in 0..10 {
        println!("Sending hearbeat {i}...");
        let mut test_scalars_record = WriteTopic::make_data_type(&schema).unwrap();

        test_scalars_record.put("heartbeat", Value::Boolean(false));

        topic_writer.write(&mut test_scalars_record).await.unwrap();
        time::sleep(time::Duration::from_secs(1)).await;
    }
    println!("Done...");
}
