import json
from pathlib import Path


FIXTURE_DIR = (
    Path(__file__).resolve().parents[1] / "fixtures" / "license_worker_contract_v1"
)
MANIFEST_PATH = FIXTURE_DIR / "manifest.json"

FROZEN_ERROR_CODES = {
    "invalid_license_key",
    "invalid_purchase_email",
    "invalid_device_identity",
    "invalid_reset_request",
    "invalid_deletion_request",
    "deletion_request_not_found",
    "invalid_deletion_lookup_token",
    "device_already_bound",
    "reauth_required",
    "worker_unreachable",
    "reset_request_not_found",
    "unauthorized",
    "storage",
    "serialization",
    "invalid_transition",
    "bad_request",
}

RESET_STATUS_VALUES = {"pending", "approved", "rejected", "expired"}
DELETION_STATUS_VALUES = {"pending", "approved", "processing", "rejected", "completed", "failed"}


def _load(name: str):
    with (FIXTURE_DIR / name).open("r", encoding="utf-8") as f:
        return json.load(f)


def _assert_success_envelope(doc):
    assert doc.get("ok") is True
    assert isinstance(doc.get("data"), dict)


def _assert_error_envelope(doc):
    assert doc.get("ok") is False
    err = doc.get("error")
    assert isinstance(err, dict)
    assert isinstance(err.get("code"), str) and err["code"]
    assert isinstance(err.get("message"), str) and err["message"]
    assert isinstance(err.get("request_id"), str) and err["request_id"]
    assert isinstance(err.get("retryable"), bool)
    assert err["code"] in FROZEN_ERROR_CODES


def test_manifest_files_exist_and_are_valid_json():
    manifest = _load("manifest.json")
    files = manifest.get("files")
    assert isinstance(files, list) and files
    for name in files:
        path = FIXTURE_DIR / name
        assert path.exists(), f"missing fixture: {name}"
        with path.open("r", encoding="utf-8") as f:
            json.load(f)


def test_activate_contract_shapes():
    req = _load("activate_request.json")
    assert isinstance(req.get("license_key"), str) and req["license_key"]
    assert isinstance(req.get("device_public_key"), str) and req["device_public_key"]
    assert isinstance(req.get("app_version"), str) and req["app_version"]
    assert isinstance(req.get("timestamp_ms"), int) and req["timestamp_ms"] > 0
    fp = req.get("fingerprint")
    assert isinstance(fp, dict)
    assert all(isinstance(fp.get(k), str) and fp.get(k) for k in ("os_name", "platform_family", "arch"))

    ok = _load("activate_success_200.json")
    _assert_success_envelope(ok)
    data = ok["data"]
    assert isinstance(data.get("access_token"), str) and data["access_token"]
    assert isinstance(data.get("masked_license_key"), str) and data["masked_license_key"]
    assert data.get("entitlement") == "active"
    assert isinstance(data.get("token_expires_at_ms"), int) and data["token_expires_at_ms"] > 0

    _assert_error_envelope(_load("activate_error_409_device_already_bound.json"))
    _assert_error_envelope(_load("activate_error_400_bad_request.json"))


def test_validate_contract_shapes():
    req = _load("validate_request.json")
    assert isinstance(req.get("access_token"), str) and req["access_token"]

    ok = _load("validate_success_200.json")
    _assert_success_envelope(ok)
    data = ok["data"]
    assert data.get("entitlement") == "active"
    assert isinstance(data.get("masked_license_key"), str) and data["masked_license_key"]
    assert isinstance(data.get("token_expires_at_ms"), int) and data["token_expires_at_ms"] > 0

    _assert_error_envelope(_load("validate_error_401_reauth_required.json"))
    _assert_error_envelope(_load("validate_error_503_worker_unreachable.json"))


