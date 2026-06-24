use shorts_tauri_app::commands::admin::*;

#[tokio::test]
async fn test_admin_commands_disabled_in_devolens_mode() {
    std::env::set_var("LICENSE_BACKEND_MODE", "devolens");
    std::env::set_var("DEVOLENS_ACCESS_TOKEN", "mock-token");
    std::env::set_var("DEVOLENS_PRODUCT_ID", "mock-product");

    // test admin_overview
    let err = admin_overview().await.unwrap_err();
    assert_eq!(err.code, "devolens_mode_active");

    // test admin_list_licenses
    let err = admin_list_licenses(None, None, None, None).await.unwrap_err();
    assert_eq!(err.code, "devolens_mode_active");

    // test admin_list_device_bindings
    let err = admin_list_device_bindings(None, None, None, None).await.unwrap_err();
    assert_eq!(err.code, "devolens_mode_active");

    // test admin_list_audit_events
    let err = admin_list_audit_events(None, None, None).await.unwrap_err();
    assert_eq!(err.code, "devolens_mode_active");

    // test admin_list_idempotency_records
    let err = admin_list_idempotency_records(None, None).await.unwrap_err();
    assert_eq!(err.code, "devolens_mode_active");

    // test admin_list_reset_requests
    let err = admin_list_reset_requests(ResetRequestStatus::Pending).await.unwrap_err();
    assert_eq!(err.code, "devolens_mode_active");

    // test admin_list_deletion_requests
    let err = admin_list_deletion_requests(DeletionRequestStatus::Pending).await.unwrap_err();
    assert_eq!(err.code, "devolens_mode_active");

    // test admin_approve_reset_request
    let err = admin_approve_reset_request("req_123".to_string(), None).await.unwrap_err();
    assert_eq!(err.code, "devolens_mode_active");

    // test admin_reject_reset_request
    let err = admin_reject_reset_request("req_123".to_string(), None).await.unwrap_err();
    assert_eq!(err.code, "devolens_mode_active");

    // test admin_approve_deletion_request
    let err = admin_approve_deletion_request("req_123".to_string(), "DELETE USER DATA".to_string(), None).await.unwrap_err();
    assert_eq!(err.code, "devolens_mode_active");

    // test admin_reject_deletion_request
    let err = admin_reject_deletion_request("req_123".to_string(), None).await.unwrap_err();
    assert_eq!(err.code, "devolens_mode_active");

    // test admin_disable_license
    let err = admin_disable_license("abc123".to_string(), "reason".to_string(), true).await.unwrap_err();
    assert_eq!(err.code, "devolens_mode_active");

    // test admin_test_connection
    let err = admin_test_connection().await.unwrap_err();
    assert_eq!(err.code, "devolens_mode_active");
}
