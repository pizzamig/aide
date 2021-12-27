use clap::Parser;

#[derive(Parser, Clone, Debug)]
pub struct Opt {
    #[clap(flatten)]
    /// Register to aide as todo plugin
    pub common_opt: aide_common::CommonOpt,
    //#[clap(short = 'K', long, hide_env_values = true, env = "WEATHERAPI_API_KEY")]
    #[clap(short = 'K', long, hide_env_values = true, env = "WEATHERAPI_API_KEY")]
    /// The API key to authenticate to the weather api service
    pub key: String,
    #[clap(
        short = 'L',
        long,
        env = "WEATHERAPI_LOCATION",
        default_value = "auto:ip"
    )]
    /// The API key to authenticate to the weather api service
    pub location: String,
}
