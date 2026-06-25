use shorts_tauri_app::commands::admin::*;

fn configure_devolens_customer_licensing() {
    unsafe {
        std::env::set_var("LICENSE_BACKEND_MODE", "devolens");
        std::env::set_var("DEVOLENS_ACCESS_TOKEN", "mock-token");
        std::env::set_var("DEVOLENS_PRODUCT_ID", "mock-product");
        std::env::set_var("SKIP_DEVOLENS_TOKEN_SAFETY_CHECK", "1");
    }
}

fn clear_devolens_customer_licensing() {
    unsafe {
        std::env::remove_var("LICENSE_BACKEND_MODE");
        std::env::remove_var("DEVOLENS_ACCESS_TOKEN");
        std::env::remove_var("DEVOLENS_PRODUCT_ID");
        std::env::remove_var("SKIP_DEVOLENS_TOKEN_SAFETY_CHECK");
    }
}

fn assert_not_devolens_guard(err: AdminCommandError) {
    assert_ne!(err.code, "devolens_mode_active");
}

#[tokio::test]
async fn admin_commands_are_not_disabled_by_devolens_customer_licensing() {
    configure_devolens_customer_licensing();

    assert_not_devolens_guard(admin_overview().await.unwrap_err());
    assert_not_devolens_guard(admin_list_licenses(None, None, None, None).await.unwrap_err());
    assert_not_devolens_guard(admin_list_device_bindings(None, None, None, None).await.unwrap_err());
    assert_not_devolens_guard(admin_list_audit_events(None, None, None).await.unwrap_err());
    assert_not_devolens_guard(admin_list_idempotency_records(None, None).await.unwrap_err());
    assert_not_devolens_guard(admin_list_reset_requests(ResetRequestStatus::Pending).await.unwrap_err());
    assert_not_devolens_guard(admin_list_deletion_requests(DeletionRequestStatus::Pending).await.unwrap_err());
    assert_not_devolens_guard(admin_approve_reset_request("req_123".to_string(), None).await.unwrap_err());
    assert_not_devolens_guard(admin_reject_reset_request("req_123".to_string(), None).await.unwrap_err());
    assert_not_devolens_guard(
        admin_approve_deletion_request("req_123".to_string(), "DELETE USER DATA".to_string(), None)
            .await
            .unwrap_err(),
    );
    assert_not_devolens_guard(admin_reject_deletion_request("req_123".to_string(), None).await.unwrap_err());
    assert_not_devolens_guard(
        admin_disable_license("abc123".to_string(), "reason".to_string(), true)
            .await
            .unwrap_err(),
    );
    assert_not_devolens_guard(admin_test_connection().await.unwrap_err());

    clear_devolens_customer_licensing();
}
