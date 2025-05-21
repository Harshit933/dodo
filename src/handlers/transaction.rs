use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use bigdecimal::BigDecimal;
use tracing::{info, error};

use crate::models::transaction::{Transaction, CreateTransaction, AccountBalance};

pub async fn create_transaction(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<CreateTransaction>,
) -> Result<Json<Transaction>, (StatusCode, String)> {
    info!("Creating transaction for user {}: {:?}", user_id, payload);
    
    // Check if user exists
    let user_exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1) as \"exists!\"",
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Failed to check user existence: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to check user existence".to_string())
    })?;
    if !user_exists {
        return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
    }

    let mut tx = pool.begin().await
        .map_err(|e| {
            error!("Failed to start transaction: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to start transaction".to_string())
        })?;

    let transaction = sqlx::query_as!(
        Transaction,
        r#"
        INSERT INTO transactions (user_id, amount, transaction_type, description)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, amount, transaction_type as "transaction_type: _", description, created_at
        "#,
        user_id,
        payload.amount,
        payload.transaction_type as _,
        payload.description
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to create transaction: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create transaction".to_string())
    })?;

    tx.commit().await
        .map_err(|e| {
            error!("Failed to commit transaction: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to commit transaction".to_string())
        })?;

    info!("Successfully created transaction: {:?}", transaction);
    Ok(Json(transaction))
}

pub async fn get_transactions(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<Transaction>>, (StatusCode, String)> {
    info!("Fetching transactions for user {}", user_id);
    
    let transactions = sqlx::query_as!(
        Transaction,
        r#"
        SELECT id, user_id, amount, transaction_type as "transaction_type: _", description, created_at
        FROM transactions
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Failed to fetch transactions: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch transactions".to_string())
    })?;

    info!("Found {} transactions for user {}", transactions.len(), user_id);
    Ok(Json(transactions))
}

