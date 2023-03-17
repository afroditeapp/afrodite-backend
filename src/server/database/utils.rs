use crate::api::model::{Account, AccountIdLight, AccountSetup, Profile};

use super::{read::ReadCmd, write::WriteCmd};

pub trait GetReadWriteCmd {
    fn read_cmd(id: AccountIdLight) -> ReadCmd;
    fn write_cmd(id: AccountIdLight) -> WriteCmd;
}

impl GetReadWriteCmd for Account {
    fn read_cmd(id: AccountIdLight) -> ReadCmd {
        ReadCmd::AccountState(id)
    }

    fn write_cmd(id: AccountIdLight) -> WriteCmd {
        WriteCmd::AccountState(id)
    }
}

impl GetReadWriteCmd for AccountSetup {
    fn read_cmd(id: AccountIdLight) -> ReadCmd {
        ReadCmd::AccountSetup(id)
    }

    fn write_cmd(id: AccountIdLight) -> WriteCmd {
        WriteCmd::AccountSetup(id)
    }
}

impl GetReadWriteCmd for Profile {
    fn read_cmd(id: AccountIdLight) -> ReadCmd {
        ReadCmd::Profile(id)
    }

    fn write_cmd(id: AccountIdLight) -> WriteCmd {
        WriteCmd::Profile(id)
    }
}
