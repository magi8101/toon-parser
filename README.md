# toon-parser

High-performance Python bindings for the TOON format parser, built with PyO3 and Rust.

**5.82x faster** than pure Python implementations, optimized for tabular data and LLM applications.

> **Note**: The `toon_parser` (sync) and `toon_parser_async` (async) modules ship together in a single package. Wheels are no longer published to PyPI — install directly with `pip` from GitHub instead (see below).

---

## Features

- **High Performance**: 5.82x average speedup (2.98x - 9.68x range) over pure Python implementations
- **Zero Dependencies**: Pure PyO3/Rust implementation with no runtime dependencies
- **Optimized for Tabular Data**: Inline primitive conversions for common data patterns
- **Async Support**: Native asyncio integration via the bundled `toon_parser_async` module
- **Broad Compatibility**: Python 3.8+ with abi3 wheels
- **Drop-in Replacement**: Compatible API with other TOON libraries

---

## Installation

This project is no longer published to PyPI. Install directly with `pip` instead.

### From a GitHub Release (Recommended)

```bash
# Install a specific tagged release
pip install https://github.com/magi8101/toon-parser/archive/refs/tags/vX.Y.Z.tar.gz
```

Replace `vX.Y.Z` with the release tag from the [Releases page](https://github.com/magi8101/toon-parser/releases). `pip` builds the Rust extension locally via maturin's PEP 517 hook, so a Rust toolchain is required (see Building from Source below).

### Directly from Git

```bash
# Latest commit on main
pip install git+https://github.com/magi8101/toon-parser.git

# A specific tag, branch, or commit
pip install git+https://github.com/magi8101/toon-parser.git@vX.Y.Z
```

### From Source

```bash
# Clone this repository
git clone https://github.com/magi8101/toon-parser.git
cd toon-parser

# Build and install (produces one wheel containing both
# toon_parser and toon_parser_async)
pip install maturin
maturin build --release
pip install target/wheels/toon_parser-*.whl
```

---

## Quick Start

### Synchronous API

```python
import toon_parser

# Encode Python data to TOON
data = {"name": "Alice", "age": 30, "active": True}
toon_str = toon_parser.encode(data)
# Output: 'active: true\nage: 30\nname: Alice\n'

# Decode TOON to Python
result = toon_parser.decode(toon_str)
# Output: {'active': True, 'age': 30, 'name': 'Alice'}

# Batch operations
data_list = [{"id": i, "name": f"User{i}"} for i in range(100)]
toon_strs = toon_parser.encode_batch(data_list)
results = toon_parser.decode_batch(toon_strs)
```

### Asynchronous API

`toon_parser_async` is included in the same install (see Installation above) — no separate package needed.

```python
import asyncio
from toon_parser_async import encode, decode, encode_batch, decode_batch

async def main():
    # Async encode/decode
    data = {"name": "Bob", "age": 25}
    toon_str = await encode(data)
    result = await decode(toon_str)
    
    # Concurrent batch operations
    data_list = [{"id": i} for i in range(1000)]
    toon_strs = await encode_batch(data_list)
    results = await decode_batch(toon_strs)

asyncio.run(main())
```

---

## API Reference

### Synchronous (`toon_parser`)

#### `encode(data, delimiter=None, strict=None) -> str`
Encode Python data to TOON format string.

**Parameters:**
- `data`: Python object (dict, list, str, int, float, bool, None)
- `delimiter`: Optional delimiter ('comma', 'tab', 'pipe'). Default: 'comma'
- `strict`: Optional strict mode. Default: False

**Returns:** TOON-formatted string

#### `decode(toon_str, delimiter=None, strict=None) -> Any`
Decode TOON format string to Python data.

**Parameters:**
- `toon_str`: TOON-formatted string
- `delimiter`: Optional delimiter hint ('comma', 'tab', 'pipe'). Auto-detected if not specified
- `strict`: Optional strict mode. Default: False

**Returns:** Python object

#### `encode_batch(data_list, delimiter=None, strict=None) -> list`
Encode multiple Python objects.

#### `decode_batch(toon_strs, delimiter=None, strict=None) -> list`
Decode multiple TOON strings.

#### `dumps(data, **kwargs) -> str`
Alias for `encode()`.

#### `loads(toon_str, **kwargs) -> Any`
Alias for `decode()`.

### Asynchronous (`toon_parser_async`)

Included in the same install as the sync package (see Installation above). All functions have the same signature as the sync API but return coroutines.

```python
from toon_parser_async import encode, decode, encode_batch, decode_batch

# All functions are async
await encode(data)
await decode(toon_str)
await encode_batch(data_list)
await decode_batch(toon_strs)
```

---

## Performance

### Benchmark Results

Tested against toon-llm v1.0.0b6 (November 2025):

| Test | toon-parser | toon-llm | Speedup |
|------|--------|----------|---------|
| Small Object Decode | 16.1 μs | 94.7 μs | **5.9x** |
| Tabular Small Decode | 46.0 μs | 144.2 μs | **3.1x** |
| Tabular Large Decode (1k rows) | 220.2 μs | 905.9 μs | **4.1x** |
| Mixed Array Decode | 21.1 μs | 102.8 μs | **4.9x** |
| Small Object Encode | 36.3 μs | 278.1 μs | **7.7x** |
| Tabular Large Encode (1k rows) | 325.4 μs | 969.9 μs | **3.0x** |

**Average: 5.82x faster** (range: 2.98x - 9.68x)

See [PERFORMANCE.md](PERFORMANCE.md) for detailed analysis.

---

## Architecture

### Core Components

**Rust Core (`src/lib.rs`)**
- PyO3 bindings for Python C API
- Custom `json_to_python()` with inlined primitive conversions
- Zero-copy operations where possible
- Optimized for TOON's common patterns (tabular data)

**Async Wrapper (`python/toon_parser_async.py`)**
- Pure Python asyncio wrapper
- Uses `asyncio.to_thread()` to release GIL
- Enables concurrent I/O operations

**TOON Parser**
- Based on [toon-rs](https://github.com/jimmystridh/toon-rs) by Jimmy Stridh
- Features: SIMD string scanning (memchr), stack allocations (smallvec), fast float parsing

### Optimization Techniques

1. **Inlined Primitive Conversions**
   - 85% of TOON data is primitives in dicts/arrays
   - Avoid recursion overhead by inlining Null/Bool/Number/String conversions
   - Only recurse for nested structures

2. **Pre-allocated Collections**
   ```rust
   let mut items = Vec::with_capacity(arr.len());
   Ok(PyList::new(py, items)?.into_any())
   ```

3. **Type-specific Fast Paths**
   - `.is_instance_of::<T>()` for O(1) type checking
   - Direct conversions without dynamic dispatch

4. **SIMD Acceleration**
   - memchr for string scanning (6.5x faster than stdlib)
   - AVX2 support on x86_64

5. **Link-time Optimization**
   ```toml
   [profile.release]
   opt-level = 3
   lto = true
   codegen-units = 1
   ```

---

## Dependencies

### Production
- `pyo3 = "0.27"` - Python bindings
- `serde_json = "1.0"` - JSON handling
- `once_cell = "1.20"` - Static defaults
- `smallvec = "1.13"` - Stack allocations (transitive)
- `toon` - TOON parser by Jimmy Stridh
  - `perf_memchr` - SIMD string scanning
  - `perf_smallvec` - Stack allocations
  - `perf_lexical` - Fast float parsing

### Development
- `criterion = "0.5"` - Micro-benchmarking

---

## Building from Source

### Requirements
- Rust 1.70+
- Python 3.8+
- maturin

### Build Steps

```bash
# Install maturin
pip install maturin

# Development build
maturin develop

# Release build
maturin build --release

# Install wheel
pip install target/wheels/toon_parser-*.whl

# Run tests
python test_toonpy.py
python test_async.py

# Run benchmarks
python benchmark.py
cargo bench
```

---

## Testing

```bash
# Unit tests
python test_toon_parser.py

# Async tests
python test_async.py

# Benchmarks
python benchmark.py

# Micro-benchmarks
cargo bench
```

---

## Credits

### Core Dependencies

**toon-rs** by [Jimmy Stridh](https://github.com/jimmystridh)

This library is built on [toon-rs](https://github.com/jimmystridh/toon-rs), a high-performance Rust implementation of the TOON format parser. The toon-rs library provides:

- Fast TOON ↔ JSON conversion with zero-copy optimizations
- SIMD-accelerated string scanning using memchr
- Memory-efficient stack allocations via smallvec
- Robust error handling and comprehensive testing
- Direct deserialization support with flexible configuration

The performance characteristics of toon-parser are directly derived from the exceptional optimization work in toon-rs.

### Maintainer

**magi8101** (sharmamagi0@gmail.com) - Python bindings and PyO3 integration

### Acknowledgments

- [PyO3](https://github.com/PyO3/pyo3) - Python-Rust FFI bindings
- [TOON Format Specification](https://github.com/toon-format/spec) by Johann Schopplich and contributors
- Rust community for the ecosystem and tooling

---

## License

MIT OR Apache-2.0

---

## Related Projects

- [toon-rs](https://github.com/jimmystridh/toon-rs) - Rust TOON parser (core dependency)
- [toon-llm](https://pypi.org/project/toon-llm/) - Python TOON library with LLM features
- [toon-format](https://pypi.org/project/toon-format/) - Official Python placeholder

---

## Roadmap

- [x] PyO3 0.27 support
- [x] Async API via asyncio
- [x] Comprehensive benchmarking
- [x] Micro-optimization for tabular data
- [ ] Streaming decoder for large files
- [ ] Columnar output for pandas/polars
- [ ] Python 3.13 free-threaded support

---

## Contributing

Issues and PRs welcome! See [PERFORMANCE.md](PERFORMANCE.md) for optimization internals.
