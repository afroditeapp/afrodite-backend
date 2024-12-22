
use std::collections::HashMap;

use database::history::write::GetDbHistoryWriteCommandsCommon;
use simple_backend_model::{MetricKey, PerfMetricValueArea};

use crate::{
    define_cmd_wrapper_write,
    result::Result,
    write::{db_transaction_history, DbTransactionHistory},
    DataError,
};

define_cmd_wrapper_write!(WriteCommandsCommonHistory);

impl WriteCommandsCommonHistory<'_> {
    pub async fn write_perf_data(
        &self,
        data: HashMap<MetricKey, PerfMetricValueArea>,
    ) -> Result<(), DataError> {
        db_transaction_history!(self, move |mut cmds| {
            cmds.common_history().write_perf_data(data)
        })
    }
}
