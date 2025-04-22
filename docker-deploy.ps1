# PowerShell deployment script for RabbitMQ to Kafka bridge

Write-Host "=== RabbitMQ to Kafka Bridge Docker Deployment ===" -ForegroundColor Cyan
Write-Host ""

$command = $args[0]

switch ($command) {
    "start" {
        Write-Host "Building and starting all services..." -ForegroundColor Green
        docker-compose up -d
        Write-Host ""
        Write-Host "Services started successfully!" -ForegroundColor Green
        Write-Host "- RabbitMQ Management UI: http://localhost:15672 (guest/guest)"
        Write-Host "- Kafka broker: localhost:29092"
        Write-Host ""
        Write-Host "To view logs: docker-compose logs -f rabbitmq-kafka-bridge"
    }
    "stop" {
        Write-Host "Stopping all services..." -ForegroundColor Yellow
        docker-compose down
        Write-Host "Services stopped." -ForegroundColor Yellow
    }
    "logs" {
        Write-Host "Showing logs for the bridge application..." -ForegroundColor Cyan
        docker-compose logs -f rabbitmq-kafka-bridge
    }
    "status" {
        Write-Host "Checking service status..." -ForegroundColor Cyan
        docker-compose ps
    }
    default {
        Write-Host "Usage: .\docker-deploy.ps1 {start|stop|logs|status}" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "Commands:"
        Write-Host "  start   - Build and start all services"
        Write-Host "  stop    - Stop all services"
        Write-Host "  logs    - Show logs for the bridge application"
        Write-Host "  status  - Show status of all services"
        exit 1
    }
}