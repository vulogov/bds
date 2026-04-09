extern crate log;
use crate::cmd::*;
use crate::cmd::{Cli, Serve};
use crate::stdlib;
use ctrlc;
use jsonrpc_core::*;
use jsonrpc_http_server::*;

fn serve_ctrl_c_handler(server_handle: &mut CloseHandle) {
    log::info!("Received Ctrl+C signal");
    //
    log::debug!("Terminating JSON/RPC server");
    server_handle.clone().close();
    log::debug!("JSON/RPC server terminated. Bye, bye ...");
}

#[time_graph::instrument]
pub fn run(cli: &Cli, serve_args: Serve) {
    log::debug!("SERVE::run() reached");
    let _ = match &cli.database {
        Some(path) => {
            log::debug!("BDS database is expected to be in: {}", &path);
        }
        None => {
            log::error!("Store path not provided");
            return;
        }
    };

    let bind_addr = match serve_args.bind_addr {
        Some(addr) => addr,
        None => "127.0.0.1:8080".to_string(),
    };
    log::debug!("Binding to address: {}", &bind_addr);
    let to_bind = match bind_addr.parse() {
        Ok(addr) => addr,
        Err(err) => {
            log::error!("Invalid bind address: {}", err);
            return;
        }
    };
    let mut io = IoHandler::new();
    stdlib::api::v1::init_api(&mut io);
    let server = ServerBuilder::new(io)
        .threads(serve_args.threads as usize)
        .start_http(&to_bind)
        .expect("Unable to start RPC server");
    log::debug!("JSON/RPC server started");
    let mut server_handle = server.close_handle();
    match ctrlc::set_handler(move || {
        // Set Control-C handler
        serve_ctrl_c_handler(&mut server_handle);
    }) {
        Ok(_) => {
            log::debug!("Ctrl+C handler set successfully");
        }
        Err(err) => {
            log::error!("Failed to set Ctrl+C handler in SERVE: {}", err);
            return;
        }
    }
    server.wait();
}
