# Kubewatch

[![Build Status](https://travis-ci.org/phoracek/kubewatch.svg?branch=master)](https://travis-ci.org/phoracek/kubewatch)
[![crates.io](https://meritbadge.herokuapp.com/kubewatch)](https://crates.io/crates/kubewatch)
[![docs.rs](https://docs.rs/kubewatch/badge.svg)](https://docs.rs/kubewatch)

This library serves as a base component for Kubernetes event watching. It allows user to
specify which resource should be monitored. Deserialization of events is done via
[Serde](https://serde.rs/), thanks to it it is possible to use both dynamic on-the-fly
deserialization and also beforehand generated Deserializer for specific structure.

[Documentation](https://docs.rs/kubewatch)

## Example

```rust
extern crate kubewatch;
extern crate serde_json;

use kubewatch::Events;

fn main() {
    let cluster = kubewatch::Cluster::new("http://localhost:8080").unwrap();
    let events = cluster.events::<serde_json::Value>("api/v1/pods").unwrap();
    for event in events.into_iter() {
        println!("{:#?}", event);
    }
}
```

Check for more in `examples/`.

## TODO

- namespaces
- TLS/SSL
- filtering
