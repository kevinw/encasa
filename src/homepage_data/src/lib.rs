#[macro_use] extern crate serde_derive;

extern crate time;
extern crate chrono;
extern crate tempfile;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate shellexpand;
extern crate humantime;
#[macro_use] extern crate failure;

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
use std::fs::{File, OpenOptions};
use std::path::{Path};
use std::io::{Read, Write, BufReader, BufRead};
use std::time::SystemTime;
use std::process::Command;

use failure::{ResultExt};

static META_YAML_PATH: &str = "~/homepage.yaml";
static DEADLINES_JSON_PATH: &str = "~/deadlines.json";

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
    pub fn from_local_config() -> Result<HomepageMeta, failure::Error> {
        let path = shellexpand::tilde(META_YAML_PATH);
        Ok(serde_yaml::from_str(&get_file_contents(&path)?)
            .expect(&format!("Couldn't parse YAML at {}", path)))
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
    #[serde(default)] pub name: String,

    pub path: String,

    #[serde(default)] pub todos: bool,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_humantime")]
    pub frequency_goal_seconds: i64,

    #[serde(default)] pub auto_project: String,

    #[serde(default)] pub hide_in_index: bool,

    #[serde(default)] pub git: String,
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
    pub file_is_showing_todos: bool, // TODO this will go away once the view crate is doing the filtering
}

fn run_shell_command(command: &str, working_dir: &str) -> String {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
                .current_dir(working_dir)
                .args(&["/C", command])
                .output()
                .expect("failed to execute process")
    } else {
        Command::new("sh")
                .current_dir(working_dir)
                .arg("-c")
                .arg(command)
                .output()
                .expect("failed to execute process")
    };

    String::from_utf8_lossy(&output.stdout).into_owned()

}

pub fn update_deadlines() -> Result<(), failure::Error> {
    let stdout_text = run_shell_command("python3 scrape_events.py", "/Users/kevin/src/encasa/tools/calscrape/");
    println!("{}", stdout_text);
    lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new(r"saved \d+ events").unwrap();
    }
    if RE.is_match(&stdout_text) {
        Ok(())
    } else {
        Err(format_err!("python command did not return successfully"))
    }
}

fn get_last_commit_date(working_dir: &str) -> std::time::SystemTime {
    // '%at': author date, UNIX timestamp
    // '%aI': author date, strict ISO 8601 format
    let git_command = "git log -1 --pretty=format:%at";
    let s = run_shell_command(git_command, working_dir);
    let secs:u64 = s.parse().unwrap();
    let duration = std::time::Duration::new(secs, 0);
    SystemTime::UNIX_EPOCH + duration
}

