use askama::Template;
use {CachedData, LocalFileDesc};
use todo::Task;
// use std::hash::Hash;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    json_dump: &'a str,
    todos_count: usize,
    local_files: &'a Vec<LocalFileDesc>,
    todos: &'a Vec<Task>,
}

pub fn render(cached_data: &CachedData, json_dump: &str) -> String {
    let hello = HelloTemplate {
        json_dump,
        todos_count: cached_data.todos_count,
        local_files: &cached_data.local_files,
        todos: &cached_data.todos,
    };
    hello.render().unwrap()
}
