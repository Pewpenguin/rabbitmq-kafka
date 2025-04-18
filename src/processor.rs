use crate::error::Result;
use crate::kafka::KafkaProducer;
use lapin::message::Delivery;
use log::{info, warn, error};
use serde_json::{Value, json};
use tokio::sync::mpsc;
use chrono::Utc;
use uuid::Uuid;

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
            Ok(json_value) => {
                info!("Successfully deserialized message as JSON");
                
                // Transform the message into the required vehicle tracking format
                let transformed_message = self.transform_to_vehicle_tracking_format(json_value);
                let transformed_json = serde_json::to_string(&transformed_message)
                    .unwrap_or_else(|_| payload.to_string());
                
                // Forward the transformed JSON message to the valid topic
                self.kafka_producer.send_valid_message(&transformed_json, None).await?
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
    
    fn transform_to_vehicle_tracking_format(&self, input: Value) -> Value {
        // Create the vehicle tracking format JSON as specified
        // If input already has some of these fields, use them, otherwise use defaults
        json!({
            "deviceId": input.get("deviceId").and_then(|v| v.as_str()).unwrap_or_else(|| {
                input.get("device_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("test-device-001")
            }),
            "gpsTime": input.get("gpsTime").and_then(|v| v.as_str()).unwrap_or_else(|| {
                input.get("timestamp")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| Utc::now().to_rfc3339())
            }),
            "deviceSpeed": input.get("deviceSpeed").and_then(|v| v.as_f64()).unwrap_or_else(|| {
                input.get("speed")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(45.5)
            }),
            "orientation": input.get("orientation").and_then(|v| v.as_f64()).unwrap_or_else(|| {
                input.get("direction")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(180.0)
            }),
            "latitude": input.get("latitude").and_then(|v| v.as_f64()).unwrap_or_else(|| {
                input.get("lat")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(12.9716)
            }),
            "longitude": input.get("longitude").and_then(|v| v.as_f64()).unwrap_or_else(|| {
                input.get("lng")
                    .or_else(|| input.get("lon"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(77.5946)
            }),
            "provider": input.get("provider").and_then(|v| v.as_str()).unwrap_or_else(|| {
                input.get("provider_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| Uuid::new_v4().to_string())
            }),
            "vehicleType": input.get("vehicleType").and_then(|v| v.as_str()).unwrap_or_else(|| {
                input.get("vehicle_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("car")
            })
        })
    }
}