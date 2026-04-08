extern crate log;

use crate::cmd::CLI;
use bund_blobstore::{DataDistributionManager, DistributionStrategy};
use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};

pub mod api;
pub mod common;

lazy_static! {
    pub static ref DB: Arc<RwLock<DataDistributionManager>> = {
        let cli = match CLI.lock() {
            Ok(cli) => cli,
            Err(e) => panic!("Unable to lock CLI: {}", e),
        };
        let db_path = match &cli.store_path {
            Some(path) => path,
            None => panic!("No store path specified"),
        };
        let manager = match DataDistributionManager::new(
            format!("{}/blob", &db_path),
            DistributionStrategy::RoundRobin,
        ) {
            Ok(manager) => Arc::new(RwLock::new(manager)),
            Err(err) => panic!("Error init main db: {}", err),
        };
        log::debug!("BDS database initialized in: {}", db_path.clone());
        manager
    };
}

lazy_static! {
    pub static ref LOGS: Arc<RwLock<DataDistributionManager>> = {
        let cli = match CLI.lock() {
            Ok(cli) => cli,
            Err(e) => panic!("Unable to lock CLI: {}", e),
        };
        let db_path = match &cli.store_path {
            Some(path) => path,
            None => panic!("No store path specified"),
        };
        let manager = match DataDistributionManager::new(
            format!("{}/logs", &db_path),
            DistributionStrategy::RoundRobin,
        ) {
            Ok(manager) => Arc::new(RwLock::new(manager)),
            Err(err) => panic!("Error init main db: {}", err),
        };
        log::debug!("BDS database initialized in: {}", db_path.clone());
        manager
    };
}
