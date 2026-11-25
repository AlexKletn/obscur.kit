use obscur_plugin_abi::value::AbiStatus;
use obscur_plugin_api::status::PluginApiError;

pub(super) fn check_status(operation: &str, status: AbiStatus) -> Result<(), PluginApiError> {
    match status {
        AbiStatus::OK => Ok(()),
        AbiStatus::PANICKED => Err(PluginApiError::Panicked),
        _ => Err(PluginApiError::Rejected(
            format!(
                "native plugin {operation} returned ABI status {}",
                status.raw()
            )
            .into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rejected_callback_reports_the_operation_and_original_status() {
        let status = AbiStatus::REJECTED;
        assert_eq!(
            check_status("initialize", status),
            Err(PluginApiError::Rejected(
                format!(
                    "native plugin initialize returned ABI status {}",
                    status.raw()
                )
                .into()
            ))
        );
        assert_eq!(
            check_status("initialize", AbiStatus::PANICKED),
            Err(PluginApiError::Panicked)
        );
    }
}
