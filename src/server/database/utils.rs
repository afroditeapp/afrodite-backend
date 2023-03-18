use crate::api::model::{Account, AccountIdInternal, AccountSetup, Profile};

use super::{read::ReadCmd, write::WriteCmd};

pub trait GetReadWriteCmd {
    fn read_cmd(id: AccountIdInternal) -> ReadCmd;
    fn write_cmd(id: AccountIdInternal) -> WriteCmd;
}

impl GetReadWriteCmd for Account {
    fn read_cmd(id: AccountIdInternal) -> ReadCmd {
        ReadCmd::AccountState(id)
    }

    fn write_cmd(id: AccountIdInternal) -> WriteCmd {
        WriteCmd::AccountState(id)
    }
}

impl GetReadWriteCmd for AccountSetup {
    fn read_cmd(id: AccountIdInternal) -> ReadCmd {
        ReadCmd::AccountSetup(id)
    }

    fn write_cmd(id: AccountIdInternal) -> WriteCmd {
        WriteCmd::AccountSetup(id)
    }
}

impl GetReadWriteCmd for Profile {
    fn read_cmd(id: AccountIdInternal) -> ReadCmd {
        ReadCmd::Profile(id)
    }

    fn write_cmd(id: AccountIdInternal) -> WriteCmd {
        WriteCmd::Profile(id)
    }
}
