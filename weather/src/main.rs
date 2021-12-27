mod cli;

use clap::Parser;

const BASE_URL: &str = "http://localhost:9091/v1/";

#[async_std::main]
async fn main() -> surf::Result<()> {
    let opt: cli::Opt = cli::Opt::parse();
    match opt.forecast {
        cli::ForecastTypes::Current => current(opt.location).await?,
        cli::ForecastTypes::Forecast => forecast(opt.location).await?,
        cli::ForecastTypes::Rain => rain(opt.location).await?,
        cli::ForecastTypes::All => {
            current(opt.location.clone()).await?;
            forecast(opt.location.clone()).await?;
            rain(opt.location).await?;
        }
    };
    Ok(())
}

async fn current(location: Option<String>) -> surf::Result<()> {
    let base_url = surf::Url::parse(BASE_URL)?;
    let url = match location {
        Some(q) => base_url.join("current/")?.join(&q)?,
        None => base_url.join("current")?,
    };
    let mut res = surf::get(url).await?;
    let cw: aide_proto::v1::weather::CurrentWeather = res.body_json().await?;
    println!("{}", cw.location);
    println!("{}", cw.description);
    println!(
        "{} C ({} C)\t{} mm\t{} mb",
        cw.temp_c, cw.feelslike_c, cw.precip_mm, cw.pressure_mb
    );
    Ok(())
}

async fn forecast(location: Option<String>) -> surf::Result<()> {
    let url = match location {
        Some(q) => surf::Url::parse(BASE_URL)?.join("forecast/")?.join(&q)?,
        None => surf::Url::parse(BASE_URL)?.join("forecast")?,
    };
    let mut res = surf::get(url).await?;
    let cf: aide_proto::v1::weather::Forecast = res.body_json().await?;
    println!("{}", cf.location);
    println!("{}\t{}", cf.time, cf.description);
    println!(
        "{}/{}\t{} mm (prob: {}%)",
        cf.mintemp_c, cf.maxtemp_c, cf.precip_mm, cf.chance_of_rain
    );
    Ok(())
}

async fn rain(location: Option<String>) -> surf::Result<()> {
    let url = match location {
        Some(q) => surf::Url::parse(BASE_URL)?
            .join("hourrainforecast/")?
            .join(&q)?,
        None => surf::Url::parse(BASE_URL)?.join("hourrainforecast")?,
    };
    let mut res = surf::get(url).await?;
    let rf: aide_proto::v1::weather::RainForecast = res.body_json().await?;
    println!("{}", rf.location);

    if rf.hour_rain_forecast.is_empty() {
        println!("No rain expected")
    } else {
        rf.hour_rain_forecast.iter().take(5).for_each(|h| {
            println!(
                "{}\t{} C,{} mm (prob: {}%)",
                h.time, h.temp_c, h.precip_mm, h.chance_of_rain
            );
        });
    }
    Ok(())
}
