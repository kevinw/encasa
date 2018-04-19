use {CachedData, LocalFileDescWithState};
use todo::Task;

use askama::Template;

impl Task {
    fn priority_label(&self) -> String {
        let letters:[&str; 26] = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"];
        if self.priority < 26 {
            format!("({})", letters[self.priority as usize])
        } else if self.priority == 26 {
            "".into()
        } else {
            "(?)".into()
        }
    }
}

mod filters {
    use regex::Regex;
    use ::{askama, std, linkify};
    use time;

    pub fn spanify(d: &std::fmt::Display) -> askama::Result<String> {
        lazy_static! {
            static ref RE: Regex  = Regex::new(r"@\w+").unwrap();
        }
        let s = format!("{}", d);
        let t = RE.replace_all(&s, "<span class=\"todo-context\">$0</span>");
        Ok(String::from(t))
    }

    pub fn humanize_duration(duration: &std::time::Duration) -> askama::Result<String> {
        let d = time::Duration::from_std(*duration).unwrap();
        let days = d.num_days();
        let string = if days > 0 {
            String::from(format!("{} days ago", days))
        } else {
            let hours = d.num_hours();
            if hours > 0 {
                String::from(format!("{} hours ago", hours))
            } else {
                let minutes = d.num_minutes();
                if minutes > 0 {
                    String::from(format!("{} minutes ago", hours))
                } else {
                    let seconds = d.num_seconds();
                    String::from(format!("{} minutes ago", seconds))
                }
            }
        };

        Ok(string)
    }

    pub fn linkify(d: &std::fmt::Display) -> askama::Result<String> {
        let s = format!("{}", d);
        let mut finder = linkify::LinkFinder::new();
        finder.kinds(&[linkify::LinkKind::Url]);
        let mut last = 0;
        let mut result = String::new();
        for link in finder.links(&s) {
            result.push_str(&s[last .. link.start()]);
            result.push_str(&format!("<a href=\"{}\">{}</a>", link.as_str(), link.as_str()));
            last = link.end();
        }
        if last == 0 {
            result = s.to_string();
        } else {
            result.push_str(&s[last ..]);
        }
        Ok(result.to_owned())
    }
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    json_dump: &'a str,
    todos_count: usize,
    local_files: &'a Vec<LocalFileDescWithState>,
    todos: &'a Vec<Task>,
}

pub fn render(cached_data: &CachedData, json_dump: &str) -> String {
    let mut todos_sorted = cached_data.todos.clone();
    todos_sorted.sort_by(|a, b| a.priority.cmp(&b.priority));
    let hello = HelloTemplate {
        json_dump,
        todos_count: cached_data.todos_count,
        local_files: &cached_data.local_files,
        todos: &todos_sorted,
    };
    hello.render().unwrap()
}
