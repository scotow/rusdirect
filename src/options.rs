use std::net::IpAddr;
use std::time::Duration;

use clap::Parser;
use log::LevelFilter;

#[derive(Parser, Debug)]
pub struct Options {
    #[clap(short = 'v', long = "verbose", parse(from_occurrences = parse_log_level))]
    pub log_level: LevelFilter,
    #[clap(short = 'd', long, default_value = "30day" ,parse(try_from_str = humantime::parse_duration))]
    pub expiration: Duration,
    #[clap(short = 'c', long, default_value = "5min" ,parse(try_from_str = humantime::parse_duration))]
    pub clean_interval: Duration,
    #[clap(short = 'n', long, default_value = "16384")]
    pub list_size: u64,
    #[clap(short = 'a', long, default_value = "127.0.0.1")]
    pub address: IpAddr,
    #[clap(short = 'p', long, default_value = "8080")]
    pub port: u16,
}

fn parse_log_level(n: u64) -> LevelFilter {
    use LevelFilter::*;
    match n {
        0 => Error,
        1 => Warn,
        2 => Info,
        3 => Debug,
        _ => Trace,
    }
}
