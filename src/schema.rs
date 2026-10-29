// @generated automatically by Diesel CLI.

diesel::table! {
    notes (id) {
        id -> Int8,
        title -> Varchar,
        message -> Varchar,
    }
}
