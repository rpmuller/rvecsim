# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rvecsim is a Rust port of the Python `vecsim` quantum vector state simulator (located at `../vecsim/vecsim.py`). It simulates quantum registers using state vector representation with complex-valued arrays.

## Build and Test Commands

```bash
cargo build              # Build the project
cargo test               # Run all 17 tests
cargo run                # Run the demo binary
cargo build --release    # Optimized build
```

Rust 1.93+ required (edition 2024). Installed via rustup; you may need `source "$HOME/.cargo/env"` if cargo is not on PATH.

## Architecture

All library code is in `src/lib.rs`. The binary entry point is `src/main.rs`.

### Core Types

- **`QReg`** (`src/lib.rs`): Central struct holding a quantum state as `Array1<Complex64>` plus qubit count `n`.
  - Gate methods (`.x()`, `.y()`, `.z()`, `.h()`, `.s()`, `.cnot()`, `.cphase()`) consume self and return Self for chaining: `ket("00").h(0).cnot(0, 1)`
  - `apply1q()`/`apply2q()` take `&mut self` for in-place mutation
  - `measure()` takes `&mut impl Rng` for testability with seeded RNGs
  - Implements `Add` (superposition), `Sub`, `Mul` (tensor product), `Display`

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

- **ndarray** 0.16: Array types (`Array1`, `Array2`)
- **num-complex** 0.4: `Complex64` type
- **rand** 0.8: RNG for measurement (note: `rng.r#gen()` needed since `gen` is reserved in edition 2024)
- **rayon** 1: Parallel iterators for gate application

## Key Implementation Details

- Qubit indexing is zero-based; qubit 0 is the rightmost bit in binary representation
- States are auto-normalized on construction and after measurement
- Complex amplitudes with magnitude < 1e-8 are treated as zero
- Display formatting uses `round_sigfigs(x, 15)` to eliminate floating-point ULP noise (e.g., `1.0000000000000002` → `1.0`)
- `format_real()` ensures floats always have a decimal point (matching Python's `str(float)`)

## Relationship to Python vecsim

This is a direct port of `../vecsim/vecsim.py`. The Python version uses NumPy arrays and doctests. The Rust version mirrors the same API and behavior, with these adaptations:
- Gate methods consume self by value (for chaining) rather than returning `&mut self`
- `measure()` takes an explicit RNG parameter instead of using a global RNG
- Parallel gate application via rayon (Python version is single-threaded)
