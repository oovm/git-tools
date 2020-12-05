#![doc = "git-retime 日期解析与时间戳生成测试。"]

use chrono::{Duration, NaiveDate, Timelike};
use git_tools::commit::{parse_date, parse_datetime, random_timestamps};

#[test]
fn parse_iso_date() {
    let parsed = parse_date("2019-01-01").unwrap();
    assert_eq!(parsed.date(), NaiveDate::from_ymd_opt(2019, 1, 1).unwrap());
}

#[test]
fn parse_iso_datetime() {
    let parsed = parse_datetime("2019-03-22T14:30:00").unwrap();
    assert_eq!(parsed.hour(), 14);
    assert_eq!(parsed.minute(), 30);
}

#[test]
fn random_timestamps_are_unique() {
    let start = parse_datetime("2020-01-01T00:00:00").unwrap();
    let end = start + Duration::days(30);
    let stamps = random_timestamps(5, start, end).unwrap();
    assert_eq!(stamps.len(), 5);
    assert!(stamps.windows(2).all(|pair| pair[0] < pair[1]));
}
