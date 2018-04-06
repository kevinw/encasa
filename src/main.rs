#[macro_use]
extern crate serde_derive;

extern crate futures;
extern crate gotham;
extern crate hyper;
extern crate mime;
extern crate serde;
extern crate serde_json;

use hyper::{Response, StatusCode};

use gotham::http::response::create_response;
use gotham::state::State;

use std::time::{Duration, SystemTime};

#[derive(Serialize, Deserialize, Debug)]
struct RecordFileModificationTime
{
    modification_time: SystemTime
}

fn get_dummy_data() -> Task
{
    let one_week_seconds = 60 * 60 * 24 * 7;

    let task = Task {
        name: String::from("Journal"),
        frequency_goal: Duration::new(one_week_seconds, 0),
        records: Vec::new(),
    };

    task
}

#[derive(Serialize, Deserialize, Debug)]
struct Record
{
    time: SystemTime,
    note: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Task {
    name: String,
    frequency_goal: Duration,
    records: Vec<Record>,
}

/// Create a `Handler` which is invoked when responding to a `Request`.
///
/// How does a function become a `Handler`?.
/// We've simply implemented the `Handler` trait, for functions that match the signature used here,
/// within Gotham itself.
pub fn say_hello(state: State) -> (State, Response) {
    let task = get_dummy_data();
    let serialized = serde_json::to_string(&task).unwrap();
    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((serialized.into_bytes(), mime::TEXT_PLAIN)),
    );

    (state, res)
}

/// Start a server and call the `Handler` we've defined above for each `Request` we receive.
pub fn main() {
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, || Ok(say_hello))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotham::test::TestServer;

    #[test]
    fn receive_hello_world_response() {
        let test_server = TestServer::new(|| Ok(say_hello)).unwrap();
        let response = test_server
            .client()
            .get("http://localhost")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);

        let body = response.read_body().unwrap();
        assert_eq!(&body[..], b"Hello World!");
    }
}
