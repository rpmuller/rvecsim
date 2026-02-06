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

GHZ state preparation (H gate + CNOT chain) on Apple M1:

| Qubits | Amplitudes | Gate time |
|--------|-----------|-----------|
| 10     | 1,024     | 0.3 ms    |
| 15     | 32,768    | 0.8 ms    |
| 18     | 262,144   | 4 ms      |
| 20     | 1,048,576 | 18 ms     |
| 22     | 4,194,304 | 63 ms     |

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
