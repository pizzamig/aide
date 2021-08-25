mod cli;
mod weatherapi;

use clap::Clap;

const WEATHERAPI_PORT: u16 = 9091;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let opt = cli::Opt::parse();
    tide::log::with_level(tide::log::LevelFilter::Debug);

    let mut app = tide::with_state(opt);
    app.at("/v1/current").get(current);
    let binded = format!("0.0.0.0:{}", WEATHERAPI_PORT);
    app.listen(binded).await?;
    Ok(())
}

async fn current(req: tide::Request<cli::Opt>) -> tide::Result<String> {
    let location_str = req
        .param("location")
        .unwrap_or_else(|_| req.state().location.as_str());
    let base_url = surf::Url::parse(weatherapi::WEATHERAPI_BASE_URL).unwrap();
    let mut forecast_url = base_url.join("forecast.json").unwrap();
    forecast_url
        .query_pairs_mut()
        .append_pair("days", "1")
        .append_pair("aqi", "yes")
        .append_pair("alerts", "yes");
    forecast_url
        .query_pairs_mut()
        .append_pair("key", req.state().key.as_str())
        .append_pair("q", location_str);
    let client = surf::Client::new();
    let mut res = client.get(forecast_url).send().await.unwrap();
    let resp_forecast: weatherapi::ForecastResponse = res.body_json().await.unwrap();
    dbg!(resp_forecast);
    Ok("".to_string())
}
