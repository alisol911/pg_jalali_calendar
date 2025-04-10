use pgrx::prelude::*;

use chrono::{Datelike, Days, MappedLocalTime, NaiveDate, TimeZone, Utc};
use icu::{calendar::Date, collections::codepointtrie::TrieValue};
use icu_calendar::{persian::Persian, Iso};

pgrx::pg_module_magic!();

fn jalali_date_parse_raw(date: &str) -> (i32, u8, u8) {
    let splitted: Vec<&str> = date.split("/").collect();
    if splitted.len() != 3 {
        panic!("invalid date {date} format");
    }

    let year = match splitted[0].parse::<i32>() {
        Ok(x) => x,
        _ => panic!("invalid date {date} year value"),
    };
    let month = match splitted[1].parse::<u8>() {
        Ok(x) => x,
        _ => panic!("invalid date {date} month value"),
    };
    let day = match splitted[2].parse::<u8>() {
        Ok(x) => x,
        _ => panic!("invalid date {date} day value"),
    };
    (year, month, day)
}

fn jalali_date_parse(date: &str) -> Date<Persian> {
    let (year, month, day) = jalali_date_parse_raw(date);
    match Date::try_new_persian_date(year, month, day) {
        Ok(x) => x,
        _ => panic!("invalid date {date} jalali date"),
    }
}

fn jalali_date_to_gregorian_internal(date: &str) -> Date<Iso> {
    jalali_date_parse(date).to_iso()
}

#[pg_extern]
fn jalali_date_diff_with_addition(date_start: &str, date_end: &str, addition: i32) -> i32 {
    let iso_date_start = jalali_date_to_gregorian_internal(date_start);
    let iso_date_end = jalali_date_to_gregorian_internal(date_end);

    let utc_date_start = match Utc.with_ymd_and_hms(
        iso_date_start.year().number,
        iso_date_start.month().ordinal,
        iso_date_start.day_of_month().0,
        0,
        0,
        0,
    ) {
        MappedLocalTime::Single(x) => x,
        _ => panic!("invalid date {date_start} start"),
    };
    let utc_date_end = match Utc.with_ymd_and_hms(
        iso_date_end.year().number,
        iso_date_end.month().ordinal,
        iso_date_end.day_of_month().0,
        0,
        0,
        0,
    ) {
        MappedLocalTime::Single(x) => x,
        _ => panic!("invalid date {date_end} end"),
    };

    let date_interval = date_component::date_component::calculate(&utc_date_start, &utc_date_end);

    (date_interval.interval_days as i32 + addition) * if date_interval.invert { -1 } else { 1 }
}

#[pg_extern]
fn jalali_date_diff(date_start: &str, date_end: &str) -> i32 {
    jalali_date_diff_with_addition(date_start, date_end, 0)
}

#[pg_extern]
fn jalali_date_to_gregorian(date: &str) -> String {
    let iso_date = jalali_date_to_gregorian_internal(date);
    format!(
        "{:0>4}-{:0>2}-{:0>2}",
        iso_date.year().number,
        iso_date.month().ordinal,
        iso_date.day_of_month().0,
    )
}

fn jalali_date_add_days_internal(date: &str, days: i32) -> Date<Persian> {
    let iso_date = jalali_date_to_gregorian_internal(date);

    let new_iso_date = match NaiveDate::from_ymd_opt(
        iso_date.year().number,
        iso_date.month().ordinal,
        iso_date.day_of_month().0,
    ) {
        Some(x) => x,
        None => panic!("invalid date {date} iso conversion"),
    };

    let added_date = match if days > 0 {
        new_iso_date.checked_add_days(Days::new(days as u64))
    } else {
        new_iso_date.checked_sub_days(Days::new(days.abs() as u64))
    } {
        Some(x) => x,
        None => panic!("invalid date {date} add day"),
    };

    match Date::try_new_iso_date(
        added_date.year().try_into().unwrap(),
        (added_date.month0() + 1).try_into().unwrap(),
        (added_date.day0() + 1).try_into().unwrap(),
    ) {
        Ok(x) => x,
        _ => panic!("invalid date {date} new jalali date"),
    }
    .to_calendar(Persian)
}

#[pg_extern]
fn jalali_date_add_days(date: &str, days: i32) -> String {
    let new_jalali_date = jalali_date_add_days_internal(date, days);
    format!(
        "{:0>4}/{:0>2}/{:0>2}",
        new_jalali_date.year().number,
        new_jalali_date.month().ordinal,
        new_jalali_date.day_of_month().0
    )
}

