use std::time::Duration;

use log::error;
use sqlx::SqlitePool;

use crate::include_query;

pub struct ExpirationCleaner {
    pub interval: Duration,
}

impl ExpirationCleaner {
    pub async fn run(&self, pool: SqlitePool) {
        loop {
            if let Err(err) = sqlx::query(include_query!("clear_expired"))
                .execute(&pool)
                .await
            {
                error!("expired redirections clean failure: {}", err);
            }
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
        if let Err(err) = sqlx::query(include_query!("clear_excess"))
            .bind(self.limit as i64)
            .execute(pool)
            .await
        {
            error!("excess redirections clean failure: {}", err);
        }
    }
}
