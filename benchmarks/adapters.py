"""Uniform encode/decode adapters over each TOON library under comparison.

Each library has a slightly different API (encode/decode, dumps/loads,
different option names). Wrapping them behind the same Adapter shape lets
test_compare.py benchmark every library through one identical code path.

toon-format (the official reference implementation) is deliberately not
included: as of the version on PyPI, its encode()/decode() both raise
NotImplementedError -- it's a placeholder package, not a working parser.
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable, List


@dataclass(frozen=True)
class Adapter:
    name: str
    encode: Callable[[Any], str]
    decode: Callable[[str], Any]


def _toon_parser() -> Adapter:
    import toon_parser

    return Adapter("toon_parser", toon_parser.encode, toon_parser.decode)


def _ctoon() -> Adapter:
    import ctoon

    return Adapter("ctoon", ctoon.encode, ctoon.decode)


def _toons() -> Adapter:
    import toons

    return Adapter("toons", toons.dumps, toons.loads)


_FACTORIES = (_toon_parser, _ctoon, _toons)


def available_adapters() -> List[Adapter]:
    """Return an Adapter for every library that's actually importable.

    Lets the suite run locally with a subset of libraries installed
    instead of hard-failing collection when one is missing.
    """
    adapters = []
    for factory in _FACTORIES:
        try:
            adapters.append(factory())
        except ImportError:
            continue
    return adapters
