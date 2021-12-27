use clap::Parser;
use std::net::IpAddr;

#[derive(Parser, Clone, Debug)]
pub struct CommonOpt {
    #[clap(short = 'R', long)]
    /// Register to aide as plugin
    pub registration: bool,

    #[clap(name = "host", long = "host", short = 'H', default_value = "127.0.0.1")]
    /// Set the listening IP address (default: 127.0.0.1)
    pub host_addr: IpAddr,

    #[clap(short = 'p', long, default_value_t = 80)]
    /// Listening TCP port of the server (default: 80)
    pub port: u16,
}

#[derive(Parser, Clone, Debug)]
pub struct CliCommonOpt {
    #[clap(
        name = "server",
        long = "Server",
        short = 'S',
        default_value = "127.0.0.1"
    )]
    /// Set the listening IP address (default: 127.0.0.1)
    pub host_addr: String,

    #[clap(short = 'p', long, default_value_t = 80)]
    /// Listening TCP port of the server (default: 80)
    pub port: u16,

    #[clap(long = "no-tls")]
    /// Force http to connect to the server (default: false)
    pub notls: bool,
}