pub async fn get_account_balance(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<AccountBalance>, (StatusCode, String)> {
    info!("Fetching balance for user {}", user_id);
    
    let balance = sqlx::query!(
        r#"
        SELECT 
            user_id,
            COALESCE(
                SUM(
                    CASE 
                        WHEN transaction_type = 'credit' THEN amount
                        WHEN transaction_type = 'debit' THEN -amount
                    END
                ),
                0
            ) as balance,
            MAX(created_at) as last_updated
        FROM transactions
        WHERE user_id = $1
        GROUP BY user_id
        "#,
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("Failed to fetch balance: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch balance".to_string())
    })?
    .ok_or((StatusCode::NOT_FOUND, "No transactions found".to_string()))?;

    let account_balance = AccountBalance {
        user_id: balance.user_id,
        balance: balance.balance.unwrap_or(BigDecimal::from(0)),
        last_updated: balance.last_updated,
    };

    info!("Balance for user {}: {:?}", user_id, account_balance);
    Ok(Json(account_balance))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::str::FromStr;
    use bigdecimal::BigDecimal;
    use crate::models::transaction::TransactionType;

    async fn setup_test_db() -> PgPool {
        // Use a test database URL
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/dodo_test".to_string());
        
        PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database")
    }

    // Helper function to clean up test data
    async fn cleanup_test_data(pool: &PgPool, user_id: Uuid) {
        sqlx::query!("DELETE FROM transactions WHERE user_id = $1", user_id)
            .execute(pool)
            .await
            .unwrap();
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(pool)
            .await
            .unwrap();
    }

    // Helper function to create a test user
    async fn create_test_user(pool: &PgPool, user_id: Uuid, email: &str) {
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, name)
            VALUES ($1, $2, $3, $4)
            "#,
            user_id,
            email,
            "hashed_password",
            "Test User"
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_create_credit_transaction() {
        let pool = setup_test_db().await;
        let user_id = Uuid::new_v4();
        
        create_test_user(&pool, user_id, &format!("test_credit_{}@example.com", user_id)).await;

        let transaction = CreateTransaction {
            amount: BigDecimal::from_str("100.50").unwrap(),
            transaction_type: TransactionType::Credit,
            description: Some("Test credit".to_string()),
        };

        let result = create_transaction(
            State(pool.clone()),
            Path(user_id),
            Json(transaction),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.0.amount, BigDecimal::from_str("100.50").unwrap());
        assert_eq!(response.0.transaction_type, TransactionType::Credit);

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_create_debit_transaction() {
        let pool = setup_test_db().await;
        let user_id = Uuid::new_v4();
        
        create_test_user(&pool, user_id, &format!("test_debit_{}@example.com", user_id)).await;

        // Create initial credit
        let credit = CreateTransaction {
            amount: BigDecimal::from_str("200.00").unwrap(),
            transaction_type: TransactionType::Credit,
            description: Some("Initial deposit".to_string()),
        };

        let _ = create_transaction(
            State(pool.clone()),
            Path(user_id),
            Json(credit),
        )
        .await
        .unwrap();

        // Create debit
        let debit = CreateTransaction {
            amount: BigDecimal::from_str("50.25").unwrap(),
            transaction_type: TransactionType::Debit,
            description: Some("Test debit".to_string()),
        };

        let result = create_transaction(
            State(pool.clone()),
            Path(user_id),
            Json(debit),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.0.amount, BigDecimal::from_str("50.25").unwrap());
        assert_eq!(response.0.transaction_type, TransactionType::Debit);

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_get_transactions() {
        let pool = setup_test_db().await;
        let user_id = Uuid::new_v4();
        
        create_test_user(&pool, user_id, &format!("test_transactions_{}@example.com", user_id)).await;

        // Create test transactions
        let transactions = vec![
            CreateTransaction {
                amount: BigDecimal::from_str("100.50").unwrap(),
                transaction_type: TransactionType::Credit,
                description: Some("First credit".to_string()),
            },
            CreateTransaction {
                amount: BigDecimal::from_str("25.75").unwrap(),
                transaction_type: TransactionType::Debit,
                description: Some("First debit".to_string()),
            },
        ];

        for transaction in transactions {
            let _ = create_transaction(
                State(pool.clone()),
                Path(user_id),
                Json(transaction),
            )
            .await
            .unwrap();
        }

        let result = get_transactions(State(pool.clone()), Path(user_id)).await;
        assert!(result.is_ok());
        
        let transactions = result.unwrap();
        assert_eq!(transactions.0.len(), 2);
        // Check that both transactions exist, regardless of order
        let mut amounts: Vec<BigDecimal> = transactions.0.iter().map(|t| t.amount.clone()).collect();
        amounts.sort();
        let mut expected = vec![BigDecimal::from_str("25.75").unwrap(), BigDecimal::from_str("100.50").unwrap()];
        expected.sort();
        assert_eq!(amounts, expected);

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_get_balance() {
        let pool = setup_test_db().await;
        let user_id = Uuid::new_v4();
        
        create_test_user(&pool, user_id, &format!("test_balance_{}@example.com", user_id)).await;

        // Create test transactions
        let transactions = vec![
            CreateTransaction {
                amount: BigDecimal::from_str("100.50").unwrap(),
                transaction_type: TransactionType::Credit,
                description: Some("First credit".to_string()),
            },
            CreateTransaction {
                amount: BigDecimal::from_str("25.75").unwrap(),
                transaction_type: TransactionType::Debit,
                description: Some("First debit".to_string()),
            },
        ];

        for transaction in transactions {
            let _ = create_transaction(
                State(pool.clone()),
                Path(user_id),
                Json(transaction),
            )
            .await
            .unwrap();
        }

        let result = get_account_balance(State(pool.clone()), Path(user_id)).await;
        assert!(result.is_ok());
        
        let balance = result.unwrap();
        assert_eq!(balance.0.balance, BigDecimal::from_str("74.75").unwrap());

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_invalid_user_id() {
        let pool = setup_test_db().await;
        let invalid_user_id = Uuid::new_v4();

        let transaction = CreateTransaction {
            amount: BigDecimal::from_str("100.50").unwrap(),
            transaction_type: TransactionType::Credit,
            description: Some("Test credit".to_string()),
        };

        let result = create_transaction(
            State(pool),
            Path(invalid_user_id),
            Json(transaction),
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.0, StatusCode::NOT_FOUND);
    }
} 