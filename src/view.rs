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

enum DateWhen {
    Past,
    Future,
    Today,
}

fn duration_relative_to_today(date: &::chrono::NaiveDate) -> ::time::Duration {
    let today_naive = ::chrono::Local::today().naive_local();
    today_naive.signed_duration_since(*date)
}

impl DateWhen {
    fn for_duration(d: ::time::Duration) -> DateWhen {
        if d < ::time::Duration::zero() {
            DateWhen::Future
        } else if d > ::time::Duration::zero() {
            DateWhen::Past
        } else {
            DateWhen::Today
        }
    }

    fn for_date(date: &::chrono::NaiveDate) -> DateWhen {
        let duration = duration_relative_to_today(date);
        DateWhen::for_duration(duration)
    }
}

mod filters {
    use regex::Regex;
    use ::{askama, std, linkify};
    use time;
    use super::DateWhen;
    
    pub fn date_when_css_class(d: &::chrono::NaiveDate) -> askama::Result<String> {
        Ok(match DateWhen::for_date(d) {
            DateWhen::Future => "future",
            DateWhen::Past => "past",
            DateWhen::Today => "today"
        }.into())
    }

    pub fn spanify(d: &std::fmt::Display) -> askama::Result<String> {
        lazy_static! {
            static ref CONTEXT_REGEX: Regex = Regex::new(r"@(\w+)").unwrap();
            static ref PROJECT_REGEX: Regex = Regex::new(r"\+(\w+)").unwrap();
        }
        let s = format!("{}", d);
        let t = CONTEXT_REGEX
            .replace_all(&s, r#"<a class="todo-context" href="?context=$1">$0</a>"#);
        let r = PROJECT_REGEX
            .replace_all(&t, r#"<a class="todo-project" href="?project=$1">$0</a>"#);
        Ok(String::from(r))
    }

    pub fn humanize_date(date: &::chrono::NaiveDate) -> askama::Result<String> {
        humanize_signed_duration(&super::duration_relative_to_today(date))
    }

    pub fn humanize_duration(duration: &std::time::Duration) -> askama::Result<String> {
        let d = time::Duration::from_std(*duration).unwrap();
        humanize_signed_duration(&d)
    }

    pub fn humanize_signed_duration(d: &::time::Duration) -> askama::Result<String> {
        let date_when = DateWhen::for_duration(*d);

        let word = match date_when {
            DateWhen::Future => "from now",
            DateWhen::Past => "ago",
            DateWhen::Today => return Ok("today".into()),
        };

        fn is_plural(a: i64) -> &'static str {
            if a == 1 { "" } else { "s" }
        }

        let days = d.num_days().abs();
        let string = String::from(if days > 0 {
            if days == 1 {
                match date_when {
                    DateWhen::Future => String::from("tomorrow"),
                    DateWhen::Past => String::from("yesterday"),
                    DateWhen::Today => panic!("DateWhen::Today but days == 1"),
                }
            } else {
                format!("{} day{} {}", days, is_plural(days), word)
            }
        } else {
            let hours = d.num_hours().abs();
            if hours > 0 {
                format!("{} hour{} {}", hours, is_plural(hours), word)
            } else {
                let minutes = d.num_minutes().abs();
                if minutes > 0 {
                    format!("{} minute{} {}", minutes, is_plural(minutes), word)
                } else {
                    let seconds = d.num_seconds().abs();
                    format!("{} second{} {}", seconds, is_plural(seconds), word)
                }
            }
        });

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

fn due_date_sort(a: &Task) -> i32 {
    if a.finished {
        0 // finished tasks don't sort by their due date
    } else {
        match &a.due_date {
            Some(due_date) => match DateWhen::for_date(due_date) {
                DateWhen::Future => 0,
                DateWhen::Today => -1,
                DateWhen::Past => -2,
            },
            None => 0,
        }
    }
}


pub fn render(cached_data: &CachedData, json_dump: &str, query_params: &::routes::QueryStringExtractor) -> String {
    let mut todos_sorted = cached_data.todos.clone();

    todos_sorted.sort_by_key(|a| { (due_date_sort(&a), a.priority) });

    // filter
    {
        if !query_params.context.is_empty() {
            todos_sorted.retain(|t| t.contexts.contains(&query_params.context));
        }
        if !query_params.project.is_empty() {
            todos_sorted.retain(|t| t.projects.contains(&query_params.project));
        }
        if !query_params.search.is_empty() {
            todos_sorted.retain(|t| t.subject.contains(&query_params.search));
        }
    }

    let hello = HelloTemplate {
        json_dump,
        todos_count: cached_data.todos_count,
        local_files: &cached_data.local_files,
        todos: &todos_sorted,
    };

    hello.render().unwrap()
}
