use crate::error::{AppError, Result};
use crate::config::KafkaConfig;
use log::{info, error};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use std::time::Duration;

pub struct KafkaProducer {
    config: KafkaConfig,
    producer: FutureProducer,
}

impl KafkaProducer {
    pub fn new(config: KafkaConfig) -> Result<Self> {
        info!("Initializing Kafka producer with brokers: {}", config.brokers);
        
        // Create a Kafka producer
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .map_err(|e| {
                error!("Failed to create Kafka producer: {}", e);
                AppError::KafkaProducerError(e.to_string())
            })?;
        
        info!("Kafka producer initialized");
        
        Ok(Self { config, producer })
    }
    
    pub async fn send_valid_message(&self, payload: &str, key: Option<&str>) -> Result<()> {
        self.send_message(&self.config.valid_topic, payload, key).await
    }
    
    pub async fn send_invalid_message(&self, payload: &str, key: Option<&str>) -> Result<()> {
        self.send_message(&self.config.invalid_topic, payload, key).await
    }
    
    async fn send_message(&self, topic: &str, payload: &str, key: Option<&str>) -> Result<()> {
        info!("Sending message to Kafka topic: {}", topic);
        
        let record = FutureRecord::to(topic)
            .payload(payload)
            .key(key.unwrap_or(""));
        
        match self.producer.send(record, Timeout::After(Duration::from_secs(5))).await {
            Ok((partition, offset)) => {
                info!("Message sent to Kafka topic: {}, partition: {}, offset: {}", topic, partition, offset);
                Ok(())
            },
            Err((e, _)) => {
                error!("Failed to send message to Kafka: {}", e);
                Err(AppError::KafkaProducerError(e.to_string()))
            }
        }
    }
}