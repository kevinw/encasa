#[macro_use]
extern crate serde_derive;

extern crate todo_txt;
 
extern crate futures;
extern crate gotham;
extern crate hyper;
extern crate mime;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

extern crate shellexpand;

use hyper::{Response, StatusCode};
use ::std::str::FromStr;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::io::{Read, Write, BufReader, BufRead};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
enum TaskType {
    FileModification
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TaskDescription
{
    name: String,
    task_type: TaskType,
    path: String,
    frequency_goal: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TaskDescriptionList
{
    tasks: Vec<TaskDescription>,
}

static JOURNAL_PATH: &str = "C:\\Users\\kevin\\Dropbox\\Journal.txt";
static DB_PATH: &str = "c:\\Users\\kevin\\homepage.json";
static DB_DIR: &str = "c:\\Users\\kevin\\.homepage";
static TASKS_JSON_PATH: &str = "c:\\Users\\kevin\\tasks.json";
static META_YAML_PATH: &str = "c:\\Users\\kevin\\homepage.yaml";

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

fn ensure_dir_exists(path_to_dir: &str) -> Result<(), std::io::Error>
{
    if Path::new(path_to_dir).exists() {
        let metadata = fs::metadata(path_to_dir)?;
        if !metadata.is_dir() {
            let message = format!("{} is not a directory", path_to_dir);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, message));
        }
    } else {
        fs::create_dir(path_to_dir)?;
    }

    Ok(())
}

fn get_file_contents(path: &str) -> Result<String, std::io::Error> {
    let mut contents = String::new();
    File::open(path)?.read_to_string(&mut contents)?;
    Ok(contents)
}

fn parse_todo_file(path: &str) -> Result<Vec<::todo_txt::Task>, std::io::Error> {
    let mut tasks:Vec<::todo_txt::Task> = vec!();
    for (num, line) in BufReader::new(File::open(path)?).lines().enumerate() {
        match todo_txt::Task::from_str(&line?) {
            Ok(task) => {
                //println!("    {}", task);
                tasks.push(task);
            }
            Err(_) => {
                eprintln!("    <ERROR parsing todo on line {}>", num);
            }
        }
    }

    Ok(tasks)
}

fn update_task(task: &TaskDescription) -> Result<(), std::io::Error>
{
    let s = serde_json::to_string_pretty(task)?;
    println!("{}", s);

    Ok(())
}

fn maybe_create_dummy_tasks_json() -> Result<(), std::io::Error> {
    if !Path::new(TASKS_JSON_PATH).exists() {
        let one_week_seconds = 60 * 60 * 24 * 7;
        let task_list = TaskDescriptionList {
            tasks: vec![TaskDescription {
                name: String::from("Journal"),
                task_type: TaskType::FileModification,
                path: String::from(JOURNAL_PATH),
                frequency_goal: Duration::new(one_week_seconds, 0),
            }]
        };

        let tasks_json = serde_json::to_string_pretty(&task_list)?.into_bytes();
        File::create(TASKS_JSON_PATH)?.write_all(&tasks_json)?;
    }

    Ok(())
}

fn update() -> Result<(), std::io::Error>
{
    maybe_create_dummy_tasks_json()?;

    let task_list: TaskDescriptionList = 
        serde_json::from_str(&get_file_contents(TASKS_JSON_PATH)?)?;

    ensure_dir_exists(DB_DIR)?;

    for task in task_list.tasks.iter() {
        update_task(task)?;
    }

    Ok(())
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
            Err(_e) => {
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
    match update() {
        Ok(_) => {},
        Err(e) => {
            let err_str = format!("{}", e);
            let error_resp = create_response(
                &state,
                StatusCode::InternalServerError,
                Some((err_str.into_bytes(), mime::TEXT_PLAIN))
            );
            return (state, error_resp);
        }
    }

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
        assert!(body.len() > 0);
        //assert_eq!(&body[..], b"Hello World!");
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct HomepageMeta {
        local: Vec<LocalFileDesc>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct LocalFileDesc {
        id: String,
        path: String,
        #[serde(default)] todos: bool,
        #[serde(default)] frequency_goal_seconds: u64,
    }

    #[test]
    fn parse_new_yaml() {
        fn foo() -> Result<(), ::std::io::Error> {
            let meta: HomepageMeta = serde_yaml::from_str(&get_file_contents(META_YAML_PATH)?).unwrap();

            for local_file in meta.local {
                let path = shellexpand::tilde(&local_file.path);
                if local_file.todos {
                    let todos = parse_todo_file(&path)?;
                    println!("{} total todos in {}", todos.len(), path);
                }
                if local_file.frequency_goal_seconds > 0 {
                    println!("frequency goal for {}: {}", path, local_file.frequency_goal_seconds);
                }
            }

            Ok(())
        }

        foo().unwrap();
    }
}
