use hyper::{Body, Response, StatusCode};
use std::string::ToString;

pub fn http_404<T: ToString>(s: &T) -> Response<Body> {
    hyper::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from(s.to_string()))
        .unwrap()
}

pub fn healthz() -> Response<Body> {
    hyper::Response::new(Body::from("OK"))
}
