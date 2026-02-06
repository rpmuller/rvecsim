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

## Performance

GHZ state preparation (H gate + CNOT chain) on Apple M1, comparing
rvecsim (Rust + rayon) vs the original Python/NumPy vecsim:

| Qubits | Amplitudes | Python   | Rust    | Speedup  |
|--------|-----------|----------|---------|----------|
| 10     | 1,024     | 2.9 ms   | 0.34 ms | **9x**   |
| 15     | 32,768    | 133 ms   | 0.96 ms | **139x** |
| 18     | 262,144   | 1.02 s   | 4.4 ms  | **231x** |
| 20     | 1,048,576 | 5.20 s   | 16.5 ms | **315x** |
| 22     | 4,194,304 | 20.9 s   | 66.6 ms | **314x** |

## Tests

```bash
cargo test
```

30 tests covering all gates, state construction, operator overloading, measurement, and multi-qubit entangled states (Bell, GHZ).

## Dependencies

- [ndarray](https://crates.io/crates/ndarray) - N-dimensional arrays
- [num-complex](https://crates.io/crates/num-complex) - Complex number types
- [rand](https://crates.io/crates/rand) - Random number generation for measurement
- [rayon](https://crates.io/crates/rayon) - Parallel iterators for gate application
