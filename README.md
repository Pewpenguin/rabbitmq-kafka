# RabbitMQ to Kafka Bridge

A Rust application that consumes messages from RabbitMQ and forwards them to Kafka. The application attempts to deserialize each message as JSON and routes it to appropriate Kafka topics based on the deserialization result.

## Features

- Consumes messages from a specified RabbitMQ queue
- Attempts to deserialize messages as JSON
- Forwards valid JSON messages to a specified Kafka topic
- Forwards invalid/unparseable messages to a separate Kafka topic
- Robust error handling and logging
- Configurable via environment variables or configuration files

## Requirements

- Rust (latest stable version)
- RabbitMQ server
- Kafka broker

## Configuration

The application can be configured using:

1. A configuration file (default: `config/default.toml`)
2. Environment variables prefixed with `APP__`

### Configuration Options

```toml
[rabbitmq]
uri = "amqp://guest:guest@localhost:5672"  # RabbitMQ connection URI
queue = "messages"                         # Queue to consume from

[kafka]
brokers = "localhost:9092"                 # Kafka brokers
valid_topic = "valid-messages"             # Topic for valid JSON messages
invalid_topic = "unparsed-messages"        # Topic for invalid/unparseable messages
```

### Environment Variables

Environment variables override configuration file settings:

- `APP__RABBITMQ__URI`: RabbitMQ connection URI
- `APP__RABBITMQ__QUEUE`: RabbitMQ queue name
- `APP__KAFKA__BROKERS`: Kafka brokers
- `APP__KAFKA__VALID_TOPIC`: Topic for valid JSON messages
- `APP__KAFKA__INVALID_TOPIC`: Topic for invalid/unparseable messages
- `CONFIG_PATH`: Path to the configuration file (default: `config`)

## Building and Running

```bash
# Build the application
cargo build --release

# Run with default configuration
cargo run --release

# Run with custom configuration file
CONFIG_PATH=path/to/config cargo run --release

# Run with environment variables
APP__RABBITMQ__URI=amqp://user:pass@host:5672 APP__KAFKA__BROKERS=host:9092 cargo run --release
```

## Logging

The application uses the `env_logger` crate for logging. Set the `RUST_LOG` environment variable to control log levels:

```bash
RUST_LOG=info cargo run --release  # Default log level
RUST_LOG=debug cargo run --release  # More detailed logging
```

## Project Structure

- `src/main.rs`: Application entry point
- `src/config.rs`: Configuration management
- `src/error.rs`: Error handling
- `src/rabbitmq.rs`: RabbitMQ consumer
- `src/kafka.rs`: Kafka producer
- `src/processor.rs`: Message processing logic
- `config/default.toml`: Default configuration

## License

MIT