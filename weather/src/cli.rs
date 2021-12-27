use clap::Parser;
use strum_macros::EnumString;

#[derive(Parser, Clone)]
pub struct Opt {
    /// One between: current, forecast, rain, all (default: current)
    #[clap(short, long)]
    pub forecast: ForecastTypes,
    #[clap(short, long)]
    pub location: Option<String>,
    #[clap(flatten)]
    pub common_opt: aide_common::CliCommonOpt,
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
