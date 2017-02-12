//! # Kubernetes Event Watcher

extern crate hyper;

#[cfg(test)]
#[macro_use]
extern crate matches;

use hyper::client::Client;
use hyper::client::response::Response;

/// Covers all errors returned by `kubewatch`.
#[derive(Debug)]
pub enum Error {
    /// Failed to parse given URL, check inner `ParseError` for more info.
    InvalidUrl(hyper::error::ParseError),
    /// HTTP request failed (does not apply to non-2xx status), check inner `Error` for more info.
    HttpRequestFailed(hyper::error::Error),
}

/// Represents connection to Kubernetes API server.
#[derive(Debug)]
pub struct Cluster {
    host: hyper::Url,
}

impl Cluster {
    /// Initialize `Cluster` with host address and port (e.g. http://127.0.0.1:8080).
    /// 
    /// ```
    /// let cluster = kubewatch::Cluster::new("http://127.0.0.1:8080").unwrap();
    /// ```
    pub fn new(host: &str) -> Result<Cluster, Error> {
        let url = try!(hyper::Url::parse(host).map_err(Error::InvalidUrl));
        Ok(Cluster { host: url })
    }

    /// Run HTTP GET request on given path (will be joined to `Cluster` URL).
    fn get(&self, path: &str) -> Result<Response, Error> {
        let url = try!(self.host.join(path).map_err(Error::InvalidUrl));
        Client::new().get(url).send().map_err(Error::HttpRequestFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cluster() {
        let cluster = Cluster::new("http://rust-lang.org");
        assert!(cluster.is_ok());
    }

    #[test]
    fn cluster_invalid_url() {
        let cluster = Cluster::new("123.456.789.000");
        assert!(matches!(cluster, Err(Error::InvalidUrl(_))));
    }

    #[test]
    fn cluster_get() {
        let cluster = Cluster::new("http://duckduckgo.com").unwrap();
        let response = cluster.get("/rust");
        assert!(response.is_ok());
    }

    #[test]
    fn cluster_get_invalid_url() {
        let cluster = Cluster::new("http://does.not").unwrap();
        let response = cluster.get("/exist");
        assert!(matches!(response, Err(Error::HttpRequestFailed(_))));
    }
}