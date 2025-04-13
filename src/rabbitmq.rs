use crate::error::{AppError, Result};
use crate::config::RabbitMQConfig;
use lapin::{Connection, ConnectionProperties, options::*, types::FieldTable};
use lapin::message::Delivery;
use log::{info, error};
use tokio::sync::mpsc;
use futures_util::StreamExt;

pub struct RabbitMQConsumer {
    config: RabbitMQConfig,
}

impl RabbitMQConsumer {
    pub fn new(config: RabbitMQConfig) -> Self {
        Self { config }
    }

    pub async fn start(&self, message_sender: mpsc::Sender<Delivery>) -> Result<()> {
        info!("Connecting to RabbitMQ at {}", self.config.uri);
        
        // Create a connection to RabbitMQ
        let connection = Connection::connect(
            &self.config.uri,
            ConnectionProperties::default(),
        ).await.map_err(|e| {
            error!("Failed to connect to RabbitMQ: {}", e);
            AppError::RabbitMQConnectionError(e)
        })?;
        
        info!("Connected to RabbitMQ");
        
        // Create a channel
        let channel = connection.create_channel().await.map_err(|e| {
            error!("Failed to create channel: {}", e);
            AppError::RabbitMQConnectionError(e)
        })?;
        
        info!("Created channel");
        
        // Declare the queue to consume from
        let queue = channel.queue_declare(
            &self.config.queue,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        ).await.map_err(|e| {
            error!("Failed to declare queue: {}", e);
            AppError::RabbitMQConnectionError(e)
        })?;
        
        info!("Declared queue: {}", self.config.queue);
        
        // Start consuming messages
        let mut consumer = channel.basic_consume(
            &self.config.queue,
            "rabbitmq-kafka-bridge",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        ).await.map_err(|e| {
            error!("Failed to start consuming: {}", e);
            AppError::RabbitMQConnectionError(e)
        })?;
        
        info!("Started consuming from queue: {}", self.config.queue);
        
        // Process incoming messages
        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    info!("Received message from RabbitMQ");
                    
                    // Send the delivery to the processor
                    if let Err(e) = message_sender.send(delivery).await {
                        error!("Failed to send message to processor: {}", e);
                    }
                },
                Err(e) => {
                    error!("Failed to receive message: {}", e);
                }
            }
        }
        
        Ok(())
    }
}