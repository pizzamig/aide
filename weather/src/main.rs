mod cli;

use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt: cli::Opt = cli::Opt::parse();
    match opt.forecast {
        cli::ForecastTypes::Current => current(opt).await?,
        cli::ForecastTypes::Forecast => forecast(opt).await?,
        cli::ForecastTypes::Rain => rain(opt).await?,
        cli::ForecastTypes::All => {
            current(opt.clone()).await?;
            forecast(opt.clone()).await?;
            rain(opt).await?;
        }
    };
    Ok(())
}

fn get_base_url(opt: &cli::Opt) -> Result<reqwest::Url, anyhow::Error> {
    let proto = opt.common_opt.get_proto_str();
    let base_url = reqwest::Url::parse(&format!(
        "{}://{}:{}/v1/",
        proto, opt.common_opt.host_addr, opt.common_opt.port
    ))?;
    Ok(base_url)
}

async fn current(opt: cli::Opt) -> Result<(), anyhow::Error> {
    let base_url = get_base_url(&opt)?;
    let url = match opt.location {
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

async fn forecast(opt: cli::Opt) -> Result<(), anyhow::Error> {
    let base_url = get_base_url(&opt)?;
    let url = match opt.location {
        Some(q) => base_url.join("forecast/")?.join(&q)?,
        None => base_url.join("forecast")?,
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

async fn rain(opt: cli::Opt) -> Result<(), anyhow::Error> {
    let base_url = get_base_url(&opt)?;
    let url = match opt.location {
        Some(q) => base_url.join("hourrainforecast/")?.join(&q)?,
        None => base_url.join("hourrainforecast")?,
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
