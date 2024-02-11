use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Europe::Helsinki;

pub(crate) fn fix_new_date(buf: &str) -> String {
    let mut index1 = 0;
    let mut result = String::with_capacity(buf.len());

    loop {
        if let Some(index2) = buf[index1..].find("new Date(") {
            let index2 = index1 + index2;
            let end = index2 + buf[index2..].find(')').expect("Missing \")\"");
            let date_str = &buf[index2 + 9..end];
            let date_int: i64 = date_str.parse().expect("parse new Date() timestamp");

            // Javascript's "new Date()" supposed to be seconds since UNIX epoch UTC,
            // but it seems Energiatili's numbers are in finnish timezone instead.
            let naive_date = NaiveDateTime::from_timestamp_opt(date_int / 1000, 0).expect("parse NaiveDateTime");
            let localtime = Helsinki.from_local_datetime(&naive_date).unwrap();
            let timestamp: DateTime<Utc> = localtime.with_timezone(&Utc);

            result.push_str(&buf[index1..index2]);
            result.push('"');
            result.push_str(&timestamp.to_rfc3339());
            result.push('"');

            index1 = end + 1;
        } else {
            result.push_str(&buf[index1..]);
            break;
        }
    }

    result
}
