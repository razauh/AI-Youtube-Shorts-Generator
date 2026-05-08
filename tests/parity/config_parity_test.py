import importlib
import os
import sys
import types


def _reload_config_module():
    module_name = "python_legacy.shorts_generator.config"
    sys.modules["dotenv"] = types.SimpleNamespace(load_dotenv=lambda: None)
    if module_name in sys.modules:
        del sys.modules[module_name]
    return importlib.import_module(module_name)


def test_python_defaults_and_trimming_behavior():
    old_env = os.environ.copy()
    try:
        for key in (
            "MUAPI_API_KEY",
            "MUAPI_BASE_URL",
            "MUAPI_POLL_INTERVAL",
            "MUAPI_POLL_TIMEOUT",
            "OPENAI_API_KEY",
            "OPENAI_MODEL",
            "LOCAL_WHISPER_MODEL",
            "LOCAL_WHISPER_DEVICE",
            "LOCAL_OUTPUT_DIR",
        ):
            os.environ.pop(key, None)

        cfg = _reload_config_module()
        assert cfg.MUAPI_API_KEY == ""
        assert cfg.MUAPI_BASE_URL == "https://api.muapi.ai/api/v1"
        assert cfg.POLL_INTERVAL_SECONDS == 5.0
        assert cfg.POLL_TIMEOUT_SECONDS == 600.0
        assert cfg.OPENAI_API_KEY == ""
        assert cfg.OPENAI_MODEL == "gpt-4o-mini"
        assert cfg.LOCAL_WHISPER_MODEL == "base"
        assert cfg.LOCAL_WHISPER_DEVICE == "auto"
        assert cfg.LOCAL_OUTPUT_DIR == "output"

        os.environ["MUAPI_BASE_URL"] = "https://api.muapi.ai/api/v1///"
        os.environ["MUAPI_API_KEY"] = "  abc  "
        os.environ["OPENAI_API_KEY"] = "  xyz  "
        cfg = _reload_config_module()
        assert cfg.MUAPI_BASE_URL == "https://api.muapi.ai/api/v1"
        assert cfg.MUAPI_API_KEY == "abc"
        assert cfg.OPENAI_API_KEY == "xyz"
    finally:
        os.environ.clear()
        os.environ.update(old_env)


def test_python_required_key_errors_and_float_parse():
    old_env = os.environ.copy()
    try:
        os.environ.pop("MUAPI_API_KEY", None)
        os.environ.pop("OPENAI_API_KEY", None)

        cfg = _reload_config_module()
        try:
            cfg.require_api_key()
            raise AssertionError("expected RuntimeError for missing MUAPI_API_KEY")
        except RuntimeError as err:
            assert str(err).startswith("MUAPI_API_KEY is not set.")

        try:
            cfg.require_openai_key()
            raise AssertionError("expected RuntimeError for missing OPENAI_API_KEY")
        except RuntimeError as err:
            assert str(err).startswith("OPENAI_API_KEY is not set.")

        os.environ["MUAPI_POLL_INTERVAL"] = "abc"
        try:
            _reload_config_module()
            raise AssertionError("expected ValueError for invalid MUAPI_POLL_INTERVAL")
        except ValueError:
            pass
    finally:
        os.environ.clear()
        os.environ.update(old_env)
