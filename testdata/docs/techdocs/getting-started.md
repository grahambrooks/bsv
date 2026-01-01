# Getting Started

This guide will help you set up and run the Payment Service locally.

## Prerequisites

- Docker and Docker Compose
- Node.js 18+
- PostgreSQL 14+
- Redis 7+

## Installation

1. Clone the repository:

```bash
git clone https://github.com/example/payment-service.git
cd payment-service
```

2. Install dependencies:

```bash
npm install
```

3. Set up environment variables:

```bash
cp .env.example .env
# Edit .env with your configuration
```

4. Start the database:

```bash
docker-compose up -d postgres redis
```

5. Run migrations:

```bash
npm run db:migrate
```

6. Start the service:

```bash
npm run dev
```

## Verification

The service should now be running at `http://localhost:3000`.

Test the health endpoint:

```bash
curl http://localhost:3000/health
```

Expected response:

```json
{
  "status": "healthy",
  "version": "1.0.0"
}
```

## Next Steps

- Review the [API Reference](./api-reference.md)
- Understand the [Architecture](./architecture.md)
- Set up [Webhook Endpoints](./webhooks.md)
