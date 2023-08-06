pub mod read;
pub mod write;

#[macro_export]
macro_rules! read_json {
    ($pool:expr, $id:expr, $sql:literal, $str_field:ident) => {{
        let id = $id.row_id();
        let pool = $pool;
        sqlx::query!($sql, id)
            .fetch_one(pool)
            .await
            .into_error(SqliteDatabaseError::Execute)
            .and_then(|data| {
                serde_json::from_str(&data.$str_field)
                    .into_error(SqliteDatabaseError::SerdeDeserialize)
            })
    }};
}

#[macro_export]
macro_rules! insert_or_update_json {
    ($pool:expr, $sql:literal, $data:expr, $id:expr) => {{
        let id = $id.row_id();
        let data = serde_json::to_string($data).into_error(SqliteDatabaseError::SerdeSerialize)?;
        let pool = $pool;
        sqlx::query!($sql, data, id)
            .execute(pool)
            .await
            .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }};
}
