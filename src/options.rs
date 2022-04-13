use clap::Parser;
use log::LevelFilter;
use std::time::Duration;

#[derive(Parser, Debug)]
pub struct Options {
    #[clap(short = 'v', long = "verbose", parse(from_occurrences = parse_log_level))]
    pub log_level: LevelFilter,
    #[clap(short = 'd', default_value = "30day" ,parse(try_from_str = humantime::parse_duration))]
    pub expiration: Duration,
    #[clap(short = 'c', default_value = "5min" ,parse(try_from_str = humantime::parse_duration))]
    pub clean_interval: Duration,
    #[clap(short = 'n', default_value = "16384")]
    pub list_size: u64,
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
