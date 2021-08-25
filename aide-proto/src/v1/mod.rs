use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct DataResponse {
    pub data: Vec<String>,
}

pub mod kind;
pub use kind::GetModuleKindResponse;
pub use kind::ModuleKind;

pub mod todo;
pub use todo::Todo;

pub mod weather;
pub use weather::CurrentWeather;
pub use weather::Forecast;
pub use weather::HourRainForecast;
pub use weather::WeatherTypes;
