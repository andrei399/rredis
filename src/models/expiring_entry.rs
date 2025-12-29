use crate::schema::expiring_entries;
use chrono::{Duration, NaiveDateTime, Utc};
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = expiring_entries)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ExpiringEntry {
    pub key: String,
    pub expires_at: NaiveDateTime,
}

impl ExpiringEntry {
    pub fn new(key: String, timeout_in_secs: i64) -> ExpiringEntry {
        let target_time = Utc::now().naive_utc() + Duration::seconds(timeout_in_secs);

        ExpiringEntry {
            key,
            expires_at: target_time,
        }
    }
}
