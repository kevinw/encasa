
pub enum DateWhen {
    Past,
    Future,
    Today,
}

pub fn duration_relative_to_today(date: &::chrono::NaiveDate) -> ::time::Duration {
    let today_naive = ::chrono::Local::today().naive_local();
    today_naive.signed_duration_since(*date)
}

impl DateWhen {
    pub fn for_duration(d: ::time::Duration) -> DateWhen {
        if d < ::time::Duration::zero() {
            DateWhen::Future
        } else if d > ::time::Duration::zero() {
            DateWhen::Past
        } else {
            DateWhen::Today
        }
    }

    pub fn for_date(date: &::chrono::NaiveDate) -> DateWhen {
        let duration = duration_relative_to_today(date);
        DateWhen::for_duration(duration)
    }
}

