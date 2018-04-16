use ::std::collections::BTreeMap;
use ::nom::rest_s;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Task {
    pub subject: String,
    pub priority: u8,
    pub create_date: Option<::Date>,
    pub finish_date: Option<::Date>,
    pub finished: bool,
    pub threshold_date: Option<::Date>,
    pub due_date: Option<::Date>,
    pub contexts: Vec<String>,
    pub projects: Vec<String>,
    pub hashtags: Vec<String>,
    pub tags: BTreeMap<String, String>,
}

impl Task {
    pub fn calc_hash(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl Default for Task
{
    fn default() -> Self
    {
        Self {
            subject: String::new(),
            priority: 26,
            create_date: None,
            finish_date: None,
            finished: false,
            threshold_date: None,
            due_date: None,
            contexts: Vec::new(),
            projects: Vec::new(),
            hashtags: Vec::new(),
            tags: BTreeMap::new(),
        }
    }
}

impl ::std::str::FromStr for Task
{
    type Err = ();

    fn from_str(s: &str) -> Result<Task, ()>
    {
        task(&s.to_owned())
    }
}

impl ::std::fmt::Display for Task
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
    {
        if self.finished {
            f.write_str("x ")?;
        }

        if self.priority < 26 {
            let priority = (b'A' + self.priority) as char;

            f.write_str(format!("({}) ", priority).as_str())?;
        }

        if let Some(finish_date) = self.finish_date {
            f.write_str(format!("{} ", finish_date.format("%Y-%m-%d")).as_str())?;
        }

        if let Some(create_date) = self.create_date {
            f.write_str(format!("{} ", create_date.format("%Y-%m-%d")).as_str())?;
        }

        f.write_str(self.subject.as_str())?;

        if let Some(due_date) = self.due_date {
            f.write_str(format!(" due:{}", due_date.format("%Y-%m-%d")).as_str())?;
        }

        if let Some(threshold_date) = self.threshold_date {
            f.write_str(format!(" t:{}", threshold_date.format("%Y-%m-%d")).as_str())?;
        }

        for (key, value) in &self.tags {
            f.write_str(format!(" {}:{}", key, value).as_str())?;
        }

        Ok(())
    }
}

named!(date<&str, ::Date>,
    do_parse!(
        year:
            take!(4) >>
            tag_s!("-") >>
        month:
            take!(2) >>
            tag_s!("-") >>
        day:
            take!(2) >>
            tag_s!(" ") >>
        ({
            let year = match year.parse() {
                Ok(year) => year,
                Err(_) => return ::nom::IResult::Error(::nom::ErrorKind::Custom(1)),
            };

            let month = match month.parse() {
                Ok(month) => month,
                Err(_) => return ::nom::IResult::Error(::nom::ErrorKind::Custom(2)),
            };

            let day = match day.parse() {
                Ok(day) => day,
                Err(_) => return ::nom::IResult::Error(::nom::ErrorKind::Custom(3)),
            };

            match ::Date::from_ymd_opt(year, month, day) {
                Some(date) => date,
                None => return ::nom::IResult::Error(::nom::ErrorKind::Custom(4)),
            }
        })
    )
);

named!(priority<&str, u8>,
    do_parse!(
            tag_s!("(") >>
        priority:
            take!(1) >>
            tag_s!(") ") >>
        ({
            let p = priority.as_bytes()[0];

            if p >= b'A' && p <= b'Z' {
                p - b'A'
            }
            else {
                26
            }
        })
    )
);

fn get_tags(regex: &::regex::Regex, subject: &str) -> Vec<String>
{
    let mut tags = regex.captures_iter(subject)
        .map(|x| {
            x["tag"].to_lowercase()
                .to_string()
        })
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>();

    tags.sort();
    tags.dedup();

    tags
}

macro_rules! regex_tags_shared {
    () => { "(?P<space>^|[\\s]){}(?P<tag>[\\w-]+)" }
}

fn get_contexts(subject: &str) -> Vec<String>
{
    lazy_static! {
        static ref REGEX: ::regex::Regex =
            ::regex::Regex::new(&format!(regex_tags_shared!(), "@")).unwrap();
    }
    get_tags(&REGEX, subject)
}

fn get_projects(subject: &str) -> Vec<String>
{
    lazy_static! {
        static ref REGEX: ::regex::Regex =
            ::regex::Regex::new(&format!(regex_tags_shared!(), "\\+")).unwrap();
    }
    get_tags(&REGEX, subject)
}

fn get_hashtags(subject: &str) -> Vec<String>
{
    lazy_static! {
        static ref REGEX: ::regex::Regex =
            ::regex::Regex::new(&format!(regex_tags_shared!(), "#")).unwrap();
    }
    get_tags(&REGEX, subject)
}

fn get_keywords(subject: &str) -> (String, BTreeMap<String, String>)
{
    lazy_static! {
        static ref REGEX: ::regex::Regex =
            ::regex::Regex::new(r" (?P<key>[^\s]+):(?P<value>[^\s^/]+)").unwrap();
    }

    let mut tags = BTreeMap::new();

    let new_subject = REGEX.replace_all(subject, |caps: &::regex::Captures| {
        let key = caps.name("key").unwrap().as_str();
        let value = caps.name("value").unwrap().as_str();

        tags.insert(key.to_owned(), value.to_owned());

        String::new()
    });

    (new_subject.into_owned(), tags)
}

named!(parse<&str, Task>,
    do_parse!(
        finished:
            opt!(complete!(tag_s!("x "))) >>
        priority:
            opt!(complete!(priority)) >>
        finish_date:
            opt!(complete!(date)) >>
        create_date:
            opt!(complete!(date)) >>
        rest:
            rest_s >>
        ({
            let mut task = Task {
                priority: match priority {
                    Some(priority) => priority,
                    None => 26,
                },
                create_date: if create_date.is_none() {
                    finish_date
                } else {
                    create_date
                },
                finish_date: if create_date.is_none() {
                    None
                } else {
                    finish_date
                },
                finished: finished.is_some(),
                contexts: get_contexts(rest),
                projects: get_projects(rest),
                hashtags: get_hashtags(rest),

                .. Default::default()
            };

            let (subject, mut tags) = get_keywords(rest);
            task.subject = subject;

            if let Some(due) = tags.remove("due") {
                task.due_date = match ::Date::parse_from_str(due.as_str(), "%Y-%m-%d") {
                    Ok(due) => Some(due),
                    Err(_) => None,
                };
            }

            if let Some(t) = tags.remove("t") {
                task.threshold_date = match ::Date::parse_from_str(t.as_str(), "%Y-%m-%d") {
                    Ok(t) => Some(t),
                    Err(_) => None,
                };
            }

            task.tags = tags;

            task
        })
    )
);

pub fn task(line: &str) -> Result<Task, ()>
{
    match parse(line) {
        ::nom::IResult::Done(_, task) => Ok(task),
        _ => Err(()),
    }
}
