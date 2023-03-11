use crate::api::model::{Account, AccountId, AccountSetup, Profile};

use super::{read::ReadCmd, write::WriteCmd};




pub trait GetReadWriteCmd {
    // const READ_CMD: ReadCmd;
    // const WRITE_CMD: WriteCmd;

    fn read_cmd(id: &AccountId) -> ReadCmd;
    fn write_cmd(id: &AccountId) -> WriteCmd;
}

impl GetReadWriteCmd for Account {
    fn read_cmd(id: &AccountId) -> ReadCmd {
        ReadCmd::AccountState(id.clone())
    }

    fn write_cmd(id: &AccountId) -> WriteCmd {
        WriteCmd::AccountState(id.clone())
    }
}

impl GetReadWriteCmd for AccountSetup {
    fn read_cmd(id: &AccountId) -> ReadCmd {
        ReadCmd::AccountSetup(id.clone())
    }

    fn write_cmd(id: &AccountId) -> WriteCmd {
        WriteCmd::AccountSetup(id.clone())
    }
}

impl GetReadWriteCmd for Profile {
    fn read_cmd(id: &AccountId) -> ReadCmd {
        ReadCmd::Profile(id.clone())
    }

    fn write_cmd(id: &AccountId) -> WriteCmd {
        WriteCmd::Profile(id.clone())
    }
}
