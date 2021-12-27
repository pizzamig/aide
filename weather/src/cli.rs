use clap::Parser;
use std::net::IpAddr;
use strum_macros::EnumString;

#[derive(Parser, Clone)]
pub struct Opt {
    /// One between: current, forecast, rain, all (default: current)
    #[clap(short, long)]
    pub forecast: ForecastTypes,
    #[clap(short, long)]
    pub location: Option<String>,
    /// Location of the search
    #[clap(
        name = "server",
        long = "server",
        short = 'S',
        default_value = "127.0.0.1"
    )]
    /// Set the listening IP address (default: 127.0.0.1)
    pub host_addr: IpAddr,
    /// Tcp port of the server (default 9091)
    #[clap(short, long, default_value_t = 9091)]
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum ForecastTypes {
    Current,
    Forecast,
    Rain,
    All,
}

impl Default for ForecastTypes {
    fn default() -> Self {
        ForecastTypes::Current
    }
}
