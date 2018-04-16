use {CachedData, LocalFileDescWithState};
use todo::Task;

use askama::Template;

mod filters {
    //use ::askama::MarkupDisplay;
    /*
    pub fn linkify(s: MarkupDisplay<&String>) -> ::askama::Result<MarkupDisplay<&String>> {
        let unsafe_str = s.unsafe_string();
        let linkified_str = linkify_str(&unsafe_str).unwrap();
        Ok(MarkupDisplay::Unsafe(&linkified_str))
    }
    */

    /*
pub fn safe<D, I>(v: I) -> Result<MarkupDisplay<D>>
where
    D: fmt::Display,
    MarkupDisplay<D>: From<I>
{
    let res: MarkupDisplay<D> = v.into();
    Ok(res.mark_safe())
}
    */

/*
    pub fn mytrim(s: &::std::fmt::Display) -> ::askama::Result<String> {
        let s = format!("{}", s);
        Ok(s.trim().to_owned())
    }

    pub fn linkify(d: &::std::fmt::Display) -> ::askama::Result<String> {
        let s = format!("{}", d);
        let mut finder = ::linkify::LinkFinder::new();
        finder.kinds(&[::linkify::LinkKind::Url]);
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
    */
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
    let hello = HelloTemplate {
        json_dump,
        todos_count: cached_data.todos_count,
        local_files: &cached_data.local_files,
        todos: &cached_data.todos,
    };
    hello.render().unwrap()
}
