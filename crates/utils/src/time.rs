use chrono::{Datelike, Local, NaiveDate};

pub fn age_in_years_from_birthdate(birthdate: NaiveDate) -> i32 {
    let now = Local::now().date_naive();

    if now < birthdate {
        // Birthdate is in future
        0
    } else if now.month() < birthdate.month()
        || (now.month() == birthdate.month() && now.day() < birthdate.day())
    {
        // Before birthday
        now.year() - birthdate.year() - 1
    } else {
        // Birthday or after birthday
        now.year() - birthdate.year()
    }
}
