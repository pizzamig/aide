use serde::{Deserialize, Serialize};
use strum_macros::{EnumString, EnumVariantNames};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, EnumString, EnumVariantNames, Deserialize, Serialize,
)]
#[strum(ascii_case_insensitive)]
pub enum WeatherTypes {
    #[default]
    Current,
    Forecast,
    Rain,
}

#[derive(Debug, Default, PartialEq)]
pub struct CurrentWeather {
    pub description: String,
    pub temp_c: f32,
    pub feelslike_c: f32,
    pub pressure_mb: f32,
    pub precip_mm: f32,
}

#[derive(Debug, Default, PartialEq)]
pub struct Forecast {
    pub description: String,
    pub mintemp_c: f32,
    pub maxtemp_c: f32,
    pub precip_mm: f32,
    pub chance_of_rain: u8,
    pub chance_of_snow: u8,
}

#[derive(Debug, Default, PartialEq)]
pub struct HourRainForecast {
    pub time: String,
    pub temp_c: f32,
    pub feelslike_c: f32,
    pub pressure_mb: f32,
    pub precip_mm: f32,
    pub chance_of_rain: u8,
    pub chance_of_snow: u8,
}
