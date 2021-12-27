use clap::Parser;
use std::net::IpAddr;
use strum_macros::EnumString;

#[derive(Parser, Clone)]
pub struct Opt {
    //#[clap(short, long)]
    //verbose: bool,
    #[clap(short, long)]
    /// Optional parameter to get only one type of todos
    /// Available types are: daily, weekly, task, periodic
    pub todo_type: Option<TodoTypes>,
    /// Specify labels
    #[clap(short, long)]
    pub label: Option<String>,
    #[clap(
        name = "server",
        long = "server",
        short = 'S',
        default_value = "127.0.0.1"
    )]
    /// Set the listening IP address (default: 127.0.0.1)
    pub host_addr: IpAddr,
    /// Tcp port of the server (default 9099)
    #[clap(short, long, default_value_t = 9099)]
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum TodoTypes {
    Daily,
    Weekly,
    Task,
    Periodic,
}
