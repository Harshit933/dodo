use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use time::OffsetDateTime;
use bigdecimal::BigDecimal;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: BigDecimal,
    pub transaction_type: TransactionType,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "transaction_type", rename_all = "lowercase")]
pub enum TransactionType {
    Credit,
    Debit,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransaction {
    pub amount: BigDecimal,
    pub transaction_type: TransactionType,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AccountBalance {
    pub user_id: Uuid,
    pub balance: BigDecimal,
    pub last_updated: Option<OffsetDateTime>,
} 