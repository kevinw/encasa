#[macro_use] extern crate serde_derive;

extern crate time;
extern crate chrono;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate shellexpand;
extern crate humantime;

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate nom;
extern crate regex;

pub mod todo;
pub mod gcal;
pub mod datetools;

pub use chrono::NaiveDate as Date;

use todo::Task;
use std::str::FromStr;
use std::fs;
use std::fs::File;
use std::path::{Path};
use std::io::{Read, Write, BufReader, BufRead};
use std::time::SystemTime;

static META_YAML_PATH: &str = "c:\\Users\\kevin\\homepage.yaml";
static DEADLINES_JSON_PATH: &str = "c:\\Users\\kevin\\deadlines.json";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FileState
{
    pub modification_time: SystemTime,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct HomepageMeta {
    local: Vec<LocalFileDesc>,
}

impl HomepageMeta {
    pub fn from_local_config() -> Result<HomepageMeta, std::io::Error> {
        Ok(serde_yaml::from_str(&get_file_contents(META_YAML_PATH)?)
            .expect(&format!("Couldn't parse YAML at {}", META_YAML_PATH)))
    }
}

fn seconds_from_humantime(s: &str) -> i64 {
    let duration = humantime::parse_duration(&s).unwrap();
    let seconds = duration.as_secs();
    seconds as i64
}

use serde::de::{Deserialize, Deserializer};
fn deserialize_humantime<'a, D>(deserializer: D) -> std::result::Result<i64, D::Error>
    where D: Deserializer<'a>
{
    let s = String::deserialize(deserializer)?;
    Ok(seconds_from_humantime(&s))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalFileDesc {
    #[serde(default)] name: String,
    pub path: String,
    #[serde(default)] todos: bool,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_humantime")]
    pub frequency_goal_seconds: i64,

    #[serde(default)]
    pub auto_project: String,
}


impl LocalFileDesc {
    pub fn expanded_path(&self) -> String {
        shellexpand::tilde(&self.path).to_string()
    }

    pub fn readable_name(&self) -> &str {
        if self.name.is_empty() {
            &self.path
        } else {
            &self.name
        }
    }
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
    pub desc: LocalFileDesc,
    pub states: Vec<FileState>,
    pub update_state: UpdateState,
}

impl LocalFileDescWithState {
    pub fn last_modified(&self) -> SystemTime {
        assert!(!&self.states.is_empty());
        let last_state = &self.states[&self.states.len() - 1];
        last_state.modification_time
    }

    pub fn duration_since_modified(&self) -> std::time::Duration {
        SystemTime::now().duration_since(self.last_modified()).unwrap()
    }

    pub fn needs_update(&self) -> bool {
        match self.update_state {
            UpdateState::NeedsUpdate => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct TaskWithContext {
    pub task:Task,
    pub auto_project:String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CachedData {
    pub last_update: SystemTime,
    pub todos_count: usize,
    pub todos: Vec<TaskWithContext>,
    pub local_files: Vec<LocalFileDescWithState>,
    pub deadlines: Deadlines,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Deadlines {
    pub deadlines: Vec<gcal::Event>,
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

pub fn archive_finished_tasks() -> Result<i32, std::io::Error> {
    Ok(0)
}

pub fn mark_todo_completed(hash: &str, finished: bool) -> Result<String, std::io::Error> {
    let meta = HomepageMeta::from_local_config()?;
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

        const MAX_ELEMS:usize = 200;
        if history.states.len() > MAX_ELEMS {
            let num_to_drain = history.states.len() - MAX_ELEMS;
            history.states.drain(0..num_to_drain);
        }

        let new_yaml = serde_yaml::to_string(&history)
            .expect("could not convert FileStateCache to YAML").into_bytes();

        File::create(&meta_path_str)?.write_all(&new_yaml)
            .expect(&format!("could not write to {}", meta_path_str));
    }

    Ok(history)
}

fn get_deadlines() -> Result<Deadlines, std::io::Error> {
    let deadlines:Deadlines = serde_yaml::from_str(&get_file_contents(DEADLINES_JSON_PATH)?).expect(
        &format!("Couldn't parse JSON at {}", DEADLINES_JSON_PATH));
    Ok(deadlines)
}

pub fn update_data() -> Result<CachedData, ::std::io::Error> {
    let mut todos_count:usize = 0;
    let mut all_todos:Vec<TaskWithContext> = vec![];

    let deadlines = get_deadlines()?;

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
                all_todos.push(TaskWithContext {
                    task: todo.clone(),
                    auto_project: local_file.auto_project.clone(),
                });
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
            update_state,
        });
    }

    Ok(CachedData {
        last_update: SystemTime::now(),
        todos_count,
        todos: all_todos,
        local_files: files,
        deadlines,
    })
}


impl TaskWithContext {
    pub fn should_show_auto_project(&self) -> bool {
        !self.auto_project.is_empty() && !self.task.projects.contains(&self.auto_project)
    }

    pub fn subject_with_auto_project(&self) -> String {
        if self.should_show_auto_project() {
            let mut s = String::with_capacity(1 + self.auto_project.len() + 1 + self.task.subject.len());
            s.push_str("+");
            s.push_str(&self.auto_project);
            s.push_str(" ");
            s.push_str(&self.task.subject);
            s
        } else {
            self.task.subject.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}