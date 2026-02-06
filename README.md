# rvecsim

A quantum vector state simulator in Rust, ported from [vecsim](https://github.com/rpmuller/vecsim) (Python/NumPy).

## Features

- **State vector simulation** of quantum circuits with arbitrary qubit counts
- **Single-qubit gates**: X, Y, Z, H, S
- **Two-qubit gates**: CNOT, CPHASE
- **Measurement** with state collapse
- **Multithreaded** gate application via [rayon](https://github.com/rayon-rs/rayon)
- Method chaining: `ket("00").h(0).cnot(0, 1)`

## Quick start

```bash
cargo run --release
```

```
|0>         = 1.0|0>
|0>.X(0)    = 1.0|1>
|0>.H(0)    = 0.707106781186548|0> 0.707106781186548|1>
Bell state  = 0.707106781186548|00> 0.707106781186548|11>
```

## Usage

### Rust

```rust
use rvecsim::ket;

// Create quantum states
let q = ket("0");          // |0>
let bell = ket("00").h(0).cnot(0, 1);  // Bell state

// Tensor product
let two_qubits = ket("0") * ket("1");  // |01>

// Superposition
let plus = ket("0") + ket("1");        // |+>

// Measurement
use rand::SeedableRng;
let mut rng = rand::rngs::StdRng::seed_from_u64(42);
let mut q = ket("+");
let results = q.measure(0, 10, &mut rng);
```

### Python (via PyO3 bindings)

Install with [maturin](https://www.maturin.rs/):

```bash
# Create virtual environment (using uv or venv)
uv venv
source .venv/bin/activate

# Install the package
uv pip install maturin
maturin develop --features pyo3
```

Python API matches the original [vecsim.py](https://github.com/rpmuller/vecsim):

```python
from rvecsim import ket

# Create quantum states
q = ket('0')           # |0>
bell = ket('00').H(0).CNOT(0, 1)  # Bell state

# Operators
two_qubits = ket('0') * ket('1')  # Tensor product |01>
plus = ket('0') + ket('1')        # Superposition |+>
minus = ket('0') - ket('1')       # |->

# Gates (uppercase, method chaining)
ghz = ket('000').H(0).CNOT(0, 1).CNOT(1, 2)

# Measurement (returns list of 0/1 values)
q = ket('+')
results = q.M(0, ntimes=10)  # Measure qubit 0 ten times

# Comparison with lists
assert ket('++').isclose([0.5, 0.5, 0.5, 0.5])

# Properties
print(f"Qubits: {q.n}, Norm: {q.norm}")
print(f"Amplitudes: {q.amplitudes}")
```

Run the test script:

```bash
python test_python_bindings.py
```

## Performance

GHZ state preparation (H gate + CNOT chain) on Apple M1 (8 cores):

| Qubits | Amplitudes | Python/NumPy | Rust (native) | Python→Rust | Speedup vs Python |
|--------|-----------|--------------|---------------|-------------|-------------------|
| 10     | 1,024     | 2.9 ms       | 0.34 ms       | 0.30 ms     | **9.7x**          |
| 15     | 32,768    | 133 ms       | 0.96 ms       | 1.39 ms     | **96x**           |
| 18     | 262,144   | 1.02 s       | 4.4 ms        | 10.6 ms     | **96x**           |
| 20     | 1,048,576 | 5.20 s       | 16.5 ms       | 36.6 ms     | **142x**          |
| 22     | 4,194,304 | 20.9 s       | 66.6 ms       | 174 ms      | **120x**          |

### Performance Notes

#### Python/NumPy (Baseline)
- Original `vecsim.py` using NumPy for matrix operations
- Single-threaded execution
- Slowest but most compatible

#### Rust (Native) - Best Performance
- Pure Rust implementation with Rayon parallelism
- Uses all 8 cores efficiently (~700-800% CPU)
- **Best choice** for maximum performance

#### Python→Rust - Best of Both Worlds
- Python code calling Rust via PyO3 bindings
- **10-142x faster** than pure Python/NumPy
- **Only 2.6x slower** than native Rust
- GIL is released during gate operations using `py.allow_threads()`
- Rayon uses ~4 cores (46% efficiency on 8-core M1)
- **Best choice** for Python users wanting major speedup with zero code changes

### Why Python→Rust is Fast

The PyO3 bindings achieve near-native Rust performance through several optimizations:

1. **GIL Release**: Python's Global Interpreter Lock is released during gate computations, allowing Rayon to use multiple CPU cores
2. **Native Rust Computation**: All matrix operations run in compiled Rust code, not Python
3. **Parallel Gate Application**: Rayon parallelizes gate operations across available cores
4. **Minimal FFI Overhead**: The clone-compute-update pattern has negligible overhead for large quantum states

The remaining 2.6x gap to native Rust comes from:
- **Cannot optimize across language boundary** (~2x): Each gate method call crosses the FFI boundary
- **Clone overhead** (~1.3x): State vector is cloned before/after computation for safety

For quantum circuits with 15+ qubits, the computation dominates and Python→Rust performance approaches native Rust.

### Build Instructions

**Rust native:**
```bash
cargo build --release
cargo test
cargo run --release
```

**Python bindings:**
```bash
# Setup environment
uv venv
source .venv/bin/activate
uv pip install maturin

# Build and install (development mode)
maturin develop --features pyo3 --release

# Run benchmarks
python benchmark_python.py
python test_gil_impact.py
```

**Benchmark scripts:**
- `benchmark_python.py` - GHZ state performance benchmarks
- `test_gil_impact.py` - Analyze GIL impact and core utilization
- `test_parallelism.py` - Monitor CPU usage during execution
- `benchmark_overhead_v2.py` - Detailed FFI overhead analysis

## Tests

```bash
cargo test
```

30 tests covering all gates, state construction, operator overloading, measurement, and multi-qubit entangled states (Bell, GHZ).

## PyO3 Optimization Details

The Python bindings achieve excellent performance through careful optimization:

### GIL Release Pattern

All gate methods use this pattern to release Python's Global Interpreter Lock:

```rust
fn H(slf: Py<Self>, target: usize, py: Python<'_>) -> PyResult<Py<Self>> {
    // 1. Validate and clone while holding GIL
    let mut inner = {
        let this = slf.borrow(py);
        // validation...
        this.inner.clone()
    };

    // 2. Release GIL during computation
    py.allow_threads(|| {
        inner.apply1q(&crate::H_GATE, target);
    });

    // 3. Update Python object
    {
        let mut this = slf.borrow_mut(py);
        this.inner = inner;
    }

    Ok(slf)
}
```

This allows:
- Rayon to use multiple cores without GIL contention
- Other Python threads to run during computation
- Safe access to Rust data without GIL-related deadlocks

### Performance Evolution

**Initial version (GIL held):**
- 22-qubit GHZ: 1360 ms
- CPU usage: ~200% (only 2 cores)
- Speedup vs Python: 15x

**Optimized version (GIL released):**
- 22-qubit GHZ: 174 ms
- CPU usage: ~370% (~4 cores)
- Speedup vs Python: 120x
- **Improvement: 8x faster!**

See `GIL_RELEASE_OPTIMIZATION.md` for complete analysis.

## Dependencies

### Rust
- [ndarray](https://crates.io/crates/ndarray) - N-dimensional arrays
- [num-complex](https://crates.io/crates/num-complex) - Complex number types
- [rand](https://crates.io/crates/rand) - Random number generation for measurement
- [rayon](https://crates.io/crates/rayon) - Parallel iterators for gate application

### Python bindings (optional)
- [PyO3](https://pyo3.rs/) - Rust bindings for Python (feature flag: `pyo3`)
- [maturin](https://www.maturin.rs/) - Build tool for Python extension modules

## Documentation

- `README.md` - This file (overview and usage)
- `CLAUDE.md` - Developer guide for working with this codebase
- `GIL_RELEASE_OPTIMIZATION.md` - Detailed analysis of GIL release optimization
- `GIL_FINDINGS.md` - Investigation of GIL impact on parallelism
- `explain_overhead.md` - FFI overhead sources and analysis
