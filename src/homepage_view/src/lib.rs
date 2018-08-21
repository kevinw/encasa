#[macro_use] extern crate lazy_static;
extern crate askama;
extern crate serde;
extern crate chrono;
extern crate homepage_data;
extern crate regex;
extern crate linkify;
extern crate time;
extern crate failure;

use homepage_data::{CachedData, LocalFileDescWithState, Deadlines, TaskWithContext};
use homepage_data::todo::Task;
use homepage_data::datetools::{DateWhen, duration_relative_to_today};
use homepage_data::datetools;

use askama::Template;

pub struct RenderOpts {
    show_priority_text_label: bool,
}

pub mod filters {
    use regex::Regex;
    use ::{askama, std, linkify};
    use time;
    use datetools::DateWhen;
    
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
        let string = if days > 0 {
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
    todos_count: usize,
    local_files: &'a Vec<LocalFileDescWithState>,
    todos: &'a Vec<TaskWithContext>,
    deadlines: &'a Deadlines,
    render_opts: &'a RenderOpts,
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

pub struct SearchParams {
    pub context: String,
    pub project: String,
    pub search: String,
    pub sort_by: String,
}

impl Default for SearchParams {
    fn default() -> SearchParams {
        SearchParams {
            context: String::new(),
            project: String::new(),
            search: String::new(),
            sort_by: String::new(),
        }
    }
}


pub fn render(
    cached_data: &CachedData,
    query_params: &SearchParams,
    
    ) -> Result<String, failure::Error> {

    let mut todos_sorted = cached_data.todos.clone();

    todos_sorted.sort_by_key(|a| { (due_date_sort(&a.task), a.task.priority) });

    // filter
    {
        if !query_params.context.is_empty() {
            todos_sorted.retain(|t| t.task.contexts.contains(&query_params.context));
        }
        if !query_params.project.is_empty() {
            todos_sorted.retain(|t| t.task.projects.contains(&query_params.project) || t.auto_project == query_params.project);
        }
        if !query_params.search.is_empty() {
            todos_sorted.retain(|t| t.task.subject.contains(&query_params.search));
        }
        if !query_params.sort_by.is_empty() {
            match query_params.sort_by.as_ref() {
                "create_date" => {
                    todos_sorted.sort_unstable_by_key(|t| t.task.create_date);
                }
                _ => {
                    return Err(failure::err_msg(format!("invalid sort_by key: '{}'", query_params.sort_by)));
                }
            }
        }
    }

    let hello = HelloTemplate {
        todos_count: cached_data.todos_count,
        local_files: &cached_data.local_files,
        todos: &todos_sorted,
        deadlines: &cached_data.deadlines,
        render_opts: &RenderOpts { show_priority_text_label: false },
    };

    Ok(hello.render().unwrap())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
