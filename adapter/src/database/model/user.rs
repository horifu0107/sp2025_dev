use kernel::model::{id::UserId, role::Role, user::User};
use shared::error::AppError;
use sqlx::types::chrono::{DateTime, Utc};
use std::str::FromStr;

pub struct UserRow {
    pub user_id: UserId,
    pub user_name: String,
    pub email: String,
    pub role_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = AppError;
    fn try_from(value: UserRow) -> Result<Self, Self::Error> {
        let UserRow {
            user_id,
            user_name,
            email,
            role_name,
            ..
        } = value;
        Ok(User {
            user_id: user_id,
            user_name,
            email,
            role: Role::from_str(role_name.as_str())
                .map_err(|e| AppError::ConversionEntityError(e.to_string()))?,
        })
    }
}