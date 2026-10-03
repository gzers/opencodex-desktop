use opencodex_desktop_lib::modules::ipc::{validate_request, IpcCommand, IpcRequest};
use std::collections::BTreeMap;

#[test]
fn frozen_ipc_request_rejects_unknown_contract_and_missing_confirmation() {
    let request = IpcRequest {
        request_id: "req_00000000-0000-0000-0000-000000000001".to_string(),
        command: IpcCommand::Status,
        args: BTreeMap::new(),
        confirm: false,
        contract_version: 1,
        secret: None,
    };
    assert!(validate_request::<serde_json::Value>(request.clone()).is_ok());

    let mut invalid = request.clone();
    invalid.contract_version = 2;
    assert_eq!(
        validate_request::<serde_json::Value>(invalid)
            .unwrap_err()
            .error
            .map(|error| error.code),
        Some(opencodex_desktop_lib::modules::ipc::IpcErrorCode::ContractVersionMismatch)
    );

    let mut unconfirmed = request;
    unconfirmed.command = IpcCommand::Start;
    assert_eq!(
        validate_request::<serde_json::Value>(unconfirmed)
            .unwrap_err()
            .error
            .map(|error| error.code),
        Some(opencodex_desktop_lib::modules::ipc::IpcErrorCode::RequireConfirm)
    );
}
