use anyhow::{Context, Result};
use redis::{AsyncCommands, Client, aio::MultiplexedConnection};
use serde::{ Serialize, de::DeserializeOwned};
use std::time::Duration;

#[allow(dead_code)]
#[derive(Clone)]
pub struct RedisCache {
    client: Client,
}

#[allow(dead_code)]
impl RedisCache {
    /// 创建一个新的 RedisCache 实例
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url).with_context(|| "redis connection failed")?;
        Ok(Self { client })
    }

    async fn get_conn(&self) -> Result<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .with_context(|| "redis connection failed")
    }

    /// 值会从 JSON 字符串反序列化
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.get_conn().await.with_context(|| "redis connection failed")?;
        let result: Option<String> = conn.get(key).await.with_context(|| "redis get failed")?;
        match result {
            Some(val_str) => {
                let val: T = serde_json::from_str(&val_str).with_context(|| "deserialization failed")?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }

    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<() > {
        let mut conn = self.get_conn().await.with_context(|| "redis connection failed")?;
        let val_str = serde_json::to_string(value).with_context(|| "serialization failed")?;
        if let Some(duration) = ttl {
            let _: () = conn.set_ex(key, val_str, duration.as_secs() as u64).await.with_context(|| "redis set failed")?;
        } else {
            let _: () = conn.set(key, val_str).await.with_context(|| "redis set failed")?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn del(&self, key: &str) -> Result<()> {
        let mut conn = self.get_conn().await.with_context(|| "redis connection failed")?;
        let _: () = conn.del(key).await.with_context(|| "redis delete failed")?;
        Ok(())
    }
}

