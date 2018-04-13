use askama::Template;
use CachedData;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
    json_dump: &'a str,
    todos_count: usize,
}


pub fn render(cached_data: &CachedData, json_dump: &str) -> String {
    let hello = HelloTemplate {
        name: "world",
        json_dump: json_dump,
        todos_count: cached_data.todos_count,
    };
    hello.render().unwrap()
}

