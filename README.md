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

GHZ state preparation (H gate + CNOT chain) on Apple M1:

| Qubits | Amplitudes | Python/NumPy | Rust (native) | Python→Rust | Speedup vs Python |
|--------|-----------|--------------|---------------|-------------|-------------------|
| 10     | 1,024     | 2.9 ms       | 0.34 ms       | 0.30 ms     | **9.7x**          |
| 15     | 32,768    | 133 ms       | 0.96 ms       | 1.39 ms     | **96x**           |
| 18     | 262,144   | 1.02 s       | 4.4 ms        | 10.6 ms     | **96x**           |
| 20     | 1,048,576 | 5.20 s       | 16.5 ms       | 36.6 ms     | **142x**          |
| 22     | 4,194,304 | 20.9 s       | 66.6 ms       | 174 ms      | **120x**          |

**Notes:**
- **Python/NumPy**: Original vecsim.py (single-threaded)
- **Rust (native)**: Pure Rust with rayon parallelism (all cores)
- **Python→Rust**: Python code calling Rust via PyO3 bindings with GIL released during computation
- PyO3 bindings are only **2-3x slower** than native Rust and **10-142x faster** than pure Python!
- GIL is released during gate operations, allowing Rayon to use ~4 cores (46% efficiency on 8-core M1)

Run the Python→Rust benchmark yourself:
```bash
source .venv/bin/activate
python benchmark_python.py
```

## Tests

```bash
cargo test
```

30 tests covering all gates, state construction, operator overloading, measurement, and multi-qubit entangled states (Bell, GHZ).

## Dependencies

### Rust
- [ndarray](https://crates.io/crates/ndarray) - N-dimensional arrays
- [num-complex](https://crates.io/crates/num-complex) - Complex number types
- [rand](https://crates.io/crates/rand) - Random number generation for measurement
- [rayon](https://crates.io/crates/rayon) - Parallel iterators for gate application

### Python bindings (optional)
- [PyO3](https://pyo3.rs/) - Rust bindings for Python (feature flag: `pyo3`)
- [maturin](https://www.maturin.rs/) - Build tool for Python extension modules
