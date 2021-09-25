use clap::Clap;
use strum_macros::EnumString;

#[derive(Clap, Clone)]
pub struct Opt {
    /// One between: current, forecast, rain, all
    #[clap(short, long)]
    pub forecast: ForecastTypes,
    #[clap(short, long)]
    pub location: Option<String>,
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
