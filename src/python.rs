// PyO3 Python bindings for rvecsim
//
// Provides a Python API matching the original vecsim.py:
// - ket('0') constructor
// - Gate methods: X, Y, Z, H, S, CNOT, CPHASE (uppercase, method chaining)
// - M for measurement
// - Operators: +, -, *
// - isclose() accepting QReg or list

#![allow(non_snake_case)]

use crate::{ket as rust_ket, QReg as RustQReg};
use num_complex::Complex64;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rand::thread_rng;

/// Python wrapper for QReg
#[pyclass(name = "QReg")]
pub struct PyQReg {
    inner: RustQReg,
}

#[pymethods]
impl PyQReg {
    // ---- Properties ----

    /// Number of qubits
    #[getter]
    fn n(&self) -> usize {
        self.inner.n
    }

    /// State vector amplitudes as list of complex numbers
    #[getter]
    fn amplitudes(&self) -> Vec<Complex64> {
        self.inner.v.to_vec()
    }

    /// L2 norm of the state vector
    #[getter]
    fn norm(&self) -> f64 {
        self.inner.norm()
    }

    // ---- String representations ----

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("QReg({})", self.inner.terms())
    }

    fn terms(&self) -> String {
        self.inner.terms()
    }

    // ---- Single-qubit gates ----

    /// Apply Pauli-X (NOT) gate to target qubit
    fn X(slf: Py<Self>, target: usize, py: Python<'_>) -> PyResult<Py<Self>> {
        {
            let this = slf.borrow(py);
            if target >= this.inner.n {
                return Err(PyValueError::new_err(format!(
                    "Invalid target qubit {}. Must be in [0, {})",
                    target, this.inner.n
                )));
            }
        }
        let mut this = slf.borrow_mut(py);
        this.inner.apply1q(&crate::X_GATE, target);
        drop(this);
        Ok(slf)
    }

    /// Apply Pauli-Y gate to target qubit
    fn Y(slf: Py<Self>, target: usize, py: Python<'_>) -> PyResult<Py<Self>> {
        {
            let this = slf.borrow(py);
            if target >= this.inner.n {
                return Err(PyValueError::new_err(format!(
                    "Invalid target qubit {}. Must be in [0, {})",
                    target, this.inner.n
                )));
            }
        }
        let mut this = slf.borrow_mut(py);
        this.inner.apply1q(&crate::Y_GATE, target);
        drop(this);
        Ok(slf)
    }

    /// Apply Pauli-Z gate to target qubit
    fn Z(slf: Py<Self>, target: usize, py: Python<'_>) -> PyResult<Py<Self>> {
        {
            let this = slf.borrow(py);
            if target >= this.inner.n {
                return Err(PyValueError::new_err(format!(
                    "Invalid target qubit {}. Must be in [0, {})",
                    target, this.inner.n
                )));
            }
        }
        let mut this = slf.borrow_mut(py);
        this.inner.apply1q(&crate::Z_GATE, target);
        drop(this);
        Ok(slf)
    }

    /// Apply Hadamard gate to target qubit
    fn H(slf: Py<Self>, target: usize, py: Python<'_>) -> PyResult<Py<Self>> {
        {
            let this = slf.borrow(py);
            if target >= this.inner.n {
                return Err(PyValueError::new_err(format!(
                    "Invalid target qubit {}. Must be in [0, {})",
                    target, this.inner.n
                )));
            }
        }
        let mut this = slf.borrow_mut(py);
        this.inner.apply1q(&crate::H_GATE, target);
        drop(this);
        Ok(slf)
    }

    /// Apply S (phase) gate to target qubit
    fn S(slf: Py<Self>, target: usize, py: Python<'_>) -> PyResult<Py<Self>> {
        {
            let this = slf.borrow(py);
            if target >= this.inner.n {
                return Err(PyValueError::new_err(format!(
                    "Invalid target qubit {}. Must be in [0, {})",
                    target, this.inner.n
                )));
            }
        }
        let mut this = slf.borrow_mut(py);
        this.inner.apply1q(&crate::S_GATE, target);
        drop(this);
        Ok(slf)
    }

    // ---- Two-qubit gates ----

    /// Apply controlled-NOT gate
    fn CNOT(slf: Py<Self>, control: usize, target: usize, py: Python<'_>) -> PyResult<Py<Self>> {
        {
            let this = slf.borrow(py);
            if control >= this.inner.n {
                return Err(PyValueError::new_err(format!(
                    "Invalid control qubit {}. Must be in [0, {})",
                    control, this.inner.n
                )));
            }
            if target >= this.inner.n {
                return Err(PyValueError::new_err(format!(
                    "Invalid target qubit {}. Must be in [0, {})",
                    target, this.inner.n
                )));
            }
            if control == target {
                return Err(PyValueError::new_err(
                    "Control and target must be different qubits",
                ));
            }
        }
        let mut this = slf.borrow_mut(py);
        this.inner.apply2q(&crate::CNOT_GATE, control, target);
        drop(this);
        Ok(slf)
    }

    /// Apply controlled-phase gate
    fn CPHASE(slf: Py<Self>, control: usize, target: usize, py: Python<'_>) -> PyResult<Py<Self>> {
        {
            let this = slf.borrow(py);
            if control >= this.inner.n {
                return Err(PyValueError::new_err(format!(
                    "Invalid control qubit {}. Must be in [0, {})",
                    control, this.inner.n
                )));
            }
            if target >= this.inner.n {
                return Err(PyValueError::new_err(format!(
                    "Invalid target qubit {}. Must be in [0, {})",
                    target, this.inner.n
                )));
            }
            if control == target {
                return Err(PyValueError::new_err(
                    "Control and target must be different qubits",
                ));
            }
        }
        let mut this = slf.borrow_mut(py);
        this.inner.apply2q(&crate::CPHASE_GATE, control, target);
        drop(this);
        Ok(slf)
    }

    // ---- Measurement ----

    /// Measure qubit i, ntimes times (default 1)
    /// Returns list of measurement outcomes (0 or 1)
    #[pyo3(signature = (i, ntimes=1))]
    fn M(&mut self, i: usize, ntimes: usize) -> PyResult<Vec<usize>> {
        if i >= self.inner.n {
            return Err(PyValueError::new_err(format!(
                "Invalid qubit {}. Must be in [0, {})",
                i, self.inner.n
            )));
        }
        let mut rng = thread_rng();
        Ok(self.inner.measure(i, ntimes, &mut rng))
    }

    // ---- Comparison ----

    /// Check if this state is close to another QReg or a list of values
    fn isclose(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        // Try to extract as PyQReg first
        if let Ok(other_qreg) = other.extract::<PyRef<PyQReg>>() {
            return Ok(self.inner.isclose(&other_qreg.inner));
        }

        // Try to extract as list of complex numbers
        if let Ok(complex_list) = other.extract::<Vec<Complex64>>() {
            if self.inner.v.len() != complex_list.len() {
                return Ok(false);
            }
            return Ok(self
                .inner
                .v
                .iter()
                .zip(complex_list.iter())
                .all(|(a, b)| (a - b).norm() < 1e-5));
        }

        // Try to extract as list of floats (real numbers)
        if let Ok(float_list) = other.extract::<Vec<f64>>() {
            return Ok(self.inner.isclose_slice(&float_list));
        }

        Err(PyValueError::new_err(
            "isclose() argument must be QReg or list of numbers",
        ))
    }

    // ---- Operators ----

    /// Superposition: (|a> + |b>) / sqrt(2)
    fn __add__(&self, other: &PyQReg) -> PyQReg {
        PyQReg {
            inner: self.inner.clone() + other.inner.clone(),
        }
    }

    /// Subtraction: (|a> - |b>) / sqrt(2)
    fn __sub__(&self, other: &PyQReg) -> PyQReg {
        PyQReg {
            inner: self.inner.clone() - other.inner.clone(),
        }
    }

    /// Tensor product: |a> ⊗ |b>
    fn __mul__(&self, other: &PyQReg) -> PyQReg {
        PyQReg {
            inner: self.inner.clone() * other.inner.clone(),
        }
    }
}

/// Create a quantum ket state from a string specification
///
/// Characters: '0' = |0>, '1' = |1>, '+' = |+>, '-' = |->
///
/// Examples: ket('0'), ket('1'), ket('00'), ket('++'), ket('101')
///
/// Args:
///     vecstring: String specifying the quantum state (default: '0')
///
/// Returns:
///     QReg: The quantum register in the specified state
#[pyfunction]
#[pyo3(signature = (vecstring="0"))]
fn ket(vecstring: &str) -> PyResult<PyQReg> {
    if vecstring.is_empty() {
        return Err(PyValueError::new_err("vecstring cannot be empty"));
    }

    let valid = ['0', '1', '+', '-'];
    for ch in vecstring.chars() {
        if !valid.contains(&ch) {
            return Err(PyValueError::new_err(format!(
                "Invalid character '{}' in vecstring. Valid: 0, 1, +, -",
                ch
            )));
        }
    }

    Ok(PyQReg {
        inner: rust_ket(vecstring),
    })
}

/// Python module definition
#[pymodule]
fn rvecsim(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyQReg>()?;
    m.add_function(wrap_pyfunction!(ket, m)?)?;
    Ok(())
}
