use crate::core::config::{Config, DatabaseConfig};
use crate::core::redis::RedisCache;
use anyhow::{Context, Result};
use sea_orm::{ConnectOptions, DatabaseConnection,  DbErr, Database};
use std::{sync::Arc, time::Duration};
use tracing_subscriber::{fmt::time::FormatTime, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::{rolling};
use chrono::{FixedOffset, Utc};
struct East8Timer;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: Arc<RedisCache>,
    pub config: Arc<Config>,
}

impl FormatTime for East8Timer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let east8 = FixedOffset::east_opt(8 * 3600).unwrap();
        let now = Utc::now().with_timezone(&east8);
        write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}

pub async fn init_db(config: &DatabaseConfig) -> Result<DatabaseConnection, DbErr> {
    tracing::info!("Connecting to database: {}", config.db_url);
    let mut opt = ConnectOptions::new(&config.db_url);
    opt.max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect_timeout(Duration::from_secs(config.connect_timeout_secs))
        .sqlx_logging(true);
    let db = Database::connect(opt).await?;
    Ok(db)
}

pub async fn init_app() -> Result<AppState> {
    // 加载配置
    let config = Config::from_env()
        .map_err(|e| anyhow::anyhow!(format!("config load failed:{}", e.to_string())))?;
    tracing::info!("Configuration loaded successfully");
    // 初始化数据库
    let db = init_db(&config.database)
        .await
        .context("database connection failed")?;
    tracing::info!("Database connected successfully");
    // 初始化 Redis
    let redis = RedisCache::new(&config.redis.url)
        .with_context(|| "redis connection failed")?;
    tracing::info!("Redis connected successfully");
    // 创建应用状态
    let app_state = AppState {
        db,
        redis: Arc::new(redis),
        config: Arc::new(config),
    };
    // 创建路由
    Ok(app_state)
}

pub fn init_log() -> Result<()> {
    // 同时输出到文件和 stdout，并保留 guard 确保文件日志 flush
    let file_appender = rolling::daily("logs", "app.log");
    let (non_blocking_appender, _) = tracing_appender::non_blocking(file_appender);
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info".into());
    let fmt_file = tracing_subscriber::fmt::layer()
        .with_timer(East8Timer)
        .with_ansi(false)
        .with_writer(non_blocking_appender);
    let fmt_stdout = tracing_subscriber::fmt::layer().with_timer(East8Timer);
    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_file)
        .with(fmt_stdout)
        .try_init();
    Ok(())
}
