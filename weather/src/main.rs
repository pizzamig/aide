mod cli;

use clap::Clap;

#[async_std::main]
async fn main() -> surf::Result<()> {
    let opt: cli::Opt = cli::Opt::parse();
    match opt.forecast {
        cli::ForecastTyeps::Current => current().await?,
        cli::ForecastTyeps::Forecast => forecast().await?,
        cli::ForecastTyeps::Rain => rain().await?,
        cli::ForecastTyeps::All => {
            current().await?;
            forecast().await?;
            rain().await?;
        }
    };
    Ok(())
}

async fn current() -> surf::Result<()> {
    let mut res = surf::get("http://localhost:9091/v1/current").await?;
    let cw: aide_proto::v1::weather::CurrentWeather = res.body_json().await?;
    println!("{}", cw.description);
    println!(
        "{} C ({} C)\t{} mm\t{} mb",
        cw.temp_c, cw.feelslike_c, cw.precip_mm, cw.pressure_mb
    );
    Ok(())
}

async fn forecast() -> surf::Result<()> {
    let mut res = surf::get("http://localhost:9091/v1/forecast").await?;
    let cf: aide_proto::v1::weather::Forecast = res.body_json().await?;
    println!("{}\t{}", cf.time, cf.description);
    println!(
        "{}/{}\t{} mm (prob: {}%)",
        cf.mintemp_c, cf.maxtemp_c, cf.precip_mm, cf.chance_of_rain
    );
    Ok(())
}

async fn rain() -> surf::Result<()> {
    let mut res = surf::get("http://localhost:9091/v1/hourrainforecast").await?;
    let hr: Vec<aide_proto::v1::weather::HourRainForecast> = res.body_json().await?;
    hr.iter().take(5).for_each(|h| {
        println!(
            "{}\t{} C,{} mm (prob: {}%)",
            h.time, h.temp_c, h.precip_mm, h.chance_of_rain
        );
    });
    Ok(())
}
