mod cli;
mod weatherapi;

use aide_common::{healthz, http_404};
use clap::Parser;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use reqwest_pool::ReqwestPool;

#[derive(Clone, Debug)]
struct State {
    opt: cli::Opt,
    pool: ReqwestPool,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::try_init()
        .unwrap_or_else(|e| eprintln!("Failed to initialize env_logger [{:?}]", e));
    let opt = cli::Opt::parse();
    if opt.common_opt.registration {
        todo!("Registration not implemented yet!")
    }

    let builder = reqwest_pool::ReqwestPoolBuilder::new(1).unwrap();
    let pool = builder.build().await;
    let state = State {
        opt: opt.clone(),
        pool,
    };
    let service = make_service_fn(|_| {
        let cloned_state = state.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                weatherapi_svc(req, cloned_state.clone())
            }))
        }
    });

    let socket_addr = std::net::SocketAddr::new(opt.common_opt.host_addr, opt.common_opt.port);
    let server = Server::bind(&socket_addr).serve(service);
    server.await?;
    Ok(())
}

async fn weatherapi_svc(req: Request<Body>, state: State) -> Result<Response<Body>, anyhow::Error> {
    if req.method() != hyper::Method::GET {
        return Ok(http_404(&"The only method supported is GET"));
    }
    if req.uri().path() == "/healthz" {
        return Ok(healthz());
    }
    if !req.uri().path().starts_with("/v1") {
        return Ok(http_404(&"Invalid path"));
    }
    if req.uri().path().starts_with("/v1/current") {
        return current(req, state).await;
    } else if req.uri().path().starts_with("/v1/forecast") {
        return forecast(req, state).await;
    } else if req.uri().path().starts_with("/v1/hourrainforecast") {
        return hour_rain_forecast(req, state).await;
    }

    Ok(http_404(&""))
}

async fn current(req: Request<Body>, state: State) -> Result<Response<Body>, anyhow::Error> {
    let location_str = if req.uri().path() == "/v1/current" {
        state.opt.location.as_str()
    } else {
        let path = req
            .uri()
            .path()
            .split('/')
            .skip_while(|x| x.is_empty())
            .skip(2)
            .collect::<Vec<_>>();
        if path.is_empty() {
            return Ok(http_404(&"Path has a slash, but no location"));
        }
        path[0]
    };
    let base_url = reqwest::Url::parse(weatherapi::WEATHERAPI_BASE_URL).unwrap();
    let mut forecast_url = base_url.join("forecast.json").unwrap();
    forecast_url
        .query_pairs_mut()
        .append_pair("days", "1")
        .append_pair("aqi", "yes")
        .append_pair("alerts", "yes");
    forecast_url
        .query_pairs_mut()
        .append_pair("key", state.opt.key.as_str())
        .append_pair("q", location_str);
    let handler = state.pool.get_handler().await?;
    let client = handler.get_client();
    let res = client.get(forecast_url).send().await?;
    let resp_forecast: weatherapi::ForecastResponse = res.json().await.unwrap();
    drop(handler);
    //dbg!(&resp_forecast);
    let result: aide_proto::v1::weather::CurrentWeather = From::from(resp_forecast);
    Ok(Response::builder()
        .body(Body::from(serde_json::to_string(&result).unwrap()))
        .unwrap())
}

async fn forecast(req: Request<Body>, state: State) -> Result<Response<Body>, anyhow::Error> {
    let location_str = if req.uri().path() == "/v1/forecast" {
        state.opt.location.as_str()
    } else {
        let path = req
            .uri()
            .path()
            .split('/')
            .skip_while(|x| x.is_empty())
            .skip(2)
            .collect::<Vec<_>>();
        if path.is_empty() {
            return Ok(http_404(&"Path has a slash, but no location"));
        }
        path[0]
    };
    let base_url = reqwest::Url::parse(weatherapi::WEATHERAPI_BASE_URL).unwrap();
    let mut forecast_url = base_url.join("forecast.json").unwrap();
    forecast_url
        .query_pairs_mut()
        .append_pair("days", "2")
        .append_pair("aqi", "yes")
        .append_pair("alerts", "yes");
    forecast_url
        .query_pairs_mut()
        .append_pair("key", state.opt.key.as_str())
        .append_pair("q", location_str);
    let handler = state.pool.get_handler().await?;
    let client = handler.get_client();
    let res = client.get(forecast_url).send().await?;
    let resp_forecast: weatherapi::ForecastResponse = res.json().await.unwrap();
    drop(handler);
    //dbg!(&resp_forecast);
    let result: aide_proto::v1::weather::Forecast = From::from(resp_forecast);
    Ok(Response::builder()
        .body(Body::from(serde_json::to_string(&result).unwrap()))
        .unwrap())
}

async fn hour_rain_forecast(
    req: Request<Body>,
    state: State,
) -> Result<Response<Body>, anyhow::Error> {
    let location_str = if req.uri().path() == "/v1/hourrainforecast" {
        state.opt.location.as_str()
    } else {
        let path = req
            .uri()
            .path()
            .split('/')
            .skip_while(|x| x.is_empty())
            .skip(2)
            .collect::<Vec<_>>();
        if path.is_empty() {
            return Ok(http_404(&"Path has a slash, but no location"));
        }
        path[0]
    };
    let base_url = reqwest::Url::parse(weatherapi::WEATHERAPI_BASE_URL).unwrap();
    let mut forecast_url = base_url.join("forecast.json").unwrap();
    forecast_url
        .query_pairs_mut()
        .append_pair("days", "2")
        .append_pair("aqi", "yes")
        .append_pair("alerts", "yes");
    forecast_url
        .query_pairs_mut()
        .append_pair("key", state.opt.key.as_str())
        .append_pair("q", location_str);
    let handler = state.pool.get_handler().await?;
    let client = handler.get_client();
    let res = client.get(forecast_url).send().await?;
    let resp_forecast: weatherapi::ForecastResponse = res.json().await.unwrap();
    drop(handler);
    //dbg!(&resp_forecast);
    let result: aide_proto::v1::weather::RainForecast = From::from(resp_forecast);
    Ok(Response::builder()
        .body(Body::from(serde_json::to_string(&result).unwrap()))
        .unwrap())
}
