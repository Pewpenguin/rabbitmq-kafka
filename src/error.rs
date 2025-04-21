use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),
    
    #[error("RabbitMQ connection error: {0}")]
    RabbitMQConnectionError(#[from] lapin::Error),
    
    #[error("Kafka producer error: {0}")]
    KafkaProducerError(String),
    
    #[error("JSON deserialization error: {0}")]
    JsonDeserializationError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("RSM protocol parsing error: {0}")]
    RsmParsingError(String),
}

pub type Result<T> = std::result::Result<T, AppError>;