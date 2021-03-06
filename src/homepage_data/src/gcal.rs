#![allow(non_snake_case)]

use chrono;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DateEntry {
    #[serde(default)] pub date: String,
    #[serde(default)] pub dateTime: String,
    #[serde(default)] pub timeZone: String,
}

impl DateEntry {
    pub fn to_naive_date(&self) -> chrono::format::ParseResult<chrono::NaiveDate> {
        if self.dateTime.is_empty() {
            assert!(self.dateTime.is_empty());
            chrono::NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")
        } else {
            assert!(!self.dateTime.is_empty(), "invalid: {:?}", self);
            // assert!(self.timeZone.is_empty());
            Ok(
                chrono::NaiveDateTime::parse_from_str(&self.dateTime, "%+").unwrap().date()
            )
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub summary: String,
    pub description: String,
    pub htmlLink: String,
    pub start: DateEntry,
}

#[cfg(test)]
mod tests {
    //use super::*;
    //use serde_json;
    use time;

    #[test]
    fn parse_gcal_datetime() {
        {
            let t = time::strptime("2018-04-18", "%F").unwrap();
            println!("t: {:?}", t);
            let now = time::now();
            let diff = now - t;
            println!("DAYS: {}, HOURS: {}", diff.num_days(), diff.num_hours());
        }

        match time::strptime("2014-03-10 11:20:34.3454", "%Y-%m-%d %H:%M:%S.%f")
        {
            Ok(v) => println!("OK OK OK: {}", time::strftime("%Y/%m/%d %H:%M:%S.%f",
                                                   &v).unwrap()),
            Err(e) => println!("Error: {}", e),
        };
    }

        /*
    use ::chrono;

    #[test]
    fn parse_gcal_event() {
        let evt = Event {
            summary: "Summary blah".to_string(),
            description: "This is a desc".to_string(),
            htmlLink: "http://google.com".to_string(),
            start: DateEntry {
                date: "2018-05-04".to_string(),
                dateTime: String::new(),
                timeZone: String::new(),
            },
        };

        let date = evt.start.to_naive_date().unwrap();
        let s = ::view::filters::humanize_date(&date).unwrap();
        println!("event relative to today: {}", s);

    }

        */
}
