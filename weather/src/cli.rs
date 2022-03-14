use clap::Parser;
use strum_macros::EnumString;

#[derive(Parser, Clone)]
#[clap(author, version, about)]
#[clap(propagate_version = true)]
pub struct Opt {
    /// The type of forecast
    #[clap(short, long, arg_enum, default_value_t = ForecastTypes::Current)]
    pub forecast: ForecastTypes,
    /// Optional paramter to specify a location
    #[clap(short, long)]
    pub location: Option<String>,
    #[clap(flatten)]
    pub common_opt: aide_common::CliCommonOpt,
}

#[derive(Debug, Clone, PartialEq, EnumString, clap::ArgEnum)]
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