impl LocalFileDescWithState {
    pub fn last_modified(&self) -> SystemTime {
        if !self.desc.git.is_empty() {
            get_last_commit_date(&shellexpand::tilde(&self.desc.git).to_string())
        } else {
            assert!(!&self.states.is_empty());
            let last_state = &self.states[&self.states.len() - 1];
            last_state.modification_time
        }
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

impl Deadlines {
    pub fn new() -> Deadlines {
        Deadlines { deadlines: vec![] }
    }
}

impl FileState {
    fn from(metadata: &std::fs::Metadata) -> FileState {
        FileState {
            modification_time: metadata.modified().unwrap(),
            size: metadata.len(),
        }
    }
}

fn _ensure_dir_exists(path_to_dir: &str) -> Result<(), failure::Error>
{
    if Path::new(path_to_dir).exists() {
        let metadata = fs::metadata(path_to_dir)?;
        if !metadata.is_dir() {
            return Err(format_err!("{} is not a directory", path_to_dir));
        }
    } else {
        fs::create_dir(path_to_dir)?;
    }

    Ok(())
}

fn get_file_contents(path: &str) -> Result<String, failure::Error> {
    let mut contents = String::new();
    let f = File::open(path).context(format!("missing file '{}'", path))?;
    BufReader::new(f).read_to_string(&mut contents)?;
    Ok(contents)
}

fn parse_todo_file(path: &str) -> Result<Vec<Task>, failure::Error> {
    let mut tasks:Vec<Task> = vec!();
    let f = File::open(path).context(format!("missing todo file {}", path))?;
    for (num, line) in BufReader::new(f).lines().enumerate() {
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

fn archive_tasks_in_todo_file(path: &str) -> Result<u32, failure::Error> {
    let mut lines:Vec<String> = vec![];
    let mut done_lines:Vec<String> = vec![];
    for line in get_file_contents(path)?.lines() {
        match &mut Task::from_str(&line) {
            Ok(task) if task.finished => {
                done_lines.push(line.into());
            }
            _ => {
                lines.push(line.into());
            }
        }
    }

    if !done_lines.is_empty() {
        // append done lines
        let mut f = OpenOptions::new()
            .append(true)
            .create(true)
            .open(get_done_filename(path)
                  .expect(&format!("couldn't make a done.txt path for {}", path)))?;

        f.write_all(&String::from("\n").into_bytes())?;
        f.write_all(&done_lines.join("\n").into_bytes())?;

        // rewrite todo file
        File::create(&path)?.write_all(&lines.join("\n").into_bytes())?;
    }

    Ok(done_lines.len() as u32)
}

pub fn archive_finished_tasks() -> Result<u32, failure::Error> {
    let mut count:u32 = 0;
    for ref local_file in HomepageMeta::from_local_config()?.local.iter().filter(|&f| f.todos) {
        let path:&str = &shellexpand::tilde(&local_file.path);
        count += archive_tasks_in_todo_file(path)?;
    }

    Ok(count)
}

pub fn mark_todo_completed(hash: &str, finished: bool) -> Result<String, failure::Error> {
    let meta = HomepageMeta::from_local_config()?;
    let mut found_any = false;
    let mut new_hash:String = String::new();

    for ref local_file in meta.local.iter().filter(|&f| f.todos) {
        let path:&str = &shellexpand::tilde(&local_file.path);
        let mut lines:Vec<String> = vec![];
        let mut found = false;
        let mut original_contents = get_file_contents(path)?;
        for (num, line) in original_contents.lines().enumerate() {
            match &mut Task::from_str(&line) {
                Ok(task) => {
                    if task.calc_hash() == hash {
                        task.finished = finished;
                        found = true;
                        new_hash.push_str(&task.calc_hash());
                        lines.push(format!("{}", task));
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
                let backup_dir = shellexpand::tilde("~/.homepage/backups/");
                let mut backup_path = backup_dir.to_string();
                std::fs::create_dir_all(&std::path::Path::new(&backup_dir.to_string())).context(format!("could not create backup path {}", backup_path))?;
                backup_path.push_str(&hasher.finish().to_string());
                let mut f = File::create(&backup_path).context(format!("could not create backup path {}", backup_path))?;
                f.write_all(&original_contents.into_bytes())?;
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
        Err(format_err!("no todo for hash {}", hash))
    }
}

fn update_file_history(path: &str) -> Result<FileStateCache, failure::Error> {
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

fn get_deadlines() -> Result<Deadlines, failure::Error> {
    let deadlines_path = String::from(shellexpand::tilde(DEADLINES_JSON_PATH));

    if !Path::new(&deadlines_path).exists() {
        return Ok(Deadlines::new())
    }

    let mut deadlines:Deadlines = serde_yaml::from_str(&get_file_contents(&deadlines_path)?).expect(
        &format!("Couldn't parse JSON at {}", DEADLINES_JSON_PATH));

    use datetools::DateWhen;
    deadlines.deadlines.retain(
        |d| match DateWhen::for_date(&d.start.to_naive_date().unwrap()) {
            DateWhen::Future => true,
            DateWhen::Today => true,
            DateWhen::Past => false,
        });
    Ok(deadlines)
}

pub fn update_data(files_to_include: &Vec<String>) -> Result<CachedData, failure::Error> {
    let mut todos_count:usize = 0;
    let mut all_todos:Vec<TaskWithContext> = vec![];
    let mut files:Vec<LocalFileDescWithState> = vec![];

    for local_file in &HomepageMeta::from_local_config()?.local {
        let path = local_file.expanded_path();
        let history = update_file_history(&path)?;

        let file_is_showing_todos = local_file.todos && if local_file.hide_in_index {
            files_to_include.contains(&local_file.name)
        } else {
            files_to_include.is_empty() || files_to_include.contains(&local_file.name)
        };

        if file_is_showing_todos {
            let todos = parse_todo_file(&path)?;
            todos_count += todos.iter().filter(|c| !c.finished && c.priority == 0).count();
            for todo in &todos {
                all_todos.push(TaskWithContext {
                    task: todo.clone(),
                    auto_project: local_file.auto_project.clone(),
                });
            }
        }

        let update_state = if local_file.frequency_goal_seconds > 0 {
            assert!(!history.states.is_empty());
            {
                let last_state = &history.states[history.states.len() - 1];
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
            file_is_showing_todos,
        });
    }

    Ok(CachedData {
        last_update: SystemTime::now(),
        todos_count,
        todos: all_todos,
        local_files: files,
        deadlines: get_deadlines()?,
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

pub fn get_done_filename(todo_filename: &str) -> Option<String> {
    if !todo_filename.to_lowercase().ends_with("todo.txt") {
        return None;
    }
    if todo_filename.len() > 8 {
        let prev = todo_filename.chars().nth(todo_filename.len() - 9).unwrap();
        // TODO: use actual path splitting here
        if !(prev == '.' || prev == '\\' || prev == '/') {
            return None;
        }
    }
    Some({
        let mut replacement = String::with_capacity(todo_filename.len());
        replacement.push_str(&todo_filename[0..todo_filename.len() - 8]);
        replacement.push_str("done.txt");
        replacement
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_done_file_names() {
        let checks = vec![
            ("C:\\Users\\Kevin\\Dropbox\\watch.todo.txt", "C:\\Users\\Kevin\\Dropbox\\watch.done.txt"),
            ("C:\\Users\\Kevin\\Dropbox\\TODO.txt", "C:\\Users\\Kevin\\Dropbox\\done.txt"),
            ("TODO.txt", "done.txt"),
            ("foo/bar/meep.TODO.txt", "foo/bar/meep.done.txt")
        ];

        for (ref inp, ref out) in checks {
            let done_filename = get_done_filename(*inp).expect(&format!(
                "couldn't make a done.txt filename from {}", inp));
            assert_eq!(done_filename, *out);
        }

        let not_todos = vec![
            "TODOO.txt",
            "not.todo.abc.txt",
            "foo-todo.txt",
        ];

        for ref inp in not_todos {
            if let Some(_) = get_done_filename(&inp) {
                panic!("expected to fail: {}", inp);
            }
        }
    }

    #[test]
    fn test_archive_todos() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("my.todo.txt");
        let path_as_str = file_path.clone().into_os_string().into_string().unwrap();
        let mut file = File::create(file_path).unwrap();
        writeln!(file, "2015-04-01 +project thing that's not done
x a thing that is done
x another thing that is done
a todo
x 2015-05-01 a third thing that is done
").unwrap();
        let count = archive_tasks_in_todo_file(&path_as_str).expect("error while archiving");
        assert_eq!(count, 3, "expected 3 archived done todos");

        let archived = parse_todo_file(&get_done_filename(&path_as_str).unwrap()).unwrap();
        assert_eq!(archived.len(), 3, "expected 3 archived done todos in the done.txt file");
        assert_eq!(archived[2].subject, "a third thing that is done");

        let todos = parse_todo_file(&path_as_str).unwrap();
        assert_eq!(todos.len(), 2, "expected 2 remaining todos");
        assert_eq!(todos[1].subject, "a todo");
    }
}
