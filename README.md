# User authentication and Payments API

**IMP** For API docs please see [API.md](https://github.com/Harshit933/dodo/blob/main/API.md).
**IMP** Docker image may be unstable at the current moment.
A secure and efficient user authentication and payment system built with Rust, Axum, and PostgreSQL.

## Features

- User registration and authentication
- JWT-based authentication
- Secure password hashing with bcrypt
- PostgreSQL database integration
- CORS support for frontend integration
- Comprehensive logging
- Input validation
- Error handling

## Prerequisites

- Rust (latest stable version)
- PostgreSQL
- Cargo (Rust's package manager)

## Environment Variables

Create a `.env` file in the root directory with the following variables:

```env
DATABASE_URL=postgres://username:password@localhost:5432/your_database
JWT_SECRET=your_jwt_secret_key
```

## Database Setup

1. Create a PostgreSQL database
2. Run the migrations:
```bash
sqlx database create
sqlx migrate run
```

## Running the Application

1. Clone the repository
2. Install dependencies:
```bash
cargo build
```

3. Run the application:
```bash
cargo run
```

The server will start on `http://localhost:8080`

## API Endpoints

### Authentication

#### Register User
```http
POST /api/auth/register
Content-Type: application/json

{
    "email": "user@example.com",
    "password": "your_password",
    "name": "User Name"
}
```

#### Login
```http
POST /api/auth/login
Content-Type: application/json

{
    "email": "user@example.com",
    "password": "your_password"
}
```

## Project Structure

```
src/
├── main.rs           # Application entry point
├── models/           # Data models
│   └── user.rs       # User model and related structures
├── handlers/         # Request handlers
│   └── auth.rs       # Authentication handlers
└── migrations/       # Database migrations
```

## Security Features

- Password hashing using bcrypt
- JWT token-based authentication
- CORS protection
- Input validation
- Secure database queries
- Error handling

## Development

### Running Tests

```bash
cargo test
```

### Database Migrations

To create a new migration:
```bash
sqlx migrate add <migration_name>
```

To run migrations:
```bash
sqlx migrate run
```

## Dependencies

- axum: Web framework
- tokio: Async runtime
- sqlx: Database toolkit
- jsonwebtoken: JWT implementation
- bcrypt: Password hashing
- tracing: Logging
- dotenv: Environment variable management

## License

MIT
