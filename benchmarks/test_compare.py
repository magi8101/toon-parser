"""Cross-library TOON encode/decode benchmarks.

Run with: pytest benchmarks/ --benchmark-only --benchmark-json=results.json

Every (library x scenario) combination round-trips through encode/decode
and asserts equality before being timed, so a broken adapter fails the
test outright instead of quietly reporting bogus numbers.
"""
from __future__ import annotations

import pytest

from .adapters import available_adapters
from .datasets import SCENARIOS

ADAPTERS = available_adapters()


@pytest.mark.parametrize("scenario", SCENARIOS, ids=lambda s: s.name)
@pytest.mark.parametrize("adapter", ADAPTERS, ids=lambda a: a.name)
def test_encode(benchmark, adapter, scenario):
    encoded = adapter.encode(scenario.data)
    assert adapter.decode(encoded) == scenario.data
    benchmark(adapter.encode, scenario.data)


@pytest.mark.parametrize("scenario", SCENARIOS, ids=lambda s: s.name)
@pytest.mark.parametrize("adapter", ADAPTERS, ids=lambda a: a.name)
def test_decode(benchmark, adapter, scenario):
    encoded = adapter.encode(scenario.data)
    assert adapter.decode(encoded) == scenario.data
    benchmark(adapter.decode, encoded)
