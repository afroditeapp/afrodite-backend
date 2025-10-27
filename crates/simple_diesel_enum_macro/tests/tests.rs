use diesel::{prelude::*, sql_query, sql_types::SmallInt};
use simple_diesel_enum_macro::SimpleDieselEnum;

#[derive(Debug, Clone, Copy, PartialEq, SimpleDieselEnum, num_enum::TryFromPrimitive)]
#[repr(i16)]
enum TestEnum {
    Variant1 = 1,
    Variant2 = 2,
}

#[test]
fn test_sqlite_insert_and_query_enum() {
    #[derive(diesel::QueryableByName, Debug, PartialEq)]
    struct Row {
        #[sql_type = "SmallInt"]
        value: TestEnum,
    }

    let mut conn = diesel::sqlite::SqliteConnection::establish(":memory:").unwrap();

    sql_query(
        "CREATE TABLE test_enum_table (id INTEGER PRIMARY KEY AUTOINCREMENT, value INTEGER NOT NULL);",
    )
    .execute(&mut conn)
    .unwrap();

    // insert two rows using the enum directly (requires SimpleDieselEnum to provide ToSql for BigInt)
    sql_query("INSERT INTO test_enum_table (value) VALUES (?1), (?2)")
        .bind::<SmallInt, _>(TestEnum::Variant1)
        .bind::<SmallInt, _>(TestEnum::Variant2)
        .execute(&mut conn)
        .unwrap();

    // read back values (requires SimpleDieselEnum to provide FromSql for BigInt)
    let rows: Vec<Row> = sql_query("SELECT value FROM test_enum_table ORDER BY id")
        .load(&mut conn)
        .unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].value, TestEnum::Variant1);
    assert_eq!(rows[1].value, TestEnum::Variant2);
}
