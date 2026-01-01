# API Reference

## Base URL

```
https://api.example.com/payments/v1
```

## Authentication

All API requests require a Bearer token:

```
Authorization: Bearer <your-api-key>
```

## Endpoints

### Create Payment

```
POST /payments
```

Create a new payment intent.

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| amount | integer | Yes | Amount in cents |
| currency | string | Yes | ISO 4217 currency code |
| customer_id | string | No | Customer identifier |
| metadata | object | No | Custom key-value pairs |

**Example:**

```json
{
  "amount": 2500,
  "currency": "USD",
  "customer_id": "cus_123",
  "metadata": {
    "order_id": "ord_456"
  }
}
```

**Response:**

```json
{
  "id": "pay_abc123",
  "status": "pending",
  "amount": 2500,
  "currency": "USD",
  "created_at": "2024-01-15T10:30:00Z"
}
```

### Get Payment

```
GET /payments/{id}
```

Retrieve a payment by ID.

### List Payments

```
GET /payments
```

List all payments with pagination.

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| limit | integer | 20 | Max results per page |
| offset | integer | 0 | Pagination offset |
| status | string | - | Filter by status |

### Refund Payment

```
POST /payments/{id}/refund
```

Refund a completed payment.

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| amount | integer | No | Partial refund amount |
| reason | string | No | Refund reason |

## Error Codes

| Code | Description |
|------|-------------|
| 400 | Bad Request - Invalid parameters |
| 401 | Unauthorized - Invalid API key |
| 404 | Not Found - Resource doesn't exist |
| 422 | Unprocessable - Business logic error |
| 500 | Internal Server Error |

## Rate Limits

- 1000 requests per minute per API key
- Burst limit: 100 requests per second
