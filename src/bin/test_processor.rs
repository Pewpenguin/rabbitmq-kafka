use lapin::{Connection, ConnectionProperties, options::*, types::FieldTable, BasicProperties};
use log::info;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    info!("Starting RabbitMQ message publisher test");
    let uri = "amqp://guest:guest@localhost:5672";
    info!("Connecting to RabbitMQ at {}", uri);
    
    let connection = Connection::connect(
        uri,
        ConnectionProperties::default(),
    ).await?;
    
    info!("Connected to RabbitMQ");
    
    let channel = connection.create_channel().await?;
    info!("Created channel");

    let queue_name = "messages";
    let _queue = channel.queue_declare(
        queue_name,
        QueueDeclareOptions::default(),
        FieldTable::default(),
    ).await?;
    
    info!("Declared queue: {}", queue_name);
    
    let valid_json = json!({
        "id": 1,
        "name": "Test Message",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    info!("Publishing valid JSON message: {}", valid_json);
    
    channel.basic_publish(
        "",
        queue_name,
        BasicPublishOptions::default(),
        serde_json::to_string(&valid_json)?.as_bytes(),
        BasicProperties::default(),
    ).await?;
    
    info!("Valid JSON message published");
    
    sleep(Duration::from_secs(1)).await;

    let invalid_json = "This is not a valid JSON message";
    
    info!("Publishing invalid JSON message: {}", invalid_json);
    
    channel.basic_publish(
        "",
        queue_name,
        BasicPublishOptions::default(),
        invalid_json.as_bytes(),
        BasicProperties::default(),
    ).await?;
    
    info!("Invalid JSON message published");

    sleep(Duration::from_secs(1)).await;
    
    info!("Test completed. Check Kafka topics 'valid-messages' and 'unparsed-messages' to verify messages were processed correctly.");
    
    Ok(())
}