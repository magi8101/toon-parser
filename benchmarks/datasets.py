"""Shared test data for cross-library TOON benchmarks.

Every library in adapters.py is benchmarked against these exact same
scenarios, so results are comparable apples-to-apples.
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class Scenario:
    name: str
    data: Any


def _tabular_rows(count: int) -> list[dict]:
    return [
        {
            "id": i,
            "name": f"User{i}",
            "active": i % 2 == 0,
            "score": round(i * 1.5, 2),
        }
        for i in range(count)
    ]


SCENARIOS: list[Scenario] = [
    Scenario(
        "small_object",
        {"name": "Alice", "age": 30, "active": True, "email": "alice@example.com"},
    ),
    Scenario("tabular_small", _tabular_rows(10)),
    Scenario("tabular_large_1k", _tabular_rows(1000)),
    Scenario(
        "mixed_array",
        [1, "two", 3.0, True, None, {"nested": "value"}, [1, 2, 3]],
    ),
]
