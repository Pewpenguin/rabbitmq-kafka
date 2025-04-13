use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct RabbitMQConfig {
    pub uri: String,
    pub queue: String,
}

#[derive(Debug, Deserialize)]
pub struct KafkaConfig {
    pub brokers: String,
    pub valid_topic: String,
    pub invalid_topic: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub rabbitmq: RabbitMQConfig,
    pub kafka: KafkaConfig,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {

        let _ = dotenv::dotenv();
        
        // Determine the configuration file path
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config".to_string());
        
        // Build configuration
        let config = Config::builder()
            // Start with default configuration
            .add_source(File::with_name("config/default").required(false))
            // Add configuration from the specified file
            .add_source(File::with_name(&config_path).required(false))
            // Add configuration from environment variables prefixed with APP_
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?;
            
        // Deserialize the configuration into AppConfig
        config.try_deserialize()
    }
}