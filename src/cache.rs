use std::collections::HashMap;
use chrono::{NaiveDateTime, Utc};
use serde_json::Value;
use anyhow::Result;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheEntry {
    key: String,
    data: String,
    created_at: NaiveDateTime,
    expires_at: NaiveDateTime,
    created_by: String,
}

impl CacheEntry {
    pub fn new(key: String, data: String, expires_at: NaiveDateTime, created_by: String) -> Self {
        Self { key, data, created_at: Utc::now().naive_utc(), expires_at, created_by }
    }
}

impl CacheEntry {
    /// Current age of cache entry in milliseconds
    pub fn current_age(&self) -> i64 {
        Utc::now().naive_utc().timestamp_millis() - self.created_at.timestamp_millis()
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().naive_utc() > self.expires_at
    }

    /// Max allowed age of entry in milliseconds
    pub fn ttl(&self) -> i64 {
        self.expires_at.timestamp_millis() - self.created_at.timestamp_millis()
    }

    pub fn as_json(&self) -> Result<Value> {
        Ok(serde_json::from_str(&self.data)?)
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct Cache {
    pub cache: HashMap<String, CacheEntry>
}

impl Cache {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Cache {
    pub fn insert(&mut self, entry: CacheEntry) {
        self.cache.insert(entry.key.clone(), entry);
    }

    pub fn remove_expired(&mut self) {
        let mut keys = vec![];
        for (key, entry) in &self.cache {
            if entry.is_expired() {
                keys.push(key.clone());
            }
        }

        for key in keys {
            self.cache.remove(&key);
        }
    }

    pub fn clear(&mut self, key: &str) {
        self.cache.remove(key);
    }

    pub fn get(&mut self, key: &str) -> Option<&CacheEntry> {
        self.remove_expired();
        self.cache.get(key)
    }

    pub fn all(&self) -> Result<Value> {
        Ok(serde_json::to_value(&self.cache)?)
    }
}