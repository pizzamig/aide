mod cli;
mod weatherapi;

use clap::Clap;
use surf_pool::SurfPool;
const WEATHERAPI_PORT: u16 = 9091;

#[derive(Clone, Debug)]
struct State {
    opt: cli::Opt,
    pool: SurfPool,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let opt = cli::Opt::parse();
    tide::log::with_level(tide::log::LevelFilter::Debug);

    let pool = surf_pool::SurfPoolBuilder::new(1).unwrap().build().await;
    let state = State { opt, pool };
    let mut app = tide::with_state(state);
    app.at("/v1/current").get(current);
    app.at("/v1/forecast").get(forecast);
    app.at("/v1/hourrainforecast").get(hour_rain_forecast);
    let binded = format!("0.0.0.0:{}", WEATHERAPI_PORT);
    app.listen(binded).await?;
    Ok(())
}

async fn current(req: tide::Request<State>) -> tide::Result<String> {
    let location_str = req
        .param("location")
        .unwrap_or_else(|_| req.state().opt.location.as_str());
    let base_url = surf::Url::parse(weatherapi::WEATHERAPI_BASE_URL).unwrap();
    let mut forecast_url = base_url.join("forecast.json").unwrap();
    forecast_url
        .query_pairs_mut()
        .append_pair("days", "1")
        .append_pair("aqi", "yes")
        .append_pair("alerts", "yes");
    forecast_url
        .query_pairs_mut()
        .append_pair("key", req.state().opt.key.as_str())
        .append_pair("q", location_str);
    let handler = req.state().pool.get_handler().await.unwrap();
    let mut res = handler.get_client().get(forecast_url).send().await.unwrap();
    drop(handler);
    let resp_forecast: weatherapi::ForecastResponse = res.body_json().await.unwrap();
    //dbg!(&resp_forecast);
    let result: aide_proto::v1::weather::CurrentWeather = From::from(resp_forecast);
    Ok(serde_json::to_string(&result).unwrap())
}

async fn forecast(req: tide::Request<State>) -> tide::Result<String> {
    let location_str = req
        .param("location")
        .unwrap_or_else(|_| req.state().opt.location.as_str());
    let base_url = surf::Url::parse(weatherapi::WEATHERAPI_BASE_URL).unwrap();
    let mut forecast_url = base_url.join("forecast.json").unwrap();
    forecast_url
        .query_pairs_mut()
        .append_pair("days", "2")
        .append_pair("aqi", "yes")
        .append_pair("alerts", "yes");
    forecast_url
        .query_pairs_mut()
        .append_pair("key", req.state().opt.key.as_str())
        .append_pair("q", location_str);
    let handler = req.state().pool.get_handler().await.unwrap();
    let mut res = handler.get_client().get(forecast_url).send().await.unwrap();
    drop(handler);
    let resp_forecast: weatherapi::ForecastResponse = res.body_json().await.unwrap();
    //dbg!(&resp_forecast);
    let result: aide_proto::v1::weather::Forecast = From::from(resp_forecast);
    Ok(serde_json::to_string(&result).unwrap())
}

async fn hour_rain_forecast(req: tide::Request<State>) -> tide::Result<String> {
    let location_str = req
        .param("location")
        .unwrap_or_else(|_| req.state().opt.location.as_str());
    let base_url = surf::Url::parse(weatherapi::WEATHERAPI_BASE_URL).unwrap();
    let mut forecast_url = base_url.join("forecast.json").unwrap();
    forecast_url
        .query_pairs_mut()
        .append_pair("days", "2")
        .append_pair("aqi", "yes")
        .append_pair("alerts", "yes");
    forecast_url
        .query_pairs_mut()
        .append_pair("key", req.state().opt.key.as_str())
        .append_pair("q", location_str);
    let handler = req.state().pool.get_handler().await.unwrap();
    let mut res = handler.get_client().get(forecast_url).send().await.unwrap();
    drop(handler);
    let resp_forecast: weatherapi::ForecastResponse = res.body_json().await.unwrap();
    //dbg!(&resp_forecast);
    let result: Vec<aide_proto::v1::weather::HourRainForecast> = From::from(resp_forecast);
    Ok(serde_json::to_string(&result).unwrap())
}
