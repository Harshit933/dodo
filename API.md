# Dodo API Documentation

This document provides detailed information about the available API endpoints in the Dodo application.

## Base URL

All API endpoints are prefixed with: `http://localhost:8080`

## Authentication

All endpoints except `/v1/register` and `/v1/auth` require a valid JWT token in the Authorization header:
```
Authorization: Bearer <your_jwt_token>
```

## API Endpoints

### Authentication

#### Register User
```http
POST /v1/register
```

Request body:
```json
{
    "email": "user@example.com",
    "password": "your_password",
    "name": "John Doe"
}
```

Response:
```json
{
    "token": "jwt_token",
    "user": {
        "id": "uuid",
        "email": "user@example.com",
        "name": "John Doe",
        "created_at": "timestamp",
        "updated_at": "timestamp"
    }
}
```

#### Login
```http
POST /v1/auth
```

Request body:
```json
{
    "email": "user@example.com",
    "password": "your_password"
}
```

Response:
```json
{
    "token": "jwt_token",
    "user": {
        "id": "uuid",
        "email": "user@example.com",
        "name": "John Doe",
        "created_at": "timestamp",
        "updated_at": "timestamp"
    }
}
```

### Transactions

#### Create Transaction
```http
POST /v1/users/{user_id}/transactions
```

Request body:
```json
{
    "amount": "100.50",
    "transaction_type": "Credit",  // or "Debit"
    "description": "Initial deposit"
}
```

Response:
```json
{
    "id": "uuid",
    "user_id": "uuid",
    "amount": "100.50",
    "transaction_type": "Credit",
    "description": "Initial deposit",
    "created_at": "timestamp"
}
```

#### Get All Transactions
```http
GET /v1/users/{user_id}/transactions
```

Response:
```json
[
    {
        "id": "uuid",
        "user_id": "uuid",
        "amount": "100.50",
        "transaction_type": "Credit",
        "description": "Initial deposit",
        "created_at": "timestamp"
    },
    {
        "id": "uuid",
        "user_id": "uuid",
        "amount": "25.75",
        "transaction_type": "Debit",
        "description": "Withdrawal",
        "created_at": "timestamp"
    }
]
```

#### Get Account Balance
```http
GET /v1/users/{user_id}/balance
```

Response:
```json
{
    "balance": "74.75"
}
```

## Error Responses

The API uses standard HTTP status codes:

- `200 OK`: Request successful
- `201 Created`: Resource created successfully
- `400 Bad Request`: Invalid request parameters
- `401 Unauthorized`: Invalid or missing authentication
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource already exists (e.g., email already registered)
- `500 Internal Server Error`: Server-side error

Error response format:
```json
{
    "error": "Error message description"
}
```

## Data Types

### Transaction Types
- `Credit`: Adds to the account balance
- `Debit`: Subtracts from the account balance

### Amount Format
- All monetary amounts are represented as decimal numbers
- Maximum precision of 2 decimal places
- Example: "100.50", "25.75"

### Timestamps
- All timestamps are in UTC
- Format: ISO 8601 (e.g., "2024-03-20T12:34:56.789Z")

## Rate Limiting

Currently, there are no rate limits implemented. However, it's recommended to implement rate limiting in a production environment.

## Security Considerations

1. All passwords are hashed using bcrypt before storage
2. JWT tokens expire after 24 hours
3. All monetary transactions are performed within database transactions to ensure consistency
4. Input validation is performed on all endpoints
5. SQL injection protection is implemented using parameterized queries

## Example Usage

### Complete Flow Example

1. Register a new user:
```bash
curl -X POST http://localhost:8080/v1/register \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password123","name":"John Doe"}'
```

2. Login to get JWT token:
```bash
curl -X POST http://localhost:8080/v1/auth \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password123"}'
```

3. Create a credit transaction:
```bash
curl -X POST http://localhost:8080/v1/users/{user_id}/transactions \
  -H "Authorization: Bearer {jwt_token}" \
  -H "Content-Type: application/json" \
  -d '{"amount":"100.50","transaction_type":"Credit","description":"Initial deposit"}'
```

4. Create a debit transaction:
```bash
curl -X POST http://localhost:8080/v1/users/{user_id}/transactions \
  -H "Authorization: Bearer {jwt_token}" \
  -H "Content-Type: application/json" \
  -d '{"amount":"25.75","transaction_type":"Debit","description":"Withdrawal"}'
```

5. Check account balance:
```bash
curl -X GET http://localhost:8080/v1/users/{user_id}/balance \
  -H "Authorization: Bearer {jwt_token}"
```

6. View transaction history:
```bash
curl -X GET http://localhost:8080/v1/users/{user_id}/transactions \
  -H "Authorization: Bearer {jwt_token}"
``` 