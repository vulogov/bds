use crate::stdlib::common::Result;
use bund_blobstore::{BlobStore, DataDistributionManager};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ============================================
// ENHANCED LOG STORAGE WITH PRIMARY-SECONDARY SEPARATION
// ============================================

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    service: String,
    message: String,
    metadata: HashMap<String, String>,
    correlation_id: Option<String>,
    primary: bool, // Primary logs are high-priority, secondary are low-priority
}

// Enhanced log storage with automatic primary/secondary separation
pub struct ManagedLogStore {
    _manager: Arc<DataDistributionManager>,
    primary_store: Mutex<BlobStore>, // High-priority logs (Errors, Critical, specific services)
    secondary_store: Mutex<BlobStore>, // Low-priority logs (Debug, Info, Warn)
}

impl ManagedLogStore {
    pub fn new(manager: Arc<DataDistributionManager>, path: &str) -> Result<Self> {
        let primary_store = BlobStore::open(&format!("{}_primary", path))?;
        let secondary_store = BlobStore::open(&format!("{}_secondary", path))?;
        Ok(Self {
            _manager: manager,
            primary_store: Mutex::new(primary_store),
            secondary_store: Mutex::new(secondary_store),
        })
    }

    // Automatic classification: determines if log should go to primary or secondary storage
    pub fn is_primary_log(&self, log: &LogEntry) -> bool {
        // Primary logs are:
        // 1. Critical or Error level logs
        // 2. Logs explicitly marked as primary
        // 3. Logs from critical services
        let critical_services = vec!["database", "payment-processor", "auth-service"];

        log.primary
            || matches!(log.level, LogLevel::Error | LogLevel::Critical)
            || critical_services.contains(&log.service.as_str())
    }

    pub fn ingest(&self, log: LogEntry) -> Result<()> {
        let timestamp = log.timestamp.timestamp_nanos_opt().unwrap_or(0);
        let key = format!("log:{}:{}:{}", log.service, timestamp, uuid::Uuid::new_v4());
        let data = serde_json::to_vec(&log)?;

        // Route to appropriate store based on priority
        if self.is_primary_log(&log) {
            self.primary_store
                .lock()
                .unwrap()
                .put(&key, &data, Some("primary_logs"))?;
        } else {
            self.secondary_store
                .lock()
                .unwrap()
                .put(&key, &data, Some("secondary_logs"))?;
        }

        Ok(())
    }

    pub fn query_by_service(
        &self,
        service: &str,
        limit: usize,
        include_secondary: bool,
    ) -> Result<Vec<LogEntry>> {
        let prefix = format!("log:{}:", service);
        let mut logs: Vec<LogEntry> = Vec::new();

        // Query primary store
        let primary_keys = self.primary_store.lock().unwrap().list_keys()?;
        for key in primary_keys {
            if logs.len() >= limit {
                break;
            }
            if key.starts_with(&prefix) {
                if let Some(data) = self.primary_store.lock().unwrap().get(&key)? {
                    if let Ok(log) = serde_json::from_slice(&data) {
                        logs.push(log);
                    }
                }
            }
        }

        // Query secondary store if requested
        if include_secondary && logs.len() < limit {
            let secondary_keys = self.secondary_store.lock().unwrap().list_keys()?;
            for key in secondary_keys {
                if logs.len() >= limit {
                    break;
                }
                if key.starts_with(&prefix) {
                    if let Some(data) = self.secondary_store.lock().unwrap().get(&key)? {
                        if let Ok(log) = serde_json::from_slice(&data) {
                            logs.push(log);
                        }
                    }
                }
            }
        }

        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(logs)
    }

    pub fn query_by_level(&self, level: LogLevel, limit: usize) -> Result<Vec<LogEntry>> {
        let mut logs: Vec<LogEntry> = Vec::new();

        // Primary logs are more important, check them first
        let primary_keys = self.primary_store.lock().unwrap().list_keys()?;
        for key in primary_keys {
            if logs.len() >= limit {
                break;
            }
            if let Some(data) = self.primary_store.lock().unwrap().get(&key)? {
                if let Ok(log) = serde_json::from_slice::<LogEntry>(&data) {
                    if log.level == level {
                        logs.push(log);
                    }
                }
            }
        }

        // Check secondary store if we need more
        if logs.len() < limit {
            let secondary_keys = self.secondary_store.lock().unwrap().list_keys()?;
            for key in secondary_keys {
                if logs.len() >= limit {
                    break;
                }
                if let Some(data) = self.secondary_store.lock().unwrap().get(&key)? {
                    if let Ok(log) = serde_json::from_slice::<LogEntry>(&data) {
                        if log.level == level {
                            logs.push(log);
                        }
                    }
                }
            }
        }

        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(logs)
    }

    pub fn get_recent_errors(&self, minutes: i64) -> Result<Vec<LogEntry>> {
        let cutoff = Utc::now() - Duration::minutes(minutes);
        let mut errors: Vec<LogEntry> = Vec::new();

        // Check primary store first (most likely to have errors)
        let primary_keys = self.primary_store.lock().unwrap().list_keys()?;
        for key in primary_keys {
            if let Some(data) = self.primary_store.lock().unwrap().get(&key)? {
                if let Ok(log) = serde_json::from_slice::<LogEntry>(&data) {
                    if log.timestamp >= cutoff
                        && matches!(log.level, LogLevel::Error | LogLevel::Critical)
                    {
                        errors.push(log);
                    }
                }
            }
        }

        errors.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(errors)
    }

    pub fn get_primary_secondary_stats(&self) -> Result<(usize, usize)> {
        let primary_count = self.primary_store.lock().unwrap().list_keys()?.len();
        let secondary_count = self.secondary_store.lock().unwrap().list_keys()?.len();
        Ok((primary_count, secondary_count))
    }

    pub fn get_primary_logs(&self, limit: usize) -> Result<Vec<LogEntry>> {
        let mut logs: Vec<LogEntry> = Vec::new();
        let keys = self.primary_store.lock().unwrap().list_keys()?;

        for key in keys.iter().take(limit) {
            if let Some(data) = self.primary_store.lock().unwrap().get(key)? {
                if let Ok(log) = serde_json::from_slice(&data) {
                    logs.push(log);
                }
            }
        }

        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(logs)
    }

    pub fn get_secondary_logs(&self, limit: usize) -> Result<Vec<LogEntry>> {
        let mut logs: Vec<LogEntry> = Vec::new();
        let keys = self.secondary_store.lock().unwrap().list_keys()?;

        for key in keys.iter().take(limit) {
            if let Some(data) = self.secondary_store.lock().unwrap().get(key)? {
                if let Ok(log) = serde_json::from_slice(&data) {
                    logs.push(log);
                }
            }
        }

        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(logs)
    }
}
