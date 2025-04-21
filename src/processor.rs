use crate::error::{AppError, Result};
use crate::kafka::KafkaProducer;
use lapin::message::Delivery;
use log::{info, warn, error, debug};
use serde_json::{Value, json};
use tokio::sync::mpsc;
use chrono::Utc;
use uuid::Uuid;
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
        
        if payload.starts_with("$RSM") {
            debug!("Detected RSM protocol message");
            match self.parse_rsm_message(&payload) {
                Ok(parsed_data) => {
                    info!("Successfully parsed RSM protocol message");
                    
                    
                    let transformed_message = self.transform_rsm_to_vehicle_tracking_format(parsed_data);
                    let transformed_json = serde_json::to_string(&transformed_message)
                        .unwrap_or_else(|_| payload.to_string());
                    
                    
                    self.kafka_producer.send_valid_message(&transformed_json, None).await?
                },
                Err(e) => {
                    warn!("Failed to parse RSM protocol message: {}", e);
                    
                    self.kafka_producer.send_invalid_message(&payload, None).await?
                }
            }
        } else {
            
            match serde_json::from_str::<Value>(&payload) {
                Ok(json_value) => {
                    info!("Successfully deserialized message as JSON");
                    
                    
                    let transformed_message = self.transform_to_vehicle_tracking_format(json_value);
                    let transformed_json = serde_json::to_string(&transformed_message)
                        .unwrap_or_else(|_| payload.to_string());
                    
                    
                    self.kafka_producer.send_valid_message(&transformed_json, None).await?
                },
                Err(e) => {
                    warn!("Failed to deserialize message as JSON: {}", e);
                    
                    self.kafka_producer.send_invalid_message(&payload, None).await?
                }
            }
        }
        
        
        if let Err(e) = delivery.ack(lapin::options::BasicAckOptions::default()).await {
            error!("Failed to acknowledge message: {}", e);
        }
        
        Ok(())
    }
    
    fn transform_to_vehicle_tracking_format(&self, input: Value) -> Value {
        
        let mut result = serde_json::Map::new();
        
        
        if let Some(device_id) = input.get("deviceId").and_then(|v| v.as_str()) {
            result.insert("deviceId".to_string(), Value::String(device_id.to_string()));
        } else if let Some(device_id) = input.get("device_id").and_then(|v| v.as_str()) {
            result.insert("deviceId".to_string(), Value::String(device_id.to_string()));
        }
        
        if let Some(gps_time) = input.get("gpsTime").and_then(|v| v.as_str()) {
            result.insert("gpsTime".to_string(), Value::String(gps_time.to_string()));
        } else if let Some(timestamp) = input.get("timestamp").and_then(|v| v.as_str()) {
            result.insert("gpsTime".to_string(), Value::String(timestamp.to_string()));
        }
        
        if let Some(speed) = input.get("deviceSpeed").and_then(|v| v.as_f64()) {
            if let Some(num) = serde_json::Number::from_f64(speed) {
                result.insert("deviceSpeed".to_string(), Value::Number(num));
            }
        } else if let Some(speed) = input.get("speed").and_then(|v| v.as_f64()) {
            if let Some(num) = serde_json::Number::from_f64(speed) {
                result.insert("deviceSpeed".to_string(), Value::Number(num));
            }
        }
        
        if let Some(orientation) = input.get("orientation").and_then(|v| v.as_f64()) {
            if let Some(num) = serde_json::Number::from_f64(orientation) {
                result.insert("orientation".to_string(), Value::Number(num));
            }
        } else if let Some(direction) = input.get("direction").and_then(|v| v.as_f64()) {
            if let Some(num) = serde_json::Number::from_f64(direction) {
                result.insert("orientation".to_string(), Value::Number(num));
            }
        }
        
        if let Some(latitude) = input.get("latitude").and_then(|v| v.as_f64()) {
            if let Some(num) = serde_json::Number::from_f64(latitude) {
                result.insert("latitude".to_string(), Value::Number(num));
            }
        } else if let Some(lat) = input.get("lat").and_then(|v| v.as_f64()) {
            if let Some(num) = serde_json::Number::from_f64(lat) {
                result.insert("latitude".to_string(), Value::Number(num));
            }
        }
        
        if let Some(longitude) = input.get("longitude").and_then(|v| v.as_f64()) {
            if let Some(num) = serde_json::Number::from_f64(longitude) {
                result.insert("longitude".to_string(), Value::Number(num));
            }
        } else if let Some(lng) = input.get("lng").and_then(|v| v.as_f64()) {
            if let Some(num) = serde_json::Number::from_f64(lng) {
                result.insert("longitude".to_string(), Value::Number(num));
            }
        } else if let Some(lon) = input.get("lon").and_then(|v| v.as_f64()) {
            if let Some(num) = serde_json::Number::from_f64(lon) {
                result.insert("longitude".to_string(), Value::Number(num));
            }
        }
        
        if let Some(provider) = input.get("provider").and_then(|v| v.as_str()) {
            result.insert("provider".to_string(), Value::String(provider.to_string()));
        } else if let Some(provider_id) = input.get("provider_id").and_then(|v| v.as_str()) {
            result.insert("provider".to_string(), Value::String(provider_id.to_string()));
        }
        
        if let Some(vehicle_type) = input.get("vehicleType").and_then(|v| v.as_str()) {
            result.insert("vehicleType".to_string(), Value::String(vehicle_type.to_string()));
        } else if let Some(vehicle_type) = input.get("vehicle_type").and_then(|v| v.as_str()) {
            result.insert("vehicleType".to_string(), Value::String(vehicle_type.to_string()));
        }
        
        Value::Object(result)
    }
    
    fn parse_rsm_message(&self, message: &str) -> Result<HashMap<String, String>> {
        
        let parts: Vec<&str> = message.split(',').collect();
        
        
        if parts.len() < 20 {
            return Err(AppError::RsmParsingError(format!("Invalid RSM message format: insufficient fields ({})", parts.len())));
        }
        
        
        let mut parsed_data = HashMap::new();
        
        
        
        
        
        if !parts[0].starts_with("$RSM") {
            return Err(AppError::RsmParsingError("Message does not start with $RSM".to_string()));
        }
        
        
        parsed_data.insert("header".to_string(), parts[0].to_string());
        parsed_data.insert("vendor_id".to_string(), parts[1].to_string());
        parsed_data.insert("firmware_version".to_string(), parts[2].to_string());
        parsed_data.insert("packet_type".to_string(), parts[3].to_string());
        parsed_data.insert("packet_status".to_string(), parts[5].to_string());
        parsed_data.insert("imei".to_string(), parts[6].to_string());
        parsed_data.insert("vehicle_reg_no".to_string(), parts[7].to_string());
        parsed_data.insert("gps_fix".to_string(), parts[8].to_string());
        
        
        if parts.len() > 9 {
            parsed_data.insert("date_time".to_string(), parts[9].to_string());
        }
        
        
        if parts.len() > 11 {
            let lat = parts[10].to_string();
            let lat_dir = parts[11].to_string();
            parsed_data.insert("latitude".to_string(), lat);
            parsed_data.insert("latitude_dir".to_string(), lat_dir);
        }
        
        
        if parts.len() > 13 {
            let lng = parts[12].to_string();
            let lng_dir = parts[13].to_string();
            parsed_data.insert("longitude".to_string(), lng);
            parsed_data.insert("longitude_dir".to_string(), lng_dir);
        }
        
        
        if parts.len() > 14 {
            parsed_data.insert("speed".to_string(), parts[14].to_string());
        }
        
        
        if parts.len() > 15 {
            parsed_data.insert("heading".to_string(), parts[15].to_string());
        }
        
        
        if parts.len() > 16 {
            parsed_data.insert("satellites".to_string(), parts[16].to_string());
        }
        
        
        if parts.len() > 17 {
            parsed_data.insert("altitude".to_string(), parts[17].to_string());
        }
        
        
        if parts.len() > 18 {
            parsed_data.insert("pdop".to_string(), parts[18].to_string());
        }
        
        
        if parts.len() > 19 {
            parsed_data.insert("hdop".to_string(), parts[19].to_string());
        }
        
        
        if parts.len() > 20 {
            parsed_data.insert("network_operator".to_string(), parts[20].to_string());
        }
        
        
        if parts.len() > 21 {
            parsed_data.insert("ignition".to_string(), parts[21].to_string());
        }
        
        
        if parts.len() > 22 {
            parsed_data.insert("main_power_status".to_string(), parts[22].to_string());
        }
        
        
        if parts.len() > 23 {
            parsed_data.insert("main_input_voltage".to_string(), parts[23].to_string());
        }
        
        
        if parts.len() > 24 {
            parsed_data.insert("internal_battery_voltage".to_string(), parts[24].to_string());
        }
        
        
        if parts.len() > 25 {
            parsed_data.insert("emergency_status".to_string(), parts[25].to_string());
        }
        
        
        if parts.len() > 26 {
            parsed_data.insert("tamper_alert".to_string(), parts[26].to_string());
        }
        
        
        if parts.len() > 27 {
            parsed_data.insert("gsm_signal_strength".to_string(), parts[27].to_string());
        }
        
        
        if parts.len() > 28 {
            parsed_data.insert("mcc".to_string(), parts[28].to_string());
        }
        
        
        if parts.len() > 29 {
            parsed_data.insert("mnc".to_string(), parts[29].to_string());
        }
        
        Ok(parsed_data)
    }
    
    fn transform_rsm_to_vehicle_tracking_format(&self, rsm_data: HashMap<String, String>) -> Value {
        let mut result = serde_json::Map::new();
        
        if let Some(imei) = rsm_data.get("imei") {
            result.insert("deviceId".to_string(), Value::String(imei.clone()));
        }
        
        if let Some(date_time) = rsm_data.get("date_time") {
            result.insert("gpsTime".to_string(), Value::String(date_time.clone()));
        }
        
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
        
        if let Some(speed_str) = rsm_data.get("speed") {
            if let Ok(speed_value) = speed_str.parse::<f64>() {
                if let Some(num) = serde_json::Number::from_f64(speed_value) {
                    result.insert("deviceSpeed".to_string(), Value::Number(num));
                }
            }
        }
        
        if let Some(heading_str) = rsm_data.get("heading") {
            if let Ok(heading_value) = heading_str.parse::<f64>() {
                if let Some(num) = serde_json::Number::from_f64(heading_value) {
                    result.insert("orientation".to_string(), Value::Number(num));
                }
            }
        }
        
        if let Some(network_operator) = rsm_data.get("network_operator") {
            result.insert("provider".to_string(), Value::String(network_operator.clone()));
        }
        
        
        if let Some(vehicle_reg_no) = rsm_data.get("vehicle_reg_no") {
            result.insert("vehicleRegNo".to_string(), Value::String(vehicle_reg_no.clone()));
        }
        
        if let Some(satellites_str) = rsm_data.get("satellites") {
            if let Ok(satellites_value) = satellites_str.parse::<i32>() {
                result.insert("satellites".to_string(), Value::Number(satellites_value.into()));
            }
        }
        
        if let Some(altitude_str) = rsm_data.get("altitude") {
            if let Ok(altitude_value) = altitude_str.parse::<f64>() {
                if let Some(num) = serde_json::Number::from_f64(altitude_value) {
                    result.insert("altitude".to_string(), Value::Number(num));
                }
            }
        }
        
        if let Some(ignition) = rsm_data.get("ignition") {
            result.insert("ignitionStatus".to_string(), Value::Bool(ignition == "1"));
        }
        
        if let Some(main_power) = rsm_data.get("main_power_status") {
            result.insert("mainPowerStatus".to_string(), Value::Bool(main_power == "1"));
        }
        
        if let Some(battery_str) = rsm_data.get("internal_battery_voltage") {
            if let Ok(battery_value) = battery_str.parse::<f64>() {
                if let Some(num) = serde_json::Number::from_f64(battery_value) {
                    result.insert("batteryVoltage".to_string(), Value::Number(num));
                }
            }
        }
        
        if let Some(signal_str) = rsm_data.get("gsm_signal_strength") {
            if let Ok(signal_value) = signal_str.parse::<i32>() {
                result.insert("gsmSignalStrength".to_string(), Value::Number(signal_value.into()));
            }
        }
        
        
        if let Some(emergency) = rsm_data.get("emergency_status") {
            result.insert("emergencyStatus".to_string(), Value::Bool(emergency == "1"));
        }
        
        
        if let Some(tamper) = rsm_data.get("tamper_alert") {
            result.insert("tamperAlert".to_string(), Value::String(tamper.clone()));
        }
        
        
        let raw_message = rsm_data.iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<String>>().join(", ");
        result.insert("rawMessage".to_string(), Value::String(raw_message));
        
        Value::Object(result)
    }
}