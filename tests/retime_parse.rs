#![doc = "git-retime 日期解析与时间戳生成测试。"]

use chrono::{Duration, NaiveDate};
use git_tools::commit::{parse_date, random_timestamps};

#[test]
fn parse_iso_date() {
    let parsed = parse_date("2019-01-01").unwrap();
    assert_eq!(parsed.date(), NaiveDate::from_ymd_opt(2019, 1, 1).unwrap());
}

#[test]
fn random_timestamps_are_unique() {
    let start = parse_date("2020-01-01").unwrap();
    let end = start + Duration::days(30);
    let stamps = random_timestamps(5, start, end).unwrap();
    assert_eq!(stamps.len(), 5);
    assert!(stamps.windows(2).all(|pair| pair[0] < pair[1]));
}
