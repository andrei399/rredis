// @generated automatically by Diesel CLI.

diesel::table! {
    expiring_entries (key) {
        key -> Text,
        expires_at -> Timestamp,
    }
}
