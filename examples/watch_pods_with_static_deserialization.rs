extern crate kubewatch;
#[macro_use]
extern crate serde_derive;

use kubewatch::Events;

mod pod {
    #[derive(Deserialize, Debug)]
    pub struct Event {
        #[serde(rename = "type")]
        pub event_type: String,
        pub object: Object,
    }

    #[derive(Deserialize, Debug)]
    pub struct Object {
        pub metadata: Metadata,
    }

    #[derive(Deserialize, Debug)]
    pub struct Metadata {
        pub name: String,
    }
}

fn main() {
    let cluster = kubewatch::Cluster::new("http://localhost:8080").unwrap();
    let events = cluster.events::<pod::Event>("pods").unwrap();
    for event in events.into_iter() {
        println!("{:#?}", event);
    }
}