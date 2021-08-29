use clap::Clap;
use strum_macros::EnumString;

#[derive(Clap, Clone)]
pub struct Opt {
    #[clap(short, long)]
    pub forecast: ForecastTyeps,
}

#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum ForecastTyeps {
    Current,
    Forecast,
    Rain,
    All,
}
