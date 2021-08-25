use clap::Clap;

#[derive(Clap, Clone, Debug)]
pub struct Opt {
    #[clap(short = 'R', long)]
    /// Register to aide as todo plugin
    pub registration: bool,
}
