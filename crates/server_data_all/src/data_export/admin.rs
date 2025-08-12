use database::{DbReadMode, DieselDatabaseError};
use serde::Serialize;
use server_data::data_export::SourceAccount;

// TODO(prod): Add more data to data export JSON

#[derive(Serialize)]
pub struct AdminDataExportJson {}

pub fn generate_admin_data_export_json(
    _current: &mut DbReadMode,
    id: SourceAccount,
) -> error_stack::Result<AdminDataExportJson, DieselDatabaseError> {
    let _id = id.0;

    let data = AdminDataExportJson {};

    Ok(data)
}
