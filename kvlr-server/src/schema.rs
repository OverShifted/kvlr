// @generated automatically by Diesel CLI.

diesel::table! {
    users (username) {
        username -> Varchar,
        secret_sha256 -> Varchar,
    }
}
