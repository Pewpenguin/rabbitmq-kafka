use crate::error::Result;
use crate::kafka::KafkaProducer;
use lapin::message::Delivery;
use log::{info, warn, error};
use serde_json::Value;
use tokio::sync::mpsc;

pub struct MessageProcessor {
    kafka_producer: KafkaProducer,
}

impl MessageProcessor {
    pub fn new(kafka_producer: KafkaProducer) -> Self {
        Self { kafka_producer }
    }

    pub async fn start(&self, mut message_receiver: mpsc::Receiver<Delivery>) -> Result<()> {
        info!("Message processor started");
        
        while let Some(delivery) = message_receiver.recv().await {
            self.process_message(delivery).await?;
        }
        
        Ok(())
    }
    
    async fn process_message(&self, delivery: Delivery) -> Result<()> {
        let payload = String::from_utf8_lossy(&delivery.data);
        info!("Processing message: {}", payload);
        
        // Try to deserialize the payload as JSON
        match serde_json::from_str::<Value>(&payload) {
            Ok(_json_value) => {
                info!("Successfully deserialized message as JSON");
                // Forward the valid JSON message to the valid topic
                self.kafka_producer.send_valid_message(&payload, None).await?
            },
            Err(e) => {
                warn!("Failed to deserialize message as JSON: {}", e);
                // Forward the invalid message to the invalid topic
                self.kafka_producer.send_invalid_message(&payload, None).await?
            }
        }
        
        // Acknowledge the message
        if let Err(e) = delivery.ack(lapin::options::BasicAckOptions::default()).await {
            error!("Failed to acknowledge message: {}", e);
        }
        
        Ok(())
    }
}