
pub use diesel::sql_types::*;

// Wokaround Diesel default i32 Integer to i64 which
// is Integer in SQLite.

pub type Integer = BigInt;
