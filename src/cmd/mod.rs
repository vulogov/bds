extern crate log;
use shadow_rs::shadow;
shadow!(build);

pub mod setloglevel;

use clap::{Args, Parser, Subcommand};
use lazy_static::lazy_static;
use std::env;
use std::sync::Mutex;
use time_graph;

pub mod bds_display_banner;
pub mod bds_serve;
pub mod bds_version;

lazy_static! {
    pub static ref CLI: Mutex<Cli> = {
        let e: Mutex<Cli> = Mutex::new(Cli::parse());
        e
    };
}

fn do_panic() {
    log::debug!("Setting a global panic handler");
    better_panic::Settings::auto()
        .most_recent_first(false)
        .lineno_suffix(true)
        .verbosity(better_panic::Verbosity::Full)
        .install();
}

pub fn main() {
    let cli = Cli::parse();
    setloglevel::setloglevel(&cli);
    do_panic();
    let init_cli = CLI.lock().unwrap();
    log::debug!(
        "BDS server tool version:{}, tag:{}, branch:{}, commit date: {}, commit author:{}({}), commit_id:{}. Build at {}",
        build::VERSION,
        build::TAG,
        build::BRANCH,
        build::COMMIT_DATE,
        build::COMMIT_AUTHOR,
        build::COMMIT_EMAIL,
        build::COMMIT_HASH,
        build::BUILD_TIME
    );
    log::debug!("BUNDCORE version: {}", bundcore::version());
    log::debug!("BLOB STORE version: {}", bund_blobstore::version());
    log::debug!("DEEPTHOUGHT version: {}", deepthought::version());
    log::debug!("Initialize global CLI");
    drop(init_cli);
    log::debug!("BDS server context initialized ...");

    if cli.profile {
        log::debug!("Enable BDS profiler");
        time_graph::enable_data_collection(true);
    }

    let db = match crate::stdlib::DB.read() {
        Ok(db) => db,
        Err(e) => panic!("Unable to read lock database: {}", e),
    };
    drop(db);
    let db = match crate::stdlib::LOGS.read() {
        Ok(db) => db,
        Err(e) => panic!("Unable to read lock logs database: {}", e),
    };
    drop(db);

    match &cli.command {
        Commands::Serve(serve) => {
            bds_serve::run(&cli, serve.clone());
        }
        Commands::Version(_) => {
            bds_version::run(&cli);
        }
    }

    if cli.profile {
        log::debug!("Generating JBUND profiler report");
        let graph = time_graph::get_full_graph();
        println!("{}", graph.as_table());
    }
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    Version(Version),
    Serve(Serve),
}

#[derive(Parser, Clone, Debug)]
#[clap(name = "bds")]
#[clap(author = "Vladimir Ulogov <vladimir@ulogov.us>")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(
    about = "BDS - Universal Data Platform",
    long_about = "Programmatic data storage and analytics engine"
)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Execute internal profiler")]
    pub profile: bool,

    #[clap(help = "Full path to the BDS storage", long)]
    pub store_path: Option<String>,

    #[clap(help = "Full path to the BDS vector storage", long)]
    pub vector_path: Option<String>,

    #[clap(help = "Full path to GGUF chat model", required = true, short, long)]
    pub chat_model: String,

    #[clap(
        help = "Full path to the GGUF vector store embedding model",
        required = true,
        short,
        long
    )]
    pub embed_model: String,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Re-initialize the database")]
    pub new_database: bool,

    #[clap(subcommand, help = "BDS subcommands")]
    command: Commands,
}

#[derive(Args, Clone, Debug)]
#[clap(about = "Start BDS server")]
pub struct Serve {
    #[clap(help = "BIND address for JSON/RPC service", long)]
    pub bind_addr: Option<String>,

    #[clap(help = "Number of threads", long, default_value_t = 4)]
    pub threads: u16,
}

#[derive(Args, Clone, Debug)]
#[clap(about = "Get the version of the BDS")]
pub struct Version {}
