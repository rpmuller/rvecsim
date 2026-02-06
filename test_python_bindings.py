#!/usr/bin/env python3
"""
Test script for rvecsim Python bindings.

Run with: python test_python_bindings.py
(Make sure to activate the virtual environment and run `maturin develop --features pyo3` first)
"""

from rvecsim import ket

def test_basic_operations():
    """Test basic quantum operations."""
    print("=== Basic Operations ===")

    # Single qubit states
    q0 = ket('0')
    q1 = ket('1')
    print(f"ket('0'): {q0}")
    print(f"ket('1'): {q1}")

    # Hadamard creates superposition
    q_plus = ket('0').H(0)
    print(f"H|0> = |+>: {q_plus}")

    # Pauli gates
    print(f"X|0> = |1>: {ket('0').X(0)}")
    print(f"Z|1> = -|1>: {ket('1').Z(0)}")

    print()

def test_entanglement():
    """Test quantum entanglement."""
    print("=== Entanglement ===")

    # Bell state
    bell = ket('00').H(0).CNOT(0, 1)
    print(f"Bell state (|00> + |11>)/√2: {bell}")

    # GHZ state
    ghz = ket('000').H(0).CNOT(0, 1).CNOT(1, 2)
    print(f"GHZ state (|000> + |111>)/√2: {ghz}")

    print()

def test_operators():
    """Test operator overloading."""
    print("=== Operators ===")

    # Superposition via addition
    q_plus = ket('0') + ket('1')
    print(f"|0> + |1> = |+>: {q_plus}")

    # Subtraction
    q_minus = ket('0') - ket('1')
    print(f"|0> - |1> = |->: {q_minus}")

    # Tensor product
    q_01 = ket('0') * ket('1')
    print(f"|0> ⊗ |1> = |01>: {q_01}")

    print()

def test_measurement():
    """Test quantum measurement."""
    print("=== Measurement ===")

    # Deterministic measurement
    q = ket('0')
    result = q.M(0, 5)
    print(f"Measure |0> 5 times: {result}")

    # Probabilistic measurement
    q = ket('+')
    result = q.M(0, 10)
    print(f"Measure |+> 10 times: {result}")
    print(f"  (Should be mix of 0s and 1s)")

    print()

def test_comparison():
    """Test state comparison."""
    print("=== Comparison ===")

    # Compare with list
    q = ket('++')
    is_close = q.isclose([0.5, 0.5, 0.5, 0.5])
    print(f"ket('++').isclose([0.5, 0.5, 0.5, 0.5]): {is_close}")

    # Compare with another QReg
    is_close = ket('+').isclose(ket('0') + ket('1'))
    print(f"ket('+').isclose(ket('0') + ket('1')): {is_close}")

    print()

def test_properties():
    """Test QReg properties."""
    print("=== Properties ===")

    q = ket('++')
    print(f"Number of qubits: {q.n}")
    print(f"Norm: {q.norm}")
    print(f"Amplitudes: {q.amplitudes}")

    print()

def main():
    """Run all tests."""
    print("\n" + "="*50)
    print("PyO3 Python Bindings for rvecsim")
    print("="*50 + "\n")

    test_basic_operations()
    test_entanglement()
    test_operators()
    test_measurement()
    test_comparison()
    test_properties()

    print("="*50)
    print("✅ All tests completed successfully!")
    print("="*50 + "\n")

if __name__ == "__main__":
    main()
