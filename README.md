# InnoSystem - Innovategy's Innovation Ecosystem Platform

## Overview

InnoSystem is a high-performance innovation ecosystem platform developed by Innovategy Oy. The platform is designed as a robust job queue system that can handle various workloads for multiple resellers, customers, projects, and tasks. Built with Rust for performance and safety, InnoSystem provides a secure, scalable solution for innovation management.

## Core Objectives

- Create a highly configurable job queue system
- Support asynchronous job execution with varying priorities
- Implement robust retry policies
- Provide an extensible runner system for job processing
- Establish a wallet-based billing system
- Enable future scaling for multi-tenant usage

## Project Structure

The InnoSystem platform is organized into several key components:

- **Core**: The foundation of the platform, containing:
  - **API**: RESTful API services for client interactions
  - **Common**: Shared utilities, models, and business logic
  - **Runner**: Background job processing and scheduled tasks
  - **Tester**: Testing framework and utilities

- **Admin**: Administrative interface and management tools
- **Web**: Frontend web application for end users
- **SDK**: Software Development Kit for third-party integrations
- **Docs**: Comprehensive documentation
- **Specs**: Technical specifications and architecture documents

## Technology Stack

InnoSystem leverages a modern technology stack:

- **Backend**: Rust 1.85.1 with Axum web framework
- **Database**: PostgreSQL 15 with Diesel ORM
- **Caching/Messaging**: Redis 7 for job queue and fast data retrieval
- **API**: RESTful with JSON serialization
- **Authentication**: Token-based with secure password hashing
- **Containerization**: Docker and Docker Compose
- **Logging**: Structured logging with tracing

## Core Concepts

### Enhanced Entity Relationships

The platform is built around a hierarchical relationship model:

```
Reseller 1───┐
             │
             └──N Customer 1──┐
                              │
                              ├──N Project 1──┐
                              │               │
                              │               └──N Job Type ───M─┐
                              │               │                  │
                              │               └──N Job (1 Type)  │
                              │                     │            │
                              │                     │            │
                              │                     │      Compatible with
                              │                     │           │
                              │                     │           │
                              │                     └──N Wallet Transaction
                              │                     │
                              │                     │
                              │                     M
                              │                 Runner
                              │
                              └──1 Wallet (with Open-to-Buy capability)
```

### Key Features

1. **Job Queue System**
   - Priority-based job scheduling
   - Configurable retry policies
   - Job status tracking and reporting

2. **Wallet-Based Billing**
   - Open-to-Buy allocation tracking
   - Transaction history and reconciliation
   - Balance management across customers

3. **BYOK (Bring Your Own Key) Encryption**
   - Client-controlled encryption keys
   - Envelope encryption for sensitive data
   - Comprehensive access logging

4. **Runner Infrastructure**
   - Distributed job processing
   - Runner health monitoring
   - Job-runner compatibility matching

## Development Roadmap

InnoSystem is being developed in a phased approach:

### Phase 1: Foundation
- Core architecture setup
- Basic API structure with Axum
- Simple Redis-based job queue
- In-memory data models
- Basic runner process for job execution
- Logging infrastructure

### Phase 2: Persistence & Robust Queueing
- PostgreSQL integration with Diesel ORM
- Database migrations for core models
- Enhanced Redis queue with priority support
- Basic wallet balance tracking
- Repository layer implementation

### Phase 3: Entity Management & Billing
- Full CRUD APIs for Resellers, Customers, and Projects
- Wallet transaction logging
- Job costing and billing integration
- Runner registration and discovery
- Job-runner compatibility matching

### Phase 4: Advanced Security
- BYOK encryption system implementation
- Access transparency logging
- Enhanced authentication mechanisms
- Role-based access control
- Job retry policy enforcement

### Phase 5: Refinement
- Comprehensive testing
- Performance optimizations
- API documentation
- Deployment guides
- Code cleanup and refactoring

## Future Plans

The vision for InnoSystem extends beyond its current implementation:

- **Multi-Currency Support**: Extending the wallet system to handle multiple currencies with exchange rate management
- **Multi-Lingual Support**: Internationalization and localization for global usage
- **Advanced Analytics**: Comprehensive metrics and reporting for innovation processes
- **AI-Powered Insights**: Machine learning integration for predictive analytics and recommendations
- **Marketplace Integration**: Connecting innovation stakeholders through a collaborative ecosystem
- **Mobile Applications**: Extending platform access to mobile devices
- **Integration Ecosystem**: Expanding the SDK to enable seamless integration with third-party tools and services

## Getting Started

### Prerequisites

- Rust 1.85.1 or later
- Docker and Docker Compose
- PostgreSQL 15
- Redis 7

### Environment Setup

1. Clone the repository
2. Set up environment variables (see `.env.example`)
3. Build and start the services using Docker Compose

### Complete Application Setup

Follow these steps to set up and verify the complete application:

```bash
# Step 1: Build all services
docker compose build

# Step 2: Start the database and Redis services
docker compose up -d postgres redis

# Step 3: Run migrations and seed the database
docker compose run --rm migrations

# Step 4: Start the API and Runner services
docker compose up -d api runner

# Step 5: Run the tester to verify functionality
docker compose run --rm tester

# View logs from all services
docker compose logs -f

# View logs from a specific service
docker compose logs -f api
docker compose logs -f runner

# Stop all services
docker compose down

# To completely reset the environment (including volumes)
docker compose down -v
docker compose build
docker compose up -d postgres redis
docker compose run --rm migrations
docker compose up -d api runner
```

### Understanding the Setup Process

1. **Build Services**: Compiles all Docker images required for the application
2. **Start Database Services**: Initializes PostgreSQL and Redis
3. **Run Migrations**: Creates database schema and seeds initial data
   - Creates tables for job_types, customers, wallets, wallet_transactions, and jobs
   - Seeds the database with sample job types, customers, and initial wallet balances
4. **Start Application Services**: Launches the API and job runner
5. **Verify Functionality**: Runs the tester service which exercises all API endpoints
   - Tests the health endpoint
   - Tests customer creation and retrieval
   - Tests job type management
   - Tests job creation and status checking

### Logs Directory

When running the tester, logs are saved to the `./logs` directory. Check these logs for detailed information about the test execution and API responses.

## Security Features

InnoSystem implements banking-grade security features:

- **BYOK Encryption**: Client-controlled encryption keys for data protection
- **Envelope Encryption**: Secure key hierarchy for data encryption
- **Access Transparency**: Comprehensive logging of all data access
- **Secure Data Access**: Controlled access to sensitive data with audit logging

## Contact

For inquiries about InnoSystem, please contact:

**Sina Ghazi / Chairman**  
sina@innovategy.fi / +358413175455

**Innovategy Oy**  
https://innovategy.fi  
PL 10, 15101 Lahti, Finland

Connect with us:  
[LinkedIn](https://www.linkedin.com/company/innovategy) | [YouTube](https://www.youtube.com/@innovategy)

## License

© 2025 Innovategy Oy. All rights reserved.

This software is proprietary and confidential. Unauthorized copying, transfer, or reproduction of the contents of this software, via any medium, is strictly prohibited.
