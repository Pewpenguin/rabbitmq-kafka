use crate::error::{AppError, Result};
use crate::kafka::KafkaProducer;
use lapin::message::Delivery;
use log::{info, warn, error, debug};
use serde_json::Value;
use tokio::sync::mpsc;
use std::collections::HashMap;

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
        
        let result = if payload.starts_with("$RSM") {
            debug!("Detected RSM protocol message");
            match self.parse_rsm_message(&payload) {
                Ok(parsed_data) => {
                    info!("Successfully parsed RSM protocol message");
                    let transformed = self.transform_rsm_to_vehicle_tracking_format(parsed_data);
                    self.kafka_producer.send_valid_message(
                        &serde_json::to_string(&transformed).unwrap_or_else(|_| payload.to_string()),
                        None
                    ).await
                },
                Err(e) => {
                    warn!("Failed to parse RSM protocol message: {}", e);
                    self.kafka_producer.send_invalid_message(&payload, None).await
                }
            }
        } else {
            warn!("Received non-RSM message, sending to invalid topic");
            self.kafka_producer.send_invalid_message(&payload, None).await
        };
        
        if let Err(e) = delivery.ack(lapin::options::BasicAckOptions::default()).await {
            error!("Failed to acknowledge message: {}", e);
        }
        
        result
    }
    

    
    fn parse_rsm_message(&self, message: &str) -> Result<HashMap<String, String>> {
        let parts: Vec<&str> = message.split(',').collect();
        
        if parts.len() < 20 {
            return Err(AppError::RsmParsingError(format!("Invalid RSM message format: insufficient fields ({})", parts.len())));
        }
        
        if !parts[0].starts_with("$RSM") {
            return Err(AppError::RsmParsingError("Message does not start with $RSM".to_string()));
        }
        
        let mut parsed_data = HashMap::new();
        
        // Define field mappings with indices
        let field_mappings = [
            ("header", 0), ("vendor_id", 1), ("firmware_version", 2),
            ("packet_type", 3), ("packet_status", 5), ("imei", 6),
            ("vehicle_reg_no", 7), ("gps_fix", 8), ("date_time", 9),
            ("latitude", 10), ("latitude_dir", 11), ("longitude", 12),
            ("longitude_dir", 13), ("speed", 14), ("heading", 15),
            ("satellites", 16), ("altitude", 17), ("pdop", 18),
            ("hdop", 19), ("network_operator", 20), ("ignition", 21),
            ("main_power_status", 22), ("main_input_voltage", 23),
            ("internal_battery_voltage", 24), ("emergency_status", 25),
            ("tamper_alert", 26), ("gsm_signal_strength", 27),
            ("mcc", 28), ("mnc", 29)
        ];
        
        // Insert fields if they exist in the parts array
        for (field, index) in field_mappings.iter() {
            if parts.len() > *index {
                parsed_data.insert(field.to_string(), parts[*index].to_string());
            }
        }
        
        Ok(parsed_data)
    }
    
    fn transform_rsm_to_vehicle_tracking_format(&self, rsm_data: HashMap<String, String>) -> Value {
        let mut result = serde_json::Map::new();
        
        // Helper closure for inserting string values
        let insert_str = |map: &mut serde_json::Map<String, Value>, key: &str, value: &str| {
            map.insert(key.to_string(), Value::String(value.to_string()));
        };
        
        // Helper closure for parsing and inserting numeric values
        let parse_and_insert_f64 = |map: &mut serde_json::Map<String, Value>, key: &str, value_str: &str| {
            if let Ok(value) = value_str.parse::<f64>() {
                if let Some(num) = serde_json::Number::from_f64(value) {
                    map.insert(key.to_string(), Value::Number(num));
                }
            }
        };
        
        
        // deviceId - Unique identifier for the tracking device
        if let Some(imei) = rsm_data.get("imei") {
            insert_str(&mut result, "deviceId", imei);
        }
        
        // gpsTime - Date and time in UTC
        if let Some(date_time) = rsm_data.get("date_time") {
            // Format the date_time to ISO 8601 format with UTC timezone if needed
            // For now, just use the value as is
            insert_str(&mut result, "gpsTime", date_time);
        } 
        
        // deviceSpeed - Speed of the device in km/h or mph
        if let Some(speed_str) = rsm_data.get("speed") {
            parse_and_insert_f64(&mut result, "deviceSpeed", speed_str);
        } 
        
        // orientation - Direction in degrees
        if let Some(heading_str) = rsm_data.get("heading") {
            parse_and_insert_f64(&mut result, "orientation", heading_str);
        } 
        
        // latitude - Geographic coordinate
        if let Some(lat_str) = rsm_data.get("latitude") {
            if let Ok(mut lat_value) = lat_str.parse::<f64>() {
                if let Some(dir) = rsm_data.get("latitude_dir") {
                    if dir == "S" {
                        lat_value = -lat_value;
                    }
                }
                if let Some(num) = serde_json::Number::from_f64(lat_value) {
                    result.insert("latitude".to_string(), Value::Number(num));
                }
            }
        } 
        
        // longitude - Geographic coordinate
        if let Some(lng_str) = rsm_data.get("longitude") {
            if let Ok(mut lng_value) = lng_str.parse::<f64>() {
                if let Some(dir) = rsm_data.get("longitude_dir") {
                    if dir == "W" {
                        lng_value = -lng_value;
                    }
                }
                if let Some(num) = serde_json::Number::from_f64(lng_value) {
                    result.insert("longitude".to_string(), Value::Number(num));
                }
            }
        } 
        
        // provider - Unique client ID for VTS
        if let Some(network_operator) = rsm_data.get("network_operator") {
            insert_str(&mut result, "provider", network_operator);
        } 
        
        Value::Object(result)
    }
}