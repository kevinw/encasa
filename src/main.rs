#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate askama;
#[macro_use] extern crate hyper;

extern crate time;
extern crate chrono;
extern crate futures;
extern crate gotham;
#[macro_use] extern crate gotham_derive;
extern crate mime;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate shellexpand;
extern crate linkify;

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate nom;
extern crate regex;

mod view;
mod todo;
mod gcal;

use todo::Task;

pub use chrono::NaiveDate as Date;

use futures::{future, Future, Stream};

use std::str::FromStr;
use std::fs;
use std::fs::File;
use std::path::{Path};
use std::io::{Read, Write, BufReader, BufRead};

use gotham::http::response::create_response;
use gotham::router::Router;

use hyper::{Body, Response, StatusCode};

use gotham::state::{FromState, State};
use gotham::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};

use gotham::handler::{HandlerFuture, IntoHandlerError};

use std::time::SystemTime;

header! { (XFrameOptions, "X-Frame-Options") => [String] }

static META_YAML_PATH: &str = "c:\\Users\\kevin\\homepage.yaml";
static DEADLINES_JSON_PATH: &str = "c:\\Users\\kevin\\deadlines.json";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct FileState
{
    modification_time: SystemTime,
    size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct HomepageMeta {
    local: Vec<LocalFileDesc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalFileDesc {
    path: String,
    #[serde(default)] todos: bool,
    #[serde(default)] frequency_goal_seconds: i64,
}

impl LocalFileDesc {
    fn expanded_path(&self) -> String { shellexpand::tilde(&self.path).to_string() }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FileStateCache {
    states: Vec<FileState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UpdateState {
    NoGoal,
    Ok,
    NeedsUpdate,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalFileDescWithState {
    desc: LocalFileDesc,
    states: Vec<FileState>,
    update_state: UpdateState,
}

impl LocalFileDescWithState {
    fn last_modified(&self) -> SystemTime {
        assert!(!&self.states.is_empty());
        let last_state = &self.states[&self.states.len() - 1];
        last_state.modification_time
    }

    fn duration_since_modified(&self) -> std::time::Duration {
        SystemTime::now().duration_since(self.last_modified()).unwrap()
    }

    fn needs_update(&self) -> bool {
        match self.update_state {
            UpdateState::NeedsUpdate => true,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CachedData
{
    last_update: SystemTime,
    todos_count: usize,
    todos: Vec<Task>,
    local_files: Vec<LocalFileDescWithState>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Deadlines {
    deadlines: Vec<gcal::Event>,
}

impl FileState {
    fn from(metadata: &std::fs::Metadata) -> FileState {
        FileState {
            modification_time: metadata.modified().unwrap(),
            size: metadata.len(),
        }
    }
}

fn _ensure_dir_exists(path_to_dir: &str) -> Result<(), std::io::Error>
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
    BufReader::new(File::open(path)?).read_to_string(&mut contents)?;
    Ok(contents)
}

fn parse_todo_file(path: &str) -> Result<Vec<Task>, std::io::Error> {
    let mut tasks:Vec<Task> = vec!();
    for (num, line) in BufReader::new(File::open(path)?).lines().enumerate() {
        match Task::from_str(&line?) {
            Ok(task) => {
                if !task.subject.is_empty() {
                    tasks.push(task);
                }
            },
            Err(_) => {
                eprintln!("ERROR parsing todo in {}:{}", path, num);
            }
        }
    }

    Ok(tasks)
}

fn update_file_history(path: &str) -> Result<FileStateCache, ::std::io::Error> {
    // Create empty YAML file if it's not there.
    let meta_path_str: String;
    {
        let mut meta_path = String::from(path);
        meta_path.push_str(".meta.yaml");
        meta_path_str = meta_path;
    }

    if !Path::new(&meta_path_str).exists() {
        let empty_file_state_cache = FileStateCache { states: vec![] };
        let serialized_bytes = serde_yaml::to_string(&empty_file_state_cache)
            .expect("failed turning fresh FileStateCache to YAML").into_bytes();
        File::create(&meta_path_str)
            .expect(&format!("could not create .metadata.yaml file at {}", meta_path_str))
            .write_all(&serialized_bytes)?;

        println!("created file at {}", &meta_path_str);
    }

    // Now that it's there, parse it
    let contents = &get_file_contents(&meta_path_str)?;
    let mut history : FileStateCache = serde_yaml::from_str(contents)
        .expect(&format!("YAML has invalid structure: '{}'", meta_path_str));

    // Update it if the file has changed.
    let md = std::fs::metadata(path)?;
    let file_state = FileState::from(&md);

    if history.states.is_empty() || history.states[history.states.len() - 1] != file_state {
        history.states.push(file_state);

        let new_yaml = serde_yaml::to_string(&history)
            .expect("could not convert FileStateCache to YAML").into_bytes();

        File::create(&meta_path_str)?.write_all(&new_yaml)
            .expect(&format!("could not write to {}", meta_path_str));
    }

    Ok(history)
}

fn get_events() -> Result<Deadlines, std::io::Error> {
    let deadlines:Deadlines = serde_yaml::from_str(&get_file_contents(DEADLINES_JSON_PATH)?).expect(
        &format!("Couldn't parse JSON at {}", DEADLINES_JSON_PATH));
    Ok(deadlines)
}

fn update_data() -> Result<CachedData, ::std::io::Error> {
    let mut todos_count:usize = 0;
    let mut all_todos:Vec<Task> = vec![];

    let _events = get_events()?;

    let meta: HomepageMeta = serde_yaml::from_str(&get_file_contents(META_YAML_PATH)?)
        .expect(&format!("Couldn't parse YAML at {}", META_YAML_PATH));

    let mut files:Vec<LocalFileDescWithState> = vec![];

    for local_file in &meta.local {
        let path = local_file.expanded_path();
        let history = update_file_history(&path)?;
        if local_file.todos {
            let todos = parse_todo_file(&path)?;
            todos_count += todos.iter().filter(|c| !c.finished).count();
            //println!("{} total todos in {}", todos.len(), path);
            for todo in &todos {
                all_todos.push(todo.clone());
            }
        }

        let update_state = if local_file.frequency_goal_seconds > 0 {
            //println!("frequency goal for {}: {}", path, local_file.frequency_goal_seconds);
            assert!(!history.states.is_empty());
            {
                let last_state = &history.states[history.states.len() - 1];
                //println!("last mod time {:?} for {}", last_state.modification_time, path);
                let diff = SystemTime::now().duration_since(last_state.modification_time).unwrap();
                let seconds = time::Duration::from_std(diff).unwrap().num_seconds();
                if seconds > local_file.frequency_goal_seconds {
                    UpdateState::NeedsUpdate
                } else {
                    UpdateState::Ok
                }
            }
        } else {
            UpdateState::NoGoal
        };

        files.push(LocalFileDescWithState {
            desc: local_file.clone(),
            states: history.states,
            update_state: update_state
        });
    }

    Ok(CachedData {
        last_update: SystemTime::now(),
        todos_count,
        todos: all_todos,
        local_files: files
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotham::test::TestServer;

    #[test]
    fn receive_hello_world_response() {
        let test_server = TestServer::new(|| Ok(index)).unwrap();
        let response = test_server
            .client()
            .get("http://localhost")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);

        let body = response.read_body().unwrap();
        assert!(body.len() > 0);
    }

    #[test]
    fn parse_new_yaml() {
        let cached_data = update_data().expect("update_data failed");
        let _json = serde_json::to_string_pretty(&cached_data).unwrap();
        //println!("{}", json);
    }
}
#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct QueryStringExtractor {
    #[serde(default)] context: String,
    #[serde(default)] project: String,
    #[serde(default)] search: String,
}

fn router() -> Router {
    build_simple_router(|route| {
        route
            .post("/todos")
            .to(post_todos);
        route
            .get("/")
            .with_query_string_extractor::<QueryStringExtractor>()
            .to(index);

    })
}

#[derive(Deserialize)]
struct TodosPost {
    hash: String,
    completed: bool,
}

fn mark_todo_completed(hash: &str, finished: bool) -> Result<String, std::io::Error> {
    let meta: HomepageMeta = serde_yaml::from_str(&get_file_contents(META_YAML_PATH)?)
        .expect(&format!("Couldn't parse YAML at {}", META_YAML_PATH));

    let mut found_any = false;
    let mut new_hash:String = String::new();
    for local_file in &meta.local {
        if !local_file.todos {
            continue;
        }

        let path:&str = &shellexpand::tilde(&local_file.path);
        let mut lines:Vec<String> = vec![];
        let mut found = false;
        let mut original_contents = get_file_contents(path)?;
        for (num, line_res) in original_contents.lines().enumerate() {
            let line = line_res;
            match &mut Task::from_str(&line) {
                Ok(task) => {
                    if task.calc_hash() == hash {
                        task.finished = finished;
                        found = true;
                        new_hash.push_str(&task.calc_hash());
                        let new_task_string = format!("{}", task);
                        println!("new line at {} of {}:\n{}", num, path, new_task_string);
                        lines.push(new_task_string);
                        continue;
                    }
                },
                Err(_) => {
                    eprintln!("ERROR parsing todo in {}:{}", path, num);
                }
            }

            lines.push(String::from(line));
        }

        if found {
            {
                // Write a backup
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                original_contents.hash(&mut hasher);
                let mut backup_path = shellexpand::tilde("~/.homepage/backups/").to_string();
                backup_path.push_str(&hasher.finish().to_string());
                File::create(&backup_path)?.write_all(&original_contents.into_bytes())?;
            }

            File::create(&path)
                .expect(&format!("could not write back {}", path))
                .write_all(&lines.join("\n").into_bytes())?;
        }

        found_any = found_any || found;
    }

    if found_any {
        Ok(new_hash)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, format!("no todo for hash {}", hash)))
    }
}

fn post_todos(mut state: State) -> Box<HandlerFuture> {
    Box::new(Body::take_from(&mut state)
        .concat2()
        .then(|full_body| match full_body {
            Ok(valid_body) => {
                // TODO: instead of .unwrap(), everything should be in a context
                // where server errors become 500 responses.
                let body_content = String::from_utf8(valid_body.to_vec()).unwrap();
                let data: TodosPost = serde_json::from_str(&body_content).unwrap();
                match mark_todo_completed(&data.hash, data.completed) {
                    Ok(new_hash) => {
                        let res = if !new_hash.is_empty() {
                            let mut response_string = String::from("{\"hash\": \"");
                            response_string.push_str(&new_hash);
                            response_string.push_str("\"}");
                            create_response(&state, StatusCode::Ok,
                                Some((response_string.into_bytes(), mime::APPLICATION_JSON)))
                        } else {
                            create_response(&state, StatusCode::NotFound, None)
                        };
                        future::ok((state, res))
                    },
                    Err(msg) => {
                        future::err((state, msg.into_handler_error()))
                    }
                }
            }
            Err(e) => future::err((state, e.into_handler_error())),
        }))
}

pub fn index(mut state: State) -> (State, Response) {
    let cached_data = update_data().unwrap();
    let serialized = serde_json::to_string_pretty(&cached_data).unwrap();
    let html_bytes = view::render(&cached_data, &serialized, &QueryStringExtractor::take_from(&mut state)).into_bytes();
    let mut res = create_response(&state, StatusCode::Ok, Some((html_bytes, mime::TEXT_HTML)));
    res.headers_mut().set(XFrameOptions("ALLOW FROM file://".to_owned()));
    (state, res)
}

pub fn main() {
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router());
}
