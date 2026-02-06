# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rvecsim is a Rust port of the Python `vecsim` quantum vector state simulator (located at `../vecsim/vecsim.py`). It simulates quantum registers using state vector representation with complex-valued arrays.

## Build and Test Commands

### Rust

```bash
cargo build              # Build the project
cargo test               # Run all 30 tests
cargo run                # Run the demo binary
cargo build --release    # Optimized build
```

Rust 1.93+ required (edition 2024). Installed via rustup; you may need `source "$HOME/.cargo/env"` if cargo is not on PATH.

### Python Bindings

```bash
# Setup (using uv for package management)
uv venv
source .venv/bin/activate
uv pip install maturin

# Build and install Python extension
maturin develop --features pyo3

# Test Python bindings
python test_python_bindings.py
```

## Architecture

- **`src/lib.rs`**: Core library code (Rust QReg implementation)
- **`src/main.rs`**: Binary entry point (demo)
- **`src/python.rs`**: PyO3 Python bindings (conditionally compiled with `pyo3` feature)

### Core Types

- **`QReg`** (`src/lib.rs`): Central struct holding a quantum state as `Array1<Complex64>` plus qubit count `n`.
  - Gate methods (`.x()`, `.y()`, `.z()`, `.h()`, `.s()`, `.cnot()`, `.cphase()`) consume self and return Self for chaining: `ket("00").h(0).cnot(0, 1)`
  - `apply1q()`/`apply2q()` take `&mut self` for in-place mutation
  - `measure()` takes `&mut impl Rng` for testability with seeded RNGs
  - Implements `Add` (superposition), `Sub`, `Mul` (tensor product), `Display`, `Clone`

- **`PyQReg`** (`src/python.rs`): Python wrapper around `QReg` for PyO3 bindings
  - Gate methods (`.X()`, `.Y()`, `.Z()`, `.H()`, `.S()`, `.CNOT()`, `.CPHASE()`) use uppercase names to match Python API
  - Methods use `Py<Self>` pattern for true method chaining (mutate in-place, return same object)
  - `.M()` for measurement (uses `thread_rng()` internally)
  - Implements `__add__`, `__sub__`, `__mul__` for operator overloading
  - `.isclose()` accepts `QReg`, `list[float]`, or `list[complex]`

### Gate Matrices

Defined as `LazyLock<Array2<Complex64>>` statics: `X_GATE`, `Y_GATE`, `Z_GATE`, `H_GATE`, `S_GATE`, `CNOT_GATE`, `CPHASE_GATE`, `I_GATE`.

### Parallelism

`apply1q` and `apply2q` use **rayon** `into_par_iter` for multithreaded gate application. A `SendPtr<T>` wrapper provides safe access to non-overlapping array elements across threads. The `read()`/`write()` methods on `SendPtr` are necessary to avoid Rust 2024's precise field capture exposing the raw pointer.

`measure()` is sequential (each measurement depends on the previous collapse).

### Key Utilities

- `ket(vecstring)`: Constructs quantum states from strings ("0", "1", "+", "-", "01", "++", etc.)
- `nqubits(vl)`: Returns number of qubits from vector length
- `conjugate_index(i, b)`: Flips bit b in index i (XOR)
- `kron()`: Kronecker product for 1D arrays (tensor product)

## Dependencies

### Rust
- **ndarray** 0.16: Array types (`Array1`, `Array2`)
- **num-complex** 0.4: `Complex64` type
- **rand** 0.8: RNG for measurement (note: `rng.r#gen()` needed since `gen` is reserved in edition 2024)
- **rayon** 1: Parallel iterators for gate application

### Python Bindings (optional, feature flag `pyo3`)
- **pyo3** 0.24: Rust bindings for Python, with `num-complex` feature for automatic `Complex64` conversion
- **maturin** 1.11+: Build tool for creating Python extension modules

## Key Implementation Details

- Qubit indexing is zero-based; qubit 0 is the rightmost bit in binary representation
- States are auto-normalized on construction and after measurement
- Complex amplitudes with magnitude < 1e-8 are treated as zero
- Display formatting uses `round_sigfigs(x, 15)` to eliminate floating-point ULP noise (e.g., `1.0000000000000002` → `1.0`)
- `format_real()` ensures floats always have a decimal point (matching Python's `str(float)`)

## Relationship to Python vecsim

This is a direct port of `../vecsim/vecsim.py`. The Python version uses NumPy arrays and doctests.

### Rust API differences from Python:
- Gate methods consume self by value (for chaining) rather than returning `&mut self`
- `measure()` takes an explicit RNG parameter instead of using a global RNG
- Parallel gate application via rayon (Python version is single-threaded)

### PyO3 Python bindings:
The `src/python.rs` module provides Python bindings that closely match the original `vecsim.py` API:
- **Uppercase gate methods**: `.X()`, `.Y()`, `.Z()`, `.H()`, `.S()`, `.CNOT()`, `.CPHASE()` (matching Python convention)
- **Method chaining**: Uses `Py<Self>` pattern to mutate in-place and return the same Python object
- **Measurement**: `.M(i, ntimes=1)` with default argument, uses `thread_rng()` internally
- **Operators**: `+`, `-`, `*` work via `__add__`, `__sub__`, `__mul__` (clones operands)
- **Comparison**: `.isclose()` accepts `QReg`, `list[float]`, or `list[complex]`
- **Properties**: `.n`, `.norm`, `.amplitudes`
- **No NumPy dependency**: All conversions use Python native types (`list`, `complex`)
