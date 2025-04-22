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
- CMake 3.5+ (required for rdkafka dependency)

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

### Local Development

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

### Docker Deployment

This project includes a complete Docker setup with:

- RabbitMQ with management UI
- Kafka and Zookeeper
- Automatic Kafka topic creation
- The RabbitMQ to Kafka bridge application

#### Using Docker Compose Directly

```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f rabbitmq-kafka-bridge

# Stop all services
docker-compose down
```

#### Using Deployment Scripts

For convenience, deployment scripts are provided:

**Linux/macOS (Bash):**
```bash
# Make the script executable
chmod +x docker-deploy.sh

# Start services
./docker-deploy.sh start

# View logs
./docker-deploy.sh logs

# Check status
./docker-deploy.sh status

# Stop services
./docker-deploy.sh stop
```

**Windows (PowerShell):**
```powershell
# Start services
.\docker-deploy.ps1 start

# View logs
.\docker-deploy.ps1 logs

# Check status
.\docker-deploy.ps1 status

# Stop services
.\docker-deploy.ps1 stop
```

The Docker setup automatically ensures CMake 3.5+ is available during the build process.

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