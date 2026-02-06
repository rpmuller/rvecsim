#!/usr/bin/env python3
"""
Test whether Rayon parallelism is working when called from Python.

The GIL (Global Interpreter Lock) could be preventing Rayon from
using multiple threads effectively.
"""

import time
import psutil
import os
from rvecsim import ket

def monitor_cpu_usage(duration=0.5):
    """Monitor CPU usage over a duration."""
    process = psutil.Process(os.getpid())

    # Take initial measurement
    cpu_percent_start = process.cpu_percent(interval=None)
    time.sleep(duration)
    cpu_percent = process.cpu_percent(interval=None)

    return cpu_percent

def test_parallelism():
    """
    Test if Rayon is using multiple cores.

    If CPU usage > 100%, multiple cores are being used.
    If CPU usage ≈ 100%, only single core is being used.
    """

    print("Testing Rayon Parallelism from Python")
    print("=" * 70)
    print(f"System has {psutil.cpu_count(logical=False)} physical cores, "
          f"{psutil.cpu_count(logical=True)} logical cores\n")

    # Test with a large circuit that should trigger parallelism
    print("Running 20-qubit GHZ state preparation (should use multiple cores):")
    print("-" * 70)

    # Start monitoring
    process = psutil.Process(os.getpid())
    process.cpu_percent(interval=None)  # Initialize

    # Run computation
    start = time.perf_counter()
    q = ket('0' * 20).H(0)
    for i in range(19):
        q = q.CNOT(i, i + 1)
    end = time.perf_counter()

    # Get CPU usage (over the last computation)
    cpu_percent = process.cpu_percent(interval=None)

    elapsed = end - start

    print(f"Time: {elapsed*1000:.2f} ms")
    print(f"CPU usage: {cpu_percent:.1f}%")

    if cpu_percent > 150:
        print("✓ Using multiple cores! (CPU > 150%)")
    elif cpu_percent > 100:
        print("⚠ Limited parallelism (CPU slightly > 100%)")
    else:
        print("✗ Single-threaded execution (CPU ≤ 100%)")

    print("\nInterpretation:")
    print("-" * 70)
    if cpu_percent > 200:
        print("Rayon is successfully using multiple cores despite the GIL.")
        print("The overhead is NOT from single-threaded execution.")
    else:
        print("Rayon may be limited by the GIL or other factors.")
        print("This could explain the 3-20x overhead vs native Rust!")

    # Test multiple iterations to get better measurement
    print("\n" + "=" * 70)
    print("Running multiple iterations for better measurement:")
    print("-" * 70)

    process.cpu_percent(interval=None)  # Reset

    start = time.perf_counter()
    for _ in range(5):
        q = ket('0' * 18).H(0)
        for i in range(17):
            q = q.CNOT(i, i + 1)
    end = time.perf_counter()

    cpu_percent = process.cpu_percent(interval=None)

    print(f"5 iterations of 18-qubit GHZ: {(end-start)*1000:.2f} ms")
    print(f"Average CPU usage: {cpu_percent:.1f}%")

    cores_used = cpu_percent / 100
    print(f"Estimated cores used: {cores_used:.1f}")

if __name__ == "__main__":
    try:
        test_parallelism()
    except ImportError:
        print("Error: psutil not installed. Install with:")
        print("  pip install psutil")
