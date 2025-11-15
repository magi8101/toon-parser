"""Async wrappers for toonpy using asyncio.to_thread()"""

import asyncio
from typing import Any, Optional
import toonpy


async def encode(data: Any, delimiter: Optional[str] = None, strict: Optional[bool] = None) -> str:
    """Encode Python data to TOON format string."""
    return await asyncio.to_thread(toonpy.encode, data, delimiter, strict)


async def decode(toon_str: str, delimiter: Optional[str] = None, strict: Optional[bool] = None) -> Any:
    """Decode TOON format string to Python data."""
    return await asyncio.to_thread(toonpy.decode, toon_str, delimiter, strict)


async def dumps(data: Any, **kwargs) -> str:
    """Encode Python data to TOON format string."""
    return await asyncio.to_thread(toonpy.dumps, data, **kwargs)


async def loads(toon_str: str, **kwargs) -> Any:
    """Decode TOON format string to Python data."""
    return await asyncio.to_thread(toonpy.loads, toon_str, **kwargs)


async def encode_batch(data_list: list, **kwargs) -> list:
    """Encode multiple objects concurrently."""
    tasks = [encode(data, **kwargs) for data in data_list]
    return await asyncio.gather(*tasks)


async def decode_batch(toon_strs: list, **kwargs) -> list:
    """Decode multiple TOON strings concurrently."""
    tasks = [decode(s, **kwargs) for s in toon_strs]
    return await asyncio.gather(*tasks)


__all__ = ['encode', 'decode', 'dumps', 'loads', 'encode_batch', 'decode_batch']
