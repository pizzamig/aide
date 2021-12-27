use clap::Parser;

#[derive(Parser, Clone, Debug)]
pub struct Opt {
    #[clap(flatten)]
    pub common_opt: aide_common::CommonOpt,
}
