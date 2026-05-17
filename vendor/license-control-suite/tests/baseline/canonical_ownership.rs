use std::fs;
use std::path::Path;

#[test]
fn canonical_client_auth_core_is_exposed_from_curated_core_path() {
    use license_control_suite::core::{
        AuthService, Clock, DeviceIdentityProvider, LocalStateStore, SecretStore, WorkerClient,
    };

    let _ = core::mem::size_of::<AuthService>();
    let _ = core::mem::size_of::<Option<&'static dyn WorkerClient>>();
    let _ = core::mem::size_of::<Option<&'static dyn SecretStore>>();
    let _ = core::mem::size_of::<Option<&'static dyn LocalStateStore>>();
    let _ = core::mem::size_of::<Option<&'static dyn DeviceIdentityProvider>>();
    let _ = core::mem::size_of::<Option<&'static dyn Clock>>();
}

#[test]
fn legacy_auth_core_paths_still_compile_during_transition() {
    use license_control_suite::modules::auth_core::{
        adapters, auth, compatibility, models, policy, reset, session,
    };
    use license_control_suite::modules::shared_contracts::{dto, errors, events, state, versioning};

    let _ = core::mem::size_of::<Option<&'static dyn adapters::ApiClient>>();
    let _ = core::mem::size_of::<Option<&'static dyn adapters::LocalStore>>();
    let _ = core::mem::size_of_val(&auth::activate);
    let _ = core::mem::size_of_val(&compatibility::supported_shared_contracts_range);
    let _ = core::mem::size_of::<models::LocalSession>();
    let _ = core::mem::size_of_val(&policy::offline_access);
    let _ = core::mem::size_of_val(&reset::poll_until_terminal);
    let _ = core::mem::size_of_val(&session::should_force_reauth);
    let _ = core::mem::size_of_val(&dto::Platform::Linux);
    let _ = core::mem::size_of_val(&errors::ErrorCode::AdminForbidden);
    let _ = core::mem::size_of_val(&events::ActorType::Admin);
    let _ = core::mem::size_of_val(&state::LicenseState::BoundActive);
    let _ = core::mem::size_of_val(&versioning::CompatibilityInfo::current);
}

#[test]
fn admin_dashboard_is_not_the_canonical_client_auth_surface() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = fs::read_to_string(root.join("README.md")).expect("README.md should exist");
    let canonical_note = fs::read_to_string(root.join("docs/canonical_surface.md"))
        .expect("canonical surface note should exist");

    assert!(readme.contains("canonical client auth/licensing path"));
    assert!(readme.contains("license_control_suite::core"));
    assert!(canonical_note.contains("admin_dashboard"));
    assert!(canonical_note.contains("separate desktop admin domain"));
    assert!(canonical_note.contains("not the canonical client auth core"));
}

#[test]
fn shared_contracts_and_auth_core_are_documented_as_compatibility_or_legacy_surfaces() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = fs::read_to_string(root.join("README.md")).expect("README.md should exist");
    let canonical_note = fs::read_to_string(root.join("docs/canonical_surface.md"))
        .expect("canonical surface note should exist");

    assert!(readme.contains("transitional compatibility imports"));
    assert!(canonical_note.contains("modules::auth_core"));
    assert!(canonical_note.contains("modules::shared_contracts"));
    assert!(canonical_note.contains("compatibility"));
    assert!(canonical_note.contains("legacy"));
}
