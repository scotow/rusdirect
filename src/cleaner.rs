use std::time::Duration;

use sqlx::SqlitePool;

use crate::include_query;

pub struct ExpirationCleaner {
    pub interval: Duration,
}

impl ExpirationCleaner {
    pub async fn run(&self, pool: SqlitePool) {
        loop {
            sqlx::query(include_query!("clear_expired"))
                .execute(&pool)
                .await
                .unwrap();
            tokio::time::sleep(self.interval).await;
        }
    }
}

#[derive(Clone)]
pub struct ExcessCleaner {
    pub limit: u64,
}

impl ExcessCleaner {
    pub async fn run(&self, pool: &SqlitePool) {
        sqlx::query(include_query!("clear_excess"))
            .bind(self.limit as i64)
            .execute(pool)
            .await
            .unwrap();
    }
}
