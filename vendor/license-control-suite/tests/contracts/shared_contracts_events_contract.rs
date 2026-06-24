use license_control_suite::modules::shared_contracts::events::*;

#[test]
fn audit_event_type_serialization_is_stable() {
    let t = AuditEventType::ResetApproved;
    let got = serde_json::to_string(&t).unwrap();
    assert_eq!(got, "\"RESET_APPROVED\"");
}

#[test]
fn audit_event_has_required_fields() {
    let evt = AuditEvent {
        event_id: "evt_123".to_string(),
        event_type: AuditEventType::AdminAuthSucceeded,
        occurred_at: "2026-05-13T10:00:00Z".to_string(),
        actor: Actor {
            actor_type: ActorType::Admin,
            id: "op_123".to_string(),
        },
        license_id: Some("lic_123".to_string()),
        reset_request_id: None,
        old_device_id: None,
        new_device_id: None,
        request_id: "req_123".to_string(),
        metadata: EventMetadata {
            ip: Some("203.0.113.10".to_string()),
            app_version: Some("1.0.0".to_string()),
            reason: None,
        },
    };

    let json = serde_json::to_value(&evt).unwrap();
    assert_eq!(json["event_type"], "ADMIN_AUTH_SUCCEEDED");
    assert_eq!(json["actor"]["type"], "admin");
    assert_eq!(json["request_id"], "req_123");
}
