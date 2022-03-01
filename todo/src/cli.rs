use clap::{ArgEnum, ArgGroup, Parser, Subcommand};
use strum_macros::EnumString;

#[derive(Parser, Clone)]
#[clap(author, version, about)]
#[clap(propagate_version = true)]
pub struct Opt {
    #[clap(short, long, arg_enum)]
    /// Optional parameter to get only one type of todos
    pub todo_type: Option<TodoTypes>,
    /// Specify labels
    #[clap(short, long)]
    pub label: Option<String>,
    #[clap(flatten)]
    pub common_opt: aide_common::CliCommonOpt,
    #[clap(subcommand)]
    pub command: Option<Subcommands>,
}

#[derive(Debug, Clone, PartialEq, EnumString, ArgEnum)]
#[strum(ascii_case_insensitive)]
pub enum TodoTypes {
    Daily,
    Weekly,
    Task,
    Periodic,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Subcommands {
    #[clap(group(ArgGroup::new("label-ops").required(true).args(&["create", "delete"])))]
    /// Manipulate labels
    Label {
        name: String,
        /// Add a new label
        #[clap(short, long)]
        create: bool,
        /// Delete a new label
        #[clap(short, long)]
        delete: bool,
    },
}
