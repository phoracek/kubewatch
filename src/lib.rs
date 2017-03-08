//! # Kubernetes Event Watcher
//!
//! This library serves as a base component for Kubernetes event watching. It allows user to
//! specify which resource should be monitored. Deserialization of events is done via
//! [Serde](https://serde.rs/), thanks to it it is possible to use both dynamic on-the-fly
//! deserialization and also beforehand generated Deserializer for specific structure.
//!
//! ## Example
//! ```rust,no_run
//! extern crate kubewatch;
//! extern crate serde_json;
//!
//! use kubewatch::Events;
//!
//! fn main() {
//!     let cluster = kubewatch::Cluster::new("http://localhost:8080").unwrap();
//!     let events = cluster.events::<serde_json::Value>("pods").unwrap();
//!     for event in events.into_iter() {
//!         println!("{:#?}", event);
//!     }
//! }
//! ```
//! Check for more in `examples/`.

extern crate hyper;
extern crate serde_json;
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate matches;
#[cfg(test)]
#[macro_use]
extern crate serde_derive;

use hyper::client::Client;
use hyper::client::response::Response;
use serde_json::Deserializer;
use serde::Deserialize;
use std::io::{self, Read};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

/// Covers all errors returned by `kubewatch`.
#[derive(Debug)]
pub enum Error {
    /// Failed to parse given URL, check inner `ParseError` for more info.
    InvalidUrl(hyper::error::ParseError),
    /// HTTP request failed (does not apply to non-2xx status), check inner `Error` for more info.
    HttpRequestFailed(hyper::error::Error),
    /// Failed while deserializating an event from JSON to Rust.
    DeserializationFailed(serde_json::Error),
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

/// This trait is used to deserialize input stream and return respective Rust structs.
pub trait Events {
    /// Read monitor of events with given `name` and return them as given `Event` structure.
    fn events<Event>(&self, name: &str) -> Result<Receiver<Result<Event, Error>>, Error>
        where Event: Deserialize + Send + 'static;

    /// Helper which reads a byte iterator, deserializes it and return respective structures.
    fn generator<Event, Iter>(&self, iter: Iter) -> Receiver<Result<Event, Error>>
        where Event: Deserialize + Send + 'static,
              Iter: Iterator<Item = io::Result<u8>> + Send + 'static
    {
        let (tx, rx) = channel();
        let stream = Deserializer::from_iter(iter).into_iter::<Event>();
        thread::spawn(move || for event in stream {
            if let Err(_) = tx.send(event.map_err(Error::DeserializationFailed)) {
                break;
            }
        });
        rx
    }
}

/// Read event monitor from Kubernetes API server.
impl Events for Cluster {
    fn events<Event>(&self, name: &str) -> Result<Receiver<Result<Event, Error>>, Error>
        where Event: Deserialize + Send + 'static
    {
        let path = format!("{}?watch=true", name);
        let bytes = try!(self.get(&path)).bytes();
        Ok(self.generator(bytes))
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

    impl Events for &'static str {
        #[allow(unused_variables)] 
        fn events<Event>(&self, name: &str) -> Result<Receiver<Result<Event, Error>>, Error>
            where Event: Deserialize + Send + 'static
        {
            Ok(self.generator(self.bytes().into_iter().map(|b| Ok(b))))
        }
    }

    #[derive(Deserialize, PartialEq, Eq, Debug)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[test]
    fn events_generator() {
        let mut events = r#"{"x": 1, "y": 2}{"x": 3, "y": 4}"#
            .events::<Point>("points")
            .unwrap()
            .into_iter();
        assert_eq!(events.next().unwrap().unwrap(), Point { x: 1, y: 2 });
        assert_eq!(events.next().unwrap().unwrap(), Point { x: 3, y: 4 });
    }
}