#![allow(non_snake_case)]

//use chrono::prelude::*;
//use chrono::offset::LocalResult;
//use chrono::format::{ParseResult, ParseError};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DateEntry {
    #[serde(default)] date: String,
    #[serde(default)] dateTime: String,
    #[serde(default)] timeZone: String,
}

/*
impl DateEntry {
    fn to_chrono_datetime(&self) -> ParseResult<DateTime<Local>> {
        println!("datetime_from_str on {}", self.date);
        Local.datetime_from_str(&self.date, "%Y-%m-%d")
    }
}
*/

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    summary: String,
    description: String,
    htmlLink: String,
    start: DateEntry,
}

#[cfg(test)]
mod tests {
    //use super::*;
    //use serde_json;
    use time;

    #[test]
    fn parse_gcal_datetime() {
        /*
        let json = r#"{"date": "2018-05-04"}"#;
        let entry:DateEntry = serde_json::from_str(&json).unwrap();

        let dt = entry.to_chrono_datetime().expect("problem parsing date");
        let local_dt = Local.ymd(2018, 5, 4).and_hms(12, 0, 0);

        assert_eq!(dt, local_dt);
        */
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
}
