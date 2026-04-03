extern crate log;
use crate::cmd::Cli;
use crate::cmd::bds_display_banner;

#[time_graph::instrument]
pub fn run(_: &Cli) {
    log::debug!("VERSION::run() reached");
    println!("{}", bds_display_banner::omatrix_banner());
}
