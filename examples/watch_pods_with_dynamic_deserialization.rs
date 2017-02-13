extern crate kubewatch;
extern crate serde_json;

use kubewatch::Events;

fn main() {
    let cluster = kubewatch::Cluster::new("http://localhost:8080").unwrap();
    let events = cluster.events::<serde_json::Value>("pods").unwrap();
    for event in events.into_iter() {
        println!("{:#?}", event);
    }
}
