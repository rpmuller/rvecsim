#!/usr/bin/env python3
"""
Benchmark GHZ state preparation using Python bindings to Rust rvecsim.
Compares with the Rust and Python/NumPy benchmarks in the README.
"""

import time
from rvecsim import ket

def benchmark_ghz(n_qubits, n_trials=5):
    """
    Benchmark GHZ state preparation: H(0), CNOT(0,1), CNOT(1,2), ..., CNOT(n-2, n-1)

    Args:
        n_qubits: Number of qubits
        n_trials: Number of trials to average

    Returns:
        Average time in seconds
    """
    times = []

    for _ in range(n_trials):
        # Create initial state
        initial_state = '0' * n_qubits

        # Time the GHZ preparation
        start = time.perf_counter()
        q = ket(initial_state).H(0)
        for i in range(n_qubits - 1):
            q = q.CNOT(i, i + 1)
        end = time.perf_counter()

        times.append(end - start)

    return sum(times) / len(times)

def format_time(seconds):
    """Format time in appropriate units."""
    if seconds < 1e-6:
        return f"{seconds * 1e9:.2f} ns"
    elif seconds < 1e-3:
        return f"{seconds * 1e6:.2f} µs"
    elif seconds < 1:
        return f"{seconds * 1e3:.2f} ms"
    else:
        return f"{seconds:.2f} s"

def main():
    print("Benchmarking GHZ state preparation (Python → Rust via PyO3)")
    print("=" * 60)

    qubit_counts = [10, 15, 18, 20, 22]

    print(f"\n{'Qubits':<8} {'Amplitudes':<12} {'Avg Time':<12}")
    print("-" * 40)

    results = []
    for n in qubit_counts:
        amplitudes = 2 ** n
        avg_time = benchmark_ghz(n, n_trials=5)
        results.append((n, amplitudes, avg_time))
        print(f"{n:<8} {amplitudes:,<12} {format_time(avg_time):<12}")

    print("\n" + "=" * 60)
    print("\nResults for README table:")
    print("-" * 40)
    for n, amplitudes, avg_time in results:
        print(f"| {n:<6} | {amplitudes:,<9} | ... | ... | {format_time(avg_time):<7} |")

if __name__ == "__main__":
    main()
