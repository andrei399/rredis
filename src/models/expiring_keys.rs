use crate::schema::posts;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = expiring_entry)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ExpiringEntry {
    pub key: String,
    pub timeout: u32,
}
