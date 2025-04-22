#!/bin/bash

# Docker deployment script for RabbitMQ to Kafka bridge

set -e

echo "=== RabbitMQ to Kafka Bridge Docker Deployment ==="
echo ""

case "$1" in
  start)
    echo "Building and starting all services..."
    docker-compose up -d
    echo ""
    echo "Services started successfully!"
    echo "- RabbitMQ Management UI: http://localhost:15672 (guest/guest)"
    echo "- Kafka broker: localhost:29092"
    echo ""
    echo "To view logs: docker-compose logs -f rabbitmq-kafka-bridge"
    ;;
  stop)
    echo "Stopping all services..."
    docker-compose down
    echo "Services stopped."
    ;;
  logs)
    echo "Showing logs for the bridge application..."
    docker-compose logs -f rabbitmq-kafka-bridge
    ;;
  status)
    echo "Checking service status..."
    docker-compose ps
    ;;
  *)
    echo "Usage: $0 {start|stop|logs|status}"
    echo ""
    echo "Commands:"
    echo "  start   - Build and start all services"
    echo "  stop    - Stop all services"
    echo "  logs    - Show logs for the bridge application"
    echo "  status  - Show status of all services"
    exit 1
esac