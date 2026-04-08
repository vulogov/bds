extern crate log;
use crate::cmd;
use env_logger::Env;

pub fn setloglevel(c: &cmd::Cli) {
    let env = Env::default().filter_or("BDS_LOG_LEVEL", "error");
    match c.debug {
        0 => {
            env_logger::init_from_env(env);
            log::debug!("Set loglevel from environment");
        }
        1 => {
            let env = Env::default().filter_or("BDS_LOG_LEVEL", "bund_blobstore,bds=info");
            env_logger::init_from_env(env);
            log::debug!("Set loglevel=info");
        }
        2 => {
            let env = Env::default().filter_or("BDS_LOG_LEVEL", "bund_blobstore,bds=debug");
            env_logger::init_from_env(env);
            log::debug!("Set loglevel=debug");
        }
        _ => {
            let env = Env::default().filter_or("BDS_LOG_LEVEL", "bund_blobstore,bds=trace");
            env_logger::init_from_env(env);
            log::debug!("Set loglevel=trace");
        }
    }
    log::debug!("setloglevel::setloglevel() reached")
}
