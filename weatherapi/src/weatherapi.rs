use serde::{Deserialize, Serialize};
pub const WEATHERAPI_BASE_URL: &str = "https://api.weatherapi.com/v1/";

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ForecastResponse {
    pub location: Location,
    pub current: Current,
    pub forecast: Forecast,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Location {
    name: String,
    region: String,
    country: String,
    localtime: String,
    localtime_epoch: u64,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Current {
    last_updated_epoch: u64,
    last_updated: String,
    condition: Condition,
    temp_c: f32,
    feelslike_c: f32,
    pressure_mb: f32,
    precip_mm: f32,
    humidity: u32,
    cloud: u32,
    wind_kph: f32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Condition {
    text: String,
    icon: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Forecast {
    forecastday: Vec<ForecastDay>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ForecastDay {
    date: String,
    date_epoch: u64,
    day: Day,
    hour: Vec<Hour>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Day {
    maxtemp_c: f32,
    mintemp_c: f32,
    totalprecip_mm: f32,
    avghumidity: f32,
    daily_will_it_rain: u8, // 1:true, 0, false
    daily_chance_of_rain: u8,
    daily_will_it_snow: u8, // 1:true, 0, false
    daily_chance_of_snow: u8,
    condition: Condition,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Hour {
    time_epoch: u64,
    time: String,
    temp_c: f32,
    feelslike_c: f32,
    pressure_mb: f32,
    precip_mm: f32,
    humidity: u32,
    cloud: u32,
    will_it_rain: u8, // 1:true, 0, false
    chance_of_rain: u8,
    will_it_snow: u8, // 1:true, 0, false
    chance_of_snow: u8,
}

use aide_proto::v1::weather::CurrentWeather;
use std::fmt::Write as FmtWrite;
impl From<ForecastResponse> for CurrentWeather {
    fn from(fr: ForecastResponse) -> Self {
        let mut location = String::new();
        write!(location, "{} ({})", fr.location.name, fr.location.country).unwrap();

        CurrentWeather {
            location,
            description: fr.current.condition.text,
            temp_c: fr.current.temp_c,
            feelslike_c: fr.current.feelslike_c,
            pressure_mb: fr.current.pressure_mb,
            precip_mm: fr.current.precip_mm,
        }
    }
}

use aide_proto::v1::weather::Forecast as AideForecast;
impl From<ForecastResponse> for AideForecast {
    fn from(fr: ForecastResponse) -> Self {
        let mut location = String::new();
        write!(location, "{} ({})", fr.location.name, fr.location.country).unwrap();
        let mut days = fr.forecast.forecastday;
        let tomorrow = days.pop().unwrap();
        let today = days.pop().unwrap();
        let current_epoch = fr.current.last_updated_epoch;
        let tomorrow_epoch = tomorrow.date_epoch;
        if tomorrow_epoch < current_epoch || (tomorrow_epoch - current_epoch) < 60 * 60 * 8 {
            get_aideforecast(location, tomorrow.date, tomorrow.day)
        } else {
            get_aideforecast(location, today.date, today.day)
        }
    }
}

fn get_aideforecast(location: String, date: String, day: Day) -> AideForecast {
    AideForecast {
        location,
        description: day.condition.text,
        time: date,
        mintemp_c: day.mintemp_c,
        maxtemp_c: day.maxtemp_c,
        precip_mm: day.totalprecip_mm,
        chance_of_rain: day.daily_chance_of_rain,
        chance_of_snow: day.daily_chance_of_snow,
    }
}

use aide_proto::v1::weather::{HourRainForecast, RainForecast};
impl From<ForecastResponse> for RainForecast {
    fn from(fr: ForecastResponse) -> Self {
        let mut location = String::new();
        write!(location, "{} ({})", fr.location.name, fr.location.country).unwrap();
        let current_epoch = fr.location.localtime_epoch;
        let mut days = fr.forecast.forecastday;
        let tomorrow = days.pop().unwrap();
        let today = days.pop().unwrap();
        let mut hours: Vec<HourRainForecast> = today
            .hour
            .iter()
            .filter(|x| x.time_epoch > current_epoch || current_epoch - x.time_epoch < 3600)
            .filter(|x| {
                x.will_it_rain == 1 /* yes */
                || x.chance_of_rain > 30
            })
            .map(HourRainForecast::from)
            .collect();
        hours.append(
            &mut tomorrow
                .hour
                .iter()
                .filter(|x| {
                    x.will_it_rain == 1 /* yes */
                    || x.chance_of_rain > 30
                })
                .map(HourRainForecast::from)
                .collect::<Vec<HourRainForecast>>(),
        );
        RainForecast {
            location,
            hour_rain_forecast: hours,
        }
    }
}

impl From<&Hour> for HourRainForecast {
    fn from(h: &Hour) -> Self {
        HourRainForecast {
            time: h.time.clone(),
            temp_c: h.temp_c,
            feelslike_c: h.temp_c,
            pressure_mb: h.pressure_mb,
            precip_mm: h.precip_mm,
            chance_of_rain: h.chance_of_rain,
            chance_of_snow: h.chance_of_snow,
        }
    }
}
#[cfg(test)]
mod test {
    #[test]
    fn deserialization() {
        let input = include_bytes!("../resources/forecast.json");
        let got = serde_json::from_slice::<super::ForecastResponse>(input);
        assert!(got.is_ok());
    }

    #[test]
    fn deserialization2() {
        let input = include_bytes!("../resources/forecast2.json");
        let got = serde_json::from_slice::<super::ForecastResponse>(input);
        assert!(got.is_ok());
    }
}
