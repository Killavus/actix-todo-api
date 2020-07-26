use anyhow::{anyhow, Result};
use std::env;
use std::time::Duration;
pub struct PoolConfig {
    url: String,
    max_lifetime: Duration,
    idle_timeout: Duration,
    conn_timeout: Duration,
    pool_size: (u32, u32),
}

use log::{info, warn};

impl PoolConfig {
    pub fn from_env() -> Result<Self> {
        let url = env::var("DATABASE_URL").map_err(|_| anyhow!("DATABASE_URL not provided"))?;

        let max_lifetime = env::var("DATABASE_MAX_LIFETIME")
            .map(|millis| {
                millis
                    .parse()
                    .map_err(|err| warn!("Failed to read max lifetime from env var: {:?}", err))
                    .map(|millis| Duration::from_millis(millis))
                    .unwrap_or_default()
            })
            .unwrap_or(Duration::from_millis(5000));
        let max_pool_size = env::var("DATABASE_POOL_MAX")
            .map(|pool_size| {
                pool_size
                    .parse()
                    .map_err(|err| warn!("Failed to read pool max from env var: {:?}", err))
                    .unwrap_or_default()
            })
            .unwrap_or(16);
        let min_pool_size = env::var("DATABASE_POOL_MIN")
            .map(|pool_size| {
                pool_size
                    .parse()
                    .map_err(|err| warn!("Failed to read pool min from env var: {:?}", err))
                    .unwrap_or_default()
            })
            .unwrap_or(8);
        let idle_timeout = env::var("DATABASE_IDLE_TIMEOUT")
            .map(|millis| {
                millis
                    .parse()
                    .map_err(|err| warn!("Failed to read idle timeout from env var: {:?}", err))
                    .map(|millis| Duration::from_millis(millis))
                    .unwrap_or_default()
            })
            .unwrap_or(Duration::from_millis(10000));
        let conn_timeout = env::var("DATABASE_CONN_TIMEOUT")
            .map(|millis| {
                millis
                    .parse()
                    .map_err(|err| warn!("Failed to read connect timeout from env var: {:?}", err))
                    .map(|millis| Duration::from_millis(millis))
                    .unwrap_or_default()
            })
            .unwrap_or(Duration::from_millis(1000));

        #[cfg(target_feature = "postgres")]
        info!("Database pool initialized (PGSql)");
        #[cfg(not(target_feature = "postgres"))]
        info!("Database pool initialized (SQLite)");
        #[cfg(not(target_feature = "postgres"))]
        info!("Database URL: {}", url);
        info!("Pool size: max {}, min {}", max_pool_size, min_pool_size);
        info!("Idle timeout: {} ms", idle_timeout.as_millis());
        info!("Connect timeout: {} ms", conn_timeout.as_millis());
        info!("Max lifetime of conn: {} ms", max_lifetime.as_millis());

        Ok(Self {
            url,
            max_lifetime,
            idle_timeout,
            conn_timeout,
            pool_size: (max_pool_size, min_pool_size),
        })
    }
}

#[cfg(target_feature = "postgres")]
pub type Pool = sqlx::PgPool;

#[cfg(not(target_feature = "postgres"))]
pub type Pool = sqlx::SqlitePool;

#[cfg(target_feature = "postgres")]
pub type Row = sqlx::postgres::PgRow;

// Right now we are not using this type but it may come very handy in future.
#[allow(dead_code)]
#[cfg(not(target_feature = "postgres"))]
pub type Row = sqlx::sqlite::SqliteRow;

pub async fn pool(config: PoolConfig) -> Result<Pool> {
    Pool::builder()
        .connect_timeout(config.conn_timeout)
        .idle_timeout(config.idle_timeout)
        .max_lifetime(config.max_lifetime)
        .max_size(config.pool_size.0)
        .min_size(config.pool_size.1)
        .build(&config.url)
        .await
        .map_err(|err| anyhow::Error::new(err))
}
