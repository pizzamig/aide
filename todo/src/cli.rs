use clap::Clap;
use strum_macros::EnumString;

#[derive(Clap, Clone)]
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
    /// Tcp port to be used (default 9099)
    #[clap(short, long, default_value = "9099")]
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
