#[macro_use]
extern crate serde_derive;
 
extern crate futures;
extern crate gotham;
extern crate hyper;
extern crate mime;
extern crate serde;
extern crate serde_json;

use hyper::{Response, StatusCode};
use std::fs::File;
use std::path::Path;
use std::io::{Read, Write};

use gotham::http::response::create_response;
use gotham::state::State;

use std::time::{Duration, SystemTime};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RecordFileModificationTime
{
    modification_time: SystemTime
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Record
{
    modification_time: SystemTime,
    note: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FileState
{
    modification_time: SystemTime,
    size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FileModificationRecords
{
    file_path: String,
    modification_times: Vec<FileState>,
}

fn record_mod_time(records: &mut FileModificationRecords)
{
    let metadata = std::fs::metadata(records.file_path.clone()).unwrap();

    if Path::new(&records.file_path).exists() {
        records.modification_times.push(FileState {
            modification_time: metadata.modified().unwrap(),
            size: metadata.len(),
        })
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct Task {
    name: String,
    frequency_goal: Duration,
    records: FileModificationRecords,
}

static JOURNAL_PATH: &str = "C:\\Users\\kevin\\Dropbox\\Journal.txt";
static DB_PATH: &str = "c:\\Users\\kevin\\homepage.json";

fn get_default_json() -> String
{
    let one_week_seconds = 60 * 60 * 24 * 7;

    let task = Task {
        name: String::from("Journal"),
        frequency_goal: Duration::new(one_week_seconds, 0),
        records: FileModificationRecords {
            file_path: String::from(JOURNAL_PATH),
            modification_times: Vec::new(),
        },
    };

    serde_json::to_string_pretty(&task).unwrap()
}

fn munge_task(task: Task) -> Task
{
    if task.records.modification_times.len() == 0 {
        println!("empty mod times");

        let mut task2 = task.clone();
        record_mod_time(&mut task2.records);
        return task2
    }

    task
}

fn get_dummy_data() -> Task
{
    let mut contents = String::new();
    {
        match File::open(DB_PATH) {
            Ok(mut f) => {
                f.read_to_string(&mut contents).unwrap();
            }
            Err(e) => {
                contents.push_str(get_default_json().as_str());
            }
        }
    }

    serde_json::from_str(&contents).unwrap()
}


/// Create a `Handler` which is invoked when responding to a `Request`.
///
/// How does a function become a `Handler`?.
/// We've simply implemented the `Handler` trait, for functions that match the signature used here,
/// within Gotham itself.
pub fn say_hello(state: State) -> (State, Response) {
    let task = get_dummy_data();
    let task = munge_task(task);
    let serialized = serde_json::to_string_pretty(&task).unwrap();
    let serialized_bytes = serialized.into_bytes();
    File::create(DB_PATH).unwrap().write_all(&serialized_bytes).unwrap();

    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((serialized_bytes, mime::TEXT_PLAIN)),
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
