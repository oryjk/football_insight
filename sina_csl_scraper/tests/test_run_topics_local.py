import importlib.util
from pathlib import Path

import pytest


def helper():
    path = Path(__file__).resolve().parents[1] / 'scripts/run_topics_local.py'
    spec = importlib.util.spec_from_file_location('run_topics_local', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_host_database_url_preserves_credentials_and_uses_actual_tunnel_binding():
    url = 'postgresql://reader:p%40ss@host.docker.internal:55434/football_data?sslmode=disable'
    assert helper().host_database_url(url) == 'postgresql://reader:p%40ss@172.17.0.1:55434/football_data?sslmode=disable'


def test_existing_direct_database_endpoint_is_preserved():
    url = 'postgresql://reader:example@127.0.0.1:55433/football_data'
    assert helper().host_database_url(url) == url


def test_database_target_guard_rejects_unexpected_database_or_gateway_port():
    for url in ['postgresql://reader:example@host.docker.internal:5432/football_data',
                'postgresql://reader:example@host.docker.internal:55434/other_data',
                'https://host.docker.internal/football_data']:
        with pytest.raises(ValueError):
            helper().host_database_url(url)
