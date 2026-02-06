#!/usr/bin/env python3
"""
Quantify the GIL's impact on Rayon parallelism.

Compare CPU utilization to understand why Python→Rust is slower than native Rust.
"""

import time
import psutil
import os
from rvecsim import ket

def run_benchmark_with_monitoring(n_qubits, n_trials=3):
    """Run GHZ benchmark while monitoring CPU usage."""
    process = psutil.Process(os.getpid())

    times = []
    cpu_usages = []

    for _ in range(n_trials):
        # Reset CPU measurement
        process.cpu_percent(interval=None)

        # Run computation
        start = time.perf_counter()
        q = ket('0' * n_qubits).H(0)
        for i in range(n_qubits - 1):
            q = q.CNOT(i, i + 1)
        end = time.perf_counter()

        # Measure CPU usage
        cpu = process.cpu_percent(interval=None)

        times.append(end - start)
        cpu_usages.append(cpu)

    avg_time = sum(times) / len(times)
    avg_cpu = sum(cpu_usages) / len(cpu_usages)

    return avg_time, avg_cpu

def main():
    n_cores = psutil.cpu_count(logical=True)

    print("=" * 70)
    print("GIL Impact on Rayon Parallelism")
    print("=" * 70)
    print(f"\nSystem: {n_cores} cores available\n")

    print("Expected behavior:")
    print("  - Native Rust: Should use ~80-100% of all cores (700-800% CPU)")
    print("  - Python→Rust: Limited by GIL (?% CPU)")
    print()

    # Test different circuit sizes
    print("Qubits | Time (ms) | CPU Usage | Cores Used | Efficiency")
    print("-" * 70)

    for n in [15, 18, 20, 22]:
        avg_time, avg_cpu = run_benchmark_with_monitoring(n, n_trials=3)
        cores_used = avg_cpu / 100
        efficiency = (cores_used / n_cores) * 100

        print(f"  {n:2d}   | {avg_time*1000:8.2f}  | {avg_cpu:6.1f}%   | "
              f"{cores_used:5.1f}     | {efficiency:5.1f}%")

    print("\n" + "=" * 70)
    print("Analysis:")
    print("=" * 70)

    # Run one more detailed test
    avg_time, avg_cpu = run_benchmark_with_monitoring(20, n_trials=5)
    cores_used = avg_cpu / 100

    print(f"\n20-qubit GHZ (5 trials):")
    print(f"  Average time: {avg_time*1000:.2f} ms")
    print(f"  Average CPU:  {avg_cpu:.1f}%")
    print(f"  Cores used:   {cores_used:.1f} / {n_cores}")
    print(f"  Efficiency:   {(cores_used/n_cores)*100:.1f}%")

    # Estimate impact
    print(f"\nIf native Rust uses all {n_cores} cores:")
    print(f"  Expected speedup: {n_cores}x")
    print(f"  Python→Rust only uses: {cores_used:.1f}x")
    print(f"  Lost speedup due to GIL: {n_cores/cores_used:.1f}x")

    print("\nConclusion:")
    if cores_used < 3:
        print("  ⚠ GIL is severely limiting parallelism!")
        print("  This explains a significant portion of the 3-20x overhead.")
        print("  Native Rust can use all cores, Python→Rust cannot.")
    elif cores_used < n_cores * 0.5:
        print("  ⚠ GIL is limiting parallelism to ~50% of cores")
        print("  This is a major contributor to the overhead.")
    else:
        print("  ✓ Rayon is using most cores despite the GIL")
        print("  The overhead is primarily from other sources.")

    print("\nPotential fix:")
    print("  Release the GIL during computation using py.allow_threads()")
    print("  This would allow Rayon to use all cores freely.")

if __name__ == "__main__":
    main()
