use jsonrpc_core::*;
use jsonrpc_http_server::*;
use serde_json::{Map, Value};

pub fn init_api(io: &mut IoHandler) {
    log::debug!("Initializing JSON/RPC API version v1");
    io.add_method("v1/ping", |_| {
        log::debug!("Received ping request on JSON/RPC server");
        Ok(Value::String("pong".to_string()))
    });
    io.add_method("v1/version", |_params: Params| {
        let mut result: Map<String, Value> = Map::new();
        result.insert(
            "bds".to_string(),
            Value::String(env!("CARGO_PKG_VERSION").into()),
        );
        result.insert(
            "bundcore".to_string(),
            Value::String(bundcore::version().into()),
        );
        result.insert(
            "rust_dynamic".to_string(),
            Value::String(rust_dynamic::version().into()),
        );
        result.insert(
            "bund_language_parser".to_string(),
            Value::String(bund_language_parser::version().into()),
        );
        result.insert(
            "database_core".to_string(),
            Value::String(bund_blobstore::version().into()),
        );
        result.insert(
            "deepthought".to_string(),
            Value::String(deepthought::version().into()),
        );
        Ok(Value::Object(result))
    });
}
