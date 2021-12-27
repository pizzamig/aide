mod cli;

use clap::Parser;

const BASE_URL: &str = "http://localhost:9091/v1/";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
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

async fn current(location: Option<String>) -> Result<(), anyhow::Error> {
    let base_url = reqwest::Url::parse(BASE_URL)?;
    let url = match location {
        Some(q) => base_url.join("current/")?.join(&q)?,
        None => base_url.join("current")?,
    };
    let res = reqwest::get(url).await?;
    let cw: aide_proto::v1::weather::CurrentWeather = res.json().await?;
    println!("{}", cw.location);
    println!("{}", cw.description);
    println!(
        "{} C ({} C)\t{} mm\t{} mb",
        cw.temp_c, cw.feelslike_c, cw.precip_mm, cw.pressure_mb
    );
    Ok(())
}

async fn forecast(location: Option<String>) -> Result<(), anyhow::Error> {
    let url = match location {
        Some(q) => reqwest::Url::parse(BASE_URL)?.join("forecast/")?.join(&q)?,
        None => reqwest::Url::parse(BASE_URL)?.join("forecast")?,
    };
    let res = reqwest::get(url).await?;
    let cf: aide_proto::v1::weather::Forecast = res.json().await?;
    println!("{}", cf.location);
    println!("{}\t{}", cf.time, cf.description);
    println!(
        "{}/{}\t{} mm (prob: {}%)",
        cf.mintemp_c, cf.maxtemp_c, cf.precip_mm, cf.chance_of_rain
    );
    Ok(())
}

async fn rain(location: Option<String>) -> Result<(), anyhow::Error> {
    let url = match location {
        Some(q) => reqwest::Url::parse(BASE_URL)?
            .join("hourrainforecast/")?
            .join(&q)?,
        None => reqwest::Url::parse(BASE_URL)?.join("hourrainforecast")?,
    };
    let res = reqwest::get(url).await?;
    let rf: aide_proto::v1::weather::RainForecast = res.json().await?;
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
