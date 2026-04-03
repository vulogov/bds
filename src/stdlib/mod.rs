extern crate log;

use crate::cmd::CLI;
use bund_blobstore::{CacheConfig, ShardManager, ShardManagerBuilder, ShardingStrategy};
use lazy_static::lazy_static;
use std::sync::RwLock;
use std::time::Duration;

pub mod api;
pub mod common;

lazy_static! {
    pub static ref DB: RwLock<ShardManager> = {
        let cli = match CLI.lock() {
            Ok(cli) => cli,
            Err(e) => panic!("Unable to lock CLI: {}", e),
        };
        let db_path = match &cli.store_path {
            Some(path) => path,
            None => panic!("No store path specified"),
        };
        // Configure cache
        let cache_config = CacheConfig {
            enabled: true,
            max_size: 5000,
            default_ttl: Duration::from_secs(300),
            key_cache_ttl: Duration::from_secs(600),
            time_cache_ttl: Duration::from_secs(300),
        };
        let manager = match ShardManagerBuilder::new()
            .with_strategy(ShardingStrategy::KeyHash)
            .with_cache_config(cache_config)
            .add_shard("shard1", &format!("{}/shard1.bds", &db_path))
            .build() {
                Ok(manager) => RwLock::new(manager),
                Err(e) => panic!("Unable to open database: {}", e),
        };
        log::debug!("BDS database initialized in: {}", db_path.clone());
        manager
    };
}
