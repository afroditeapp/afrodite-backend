use std::collections::HashMap;

use database::history::write::GetDbHistoryWriteCommandsCommon;
use simple_backend_model::{MetricKey, PerfMetricValueArea};

use crate::{
    DataError, define_cmd_wrapper_write,
    result::Result,
    write::{DbTransactionHistory, db_transaction_history},
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