def test_reset_request_contract_shapes():
    req = _load("reset_request_request.json")
    assert isinstance(req.get("purchaser_email"), str) and "@" in req["purchaser_email"]
    assert isinstance(req.get("device_public_key"), str) and req["device_public_key"]
    assert isinstance(req.get("timestamp_ms"), int) and req["timestamp_ms"] > 0

    ok = _load("reset_request_success_200.json")
    _assert_success_envelope(ok)
    data = ok["data"]
    assert isinstance(data.get("request_id"), str) and data["request_id"]
    assert data.get("status") in RESET_STATUS_VALUES

    _assert_error_envelope(_load("reset_request_error_400_invalid_purchase_email.json"))
    _assert_error_envelope(_load("reset_request_error_409_invalid_transition.json"))


def test_reset_status_contract_shapes():
    req = _load("reset_status_request.json")
    assert isinstance(req.get("request_id"), str) and req["request_id"]

    pending = _load("reset_status_success_200_pending.json")
    _assert_success_envelope(pending)
    assert pending["data"].get("status") == "pending"

    approved = _load("reset_status_success_200_approved.json")
    _assert_success_envelope(approved)
    assert approved["data"].get("status") == "approved"

    _assert_error_envelope(_load("reset_status_error_404_not_found.json"))


def test_delete_user_data_contract_shapes():
    req = _load("delete_user_data_request.json")
    assert isinstance(req.get("license_key"), str) and req["license_key"]
    assert req.get("confirmation") == "DELETE"
    assert isinstance(req.get("timestamp_ms"), int) and req["timestamp_ms"] > 0

    ok = _load("delete_user_data_success_200_pending.json")
    _assert_success_envelope(ok)
    data = ok["data"]
    assert isinstance(data.get("request_id"), str) and data["request_id"]
    assert isinstance(data.get("lookup_token"), str) and data["lookup_token"]
    assert data.get("status") in DELETION_STATUS_VALUES

    _assert_error_envelope(_load("delete_user_data_error_400_invalid_request.json"))


def test_delete_user_data_status_contract_shapes():
    req = _load("delete_user_data_status_request.json")
    assert isinstance(req.get("request_id"), str) and req["request_id"]
    assert isinstance(req.get("lookup_token"), str) and req["lookup_token"]

    ok = _load("delete_user_data_status_success_200_completed.json")
    _assert_success_envelope(ok)
    data = ok["data"]
    assert data.get("status") == "completed"
    assert isinstance(data.get("completed_at_ms"), int) and data["completed_at_ms"] > 0

    _assert_error_envelope(_load("delete_user_data_status_error_401_invalid_lookup_token.json"))


def test_admin_delete_user_data_contract_shapes():
    listing = _load("admin_delete_user_data_list_success_200.json")
    _assert_success_envelope(listing)
    requests = listing["data"].get("requests")
    assert isinstance(requests, list) and requests
    item = requests[0]
    assert item.get("status") in DELETION_STATUS_VALUES
    assert item.get("purchaser_email") == "b***@example.com"
    assert isinstance(item.get("deletion_preview"), dict)

    decision = _load("admin_delete_user_data_decision_success_200_completed.json")
    _assert_success_envelope(decision)
    assert decision["data"].get("status") == "completed"
    assert isinstance(decision["data"].get("deletion_summary"), dict)


def test_webhook_contract_shapes():
    req = _load("webhook_gumroad_request_purchase.json")
    assert req.get("content_type") == "application/x-www-form-urlencoded"
    body = req.get("body")
    assert isinstance(body, dict)
    assert isinstance(body.get("sale_id"), str) and body["sale_id"]
    assert isinstance(body.get("product_id"), str) and body["product_id"]
    assert isinstance(body.get("email"), str) and body["email"]

    ok = _load("webhook_gumroad_success_200.json")
    _assert_success_envelope(ok)
    assert ok["data"].get("provider") == "gumroad"
    assert ok["data"].get("verified") is True

    _assert_error_envelope(_load("webhook_gumroad_error_400_bad_request.json"))