#[pg_extern]
fn jalali_date_add_months(date: &str, months: i32) -> String {
    let (year, month, day) = jalali_date_parse_raw(date);

    let _parsed = match Date::try_new_persian_date(year, month, day) {
        Ok(x) => x,
        _ => panic!("invalid date {date} jalali date"),
    };

    if months <= 0 {
        panic!("invalid months value")
    }

    let new_year_raw = if months >= 0 {
        year + (months as i32 / 12)
    } else {
        year - (-months as i32 / 12)
    };

    let new_month_raw = if months >= 0 {
        month + (months as i32 % 12) as u8
    } else {
        month - (-months as i32 % 12) as u8
    };

    let (new_year, new_month) = if new_month_raw > 12 {
        (new_year_raw + 1, new_month_raw - 12)
    } else {
        (new_year_raw, new_month_raw)
    };

    let date_check = match Date::try_new_persian_date(new_year, 1, 1) {
        Ok(x) => x,
        _ => panic!("invalid date {new_year}/01/01 jalali date"),
    };

    let day = if (date_check.is_in_leap_year() && new_month == 12 && day > 29)
        || (new_month > 6 && new_month < 12 && day > 30)
    {
        30
    } else if new_month == 12 && day > 29 {
        29
    } else {
        day
    };
    format!("{:0>4}/{:0>2}/{:0>2}", new_year, new_month, day,)
}

#[pg_extern]
fn jalali_date_now() -> String {
    let now = chrono::offset::Utc::now();
    let new_date = match Date::try_new_iso_date(now.year(), now.month() as u8, now.day() as u8) {
        Ok(x) => x,
        _ => panic!("invalid date"),
    }
    .to_calendar(Persian);
    format!(
        "{:0>4}/{:0>2}/{:0>2}",
        new_date.year().number,
        new_date.month().ordinal,
        new_date.day_of_month().0
    )
}

#[pg_extern]
fn gregorian_date_to_jalali(date: &str) -> String {
    let splitted: Vec<&str> = date.split("-").collect();
    if splitted.len() != 3 {
        panic!("invalid date {date} format");
    }

    let year = match splitted[0].parse::<i32>() {
        Ok(x) => x,
        _ => panic!("invalid date {date} year value"),
    };
    let month = match splitted[1].parse::<u8>() {
        Ok(x) => x,
        _ => panic!("invalid date {date} month value"),
    };
    let day = match splitted[2].parse::<u8>() {
        Ok(x) => x,
        _ => panic!("invalid date {date} day value"),
    };

    let gregorian_date = match icu::calendar::Date::try_new_gregorian_date(year, month, day) {
        Ok(x) => x,
        _ => panic!("invalid date {date} gregorian date"),
    };

    let jalali_date = gregorian_date.to_calendar(Persian);

    format!(
        "{:0>4}/{:0>2}/{:0>2}",
        jalali_date.year().number,
        jalali_date.month().ordinal,
        jalali_date.day_of_month().0
    )
}

#[pg_extern]
fn jalali_date_period_state(date: &str, start: i32) -> String {
    let date_value = jalali_date_parse(date);
    let month_end = (date_value.day_of_month().0 == 29
        && date_value.month().ordinal == 12
        && !date_value.is_in_leap_year())
        || (date_value.day_of_month().0 == 30
            && date_value.month().ordinal >= 7
            && (date_value.month().ordinal <= 11
                || (date_value.month().ordinal == 12 && date_value.is_in_leap_year())))
        || (date_value.day_of_month().0 == 31 && date_value.month().ordinal <= 6);

    if month_end && date_value.day_of_month().0 <= start.to_u32() {
        return "End".to_string();
    };

    if date_value.day_of_month().0 == 1 {
        if date_value.month().ordinal == 1
            && (start >= 30
                || start.to_u32() == jalali_date_add_days_internal(date, -1).day_of_month().0)
        {
            return "Start".to_string();
        }

        if date_value.month().ordinal >= 2 && date_value.month().ordinal <= 7 && start == 31
            || date_value.month().ordinal >= 8 && date_value.month().ordinal <= 12 && start >= 30
        {
            return "Start".to_string();
        }
    }

    if start >= 1 && start <= 31 {
        if date_value.day_of_month().0 == start.to_u32() {
            return "End".to_string();
        } else if date_value.day_of_month().0 == start.to_u32() + 1 {
            return "Start".to_string();
        } else {
            return "Middle".to_string();
        }
    }

    "Unknown".to_string()
}

#[pg_extern]
fn jalali_date_is_leap_year(date: &str) -> bool {
    let date_value = jalali_date_parse(date);
    date_value.is_in_leap_year()
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    fn test_jalali_date_add_days() {
        assert_eq!("1403/05/30", crate::jalali_date_add_days("1403/05/28", 2));
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    #[must_use]
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
