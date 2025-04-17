use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use log::{info, error};
use std::time::Duration;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    info!("Starting Kafka consumer test to verify message processing");

    let valid_topic = "valid-messages";
    let invalid_topic = "unparsed-messages";
    
    info!("Creating consumer for topic: {}", valid_topic);
    let valid_consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "test-consumer-group")
        .set("bootstrap.servers", "localhost:9092")
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest") // Start from the beginning
        .create()
        .expect("Failed to create valid messages consumer");

    info!("Creating consumer for topic: {}", invalid_topic);
    let invalid_consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "test-consumer-group")
        .set("bootstrap.servers", "localhost:9092")
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest") // Start from the beginning
        .create()
        .expect("Failed to create invalid messages consumer");

    valid_consumer.subscribe(&[valid_topic])?;
    invalid_consumer.subscribe(&[invalid_topic])?;

    info!("Subscribed to topics: {} and {}", valid_topic, invalid_topic);
    info!("Waiting for messages (will timeout after 10 seconds)...");

    let timeout = tokio::time::sleep(Duration::from_secs(10));
    let mut valid_count = 0;
    let mut invalid_count = 0;

    let valid_handle = tokio::spawn(async move {
        let mut message_stream = valid_consumer.stream();
        while let Ok(Some(message)) = tokio::time::timeout(Duration::from_secs(1), message_stream.next()).await {
            match message {
                Ok(borrowed_message) => {
                    valid_count += 1;
                    if let Some(payload) = borrowed_message.payload() {
                        let payload_str = String::from_utf8_lossy(payload);
                        info!("Received valid message: {}", payload_str);
                    }
                },
                Err(e) => error!("Error while receiving from valid topic: {}", e),
            }
        }
        valid_count
    });

    let invalid_handle = tokio::spawn(async move {
        let mut message_stream = invalid_consumer.stream();
        while let Ok(Some(message)) = tokio::time::timeout(Duration::from_secs(1), message_stream.next()).await {
            match message {
                Ok(borrowed_message) => {
                    invalid_count += 1;
                    if let Some(payload) = borrowed_message.payload() {
                        let payload_str = String::from_utf8_lossy(payload);
                        info!("Received invalid message: {}", payload_str);
                    }
                },
                Err(e) => error!("Error while receiving from invalid topic: {}", e),
            }
        }
        invalid_count
    });

    tokio::select! {
        _ = timeout => {
            info!("Timeout reached after 10 seconds");
        }
    }

    // Wait for consumer tasks to complete
    let valid_count = match valid_handle.await {
        Ok(count) => count,
        Err(e) => {
            error!("Error joining valid consumer task: {}", e);
            0
        }
    };

    let invalid_count = match invalid_handle.await {
        Ok(count) => count,
        Err(e) => {
            error!("Error joining invalid consumer task: {}", e);
            0
        }
    };

    // Report results
    info!("Test completed. Results:");
    info!("Valid messages received: {}", valid_count);
    info!("Invalid messages received: {}", invalid_count);

    if valid_count > 0 || invalid_count > 0 {
        info!("✅ Success! Messages were successfully processed and sent to Kafka topics.");
    } else {
        info!("❌ No messages were received. Check if the RabbitMQ to Kafka bridge is running properly.");
    }

    Ok(())
}