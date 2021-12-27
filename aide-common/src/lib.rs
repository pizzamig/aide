pub mod cli;
pub use cli::CliCommonOpt;
pub use cli::CommonOpt;

pub mod http;
pub use http::healthz;
pub use http::http_404;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
