mod config;
mod error;
mod kafka;
mod processor;
mod rabbitmq;

use config::AppConfig;
use error::{AppError, Result};
use kafka::KafkaProducer;
use processor::MessageProcessor;
use rabbitmq::RabbitMQConsumer;
use log::{info, error};
use tokio::sync::mpsc;
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    info!("Starting RabbitMQ to Kafka bridge");

    let config = AppConfig::new().map_err(|e| {
        error!("Failed to load configuration: {}", e);
        AppError::ConfigError(e)
    })?;
    
    info!("Configuration loaded successfully");

    let (sender, receiver) = mpsc::channel(100);

    let kafka_producer = KafkaProducer::new(config.kafka)?;

    let processor = MessageProcessor::new(kafka_producer);

    let rabbitmq_consumer = RabbitMQConsumer::new(config.rabbitmq);

    let processor_handle = task::spawn(async move {
        if let Err(e) = processor.start(receiver).await {
            error!("Message processor error: {}", e);
        }
    });

    if let Err(e) = rabbitmq_consumer.start(sender).await {
        error!("RabbitMQ consumer error: {}", e);
        return Err(e);
    }
    
    if let Err(e) = processor_handle.await {
        error!("Failed to join processor task: {}", e);
    }
    
    info!("RabbitMQ to Kafka bridge stopped");
    
    Ok(())
}
