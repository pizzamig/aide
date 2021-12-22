pub mod cli;
pub mod http;

pub use cli::CliCommonOpt;
pub use http::http_404;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
