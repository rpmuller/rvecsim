// rvecsim - quantum vector state simulator in Rust
// Port of vecsim.py

use ndarray::{array, Array1, Array2};
use num_complex::Complex64;
use rand::Rng;
use rayon::prelude::*;
use std::fmt;
use std::ops::{Add, Mul, Sub};
use std::sync::LazyLock;

// ---- Thread-safe pointer wrapper for parallel mutation ----

/// Wrapper around a raw pointer that is Send+Sync.
/// SAFETY: Caller must ensure no two threads access the same index.
#[derive(Copy, Clone)]
struct SendPtr<T>(*mut T);
unsafe impl<T> Send for SendPtr<T> {}
unsafe impl<T> Sync for SendPtr<T> {}

impl<T: Copy> SendPtr<T> {
    #[inline]
    unsafe fn read(&self, i: usize) -> T {
        unsafe { *self.0.add(i) }
    }
    #[inline]
    unsafe fn write(&self, i: usize, val: T) {
        unsafe { *self.0.add(i) = val; }
    }
}

// ---- Complex Constants ----

const ZERO: Complex64 = Complex64::new(0.0, 0.0);
const ONE: Complex64 = Complex64::new(1.0, 0.0);
const NEG1: Complex64 = Complex64::new(-1.0, 0.0);
const IM: Complex64 = Complex64::new(0.0, 1.0);
const NEG_IM: Complex64 = Complex64::new(0.0, -1.0);
const S2: Complex64 = Complex64::new(std::f64::consts::FRAC_1_SQRT_2, 0.0);
const NEG_S2: Complex64 = Complex64::new(-std::f64::consts::FRAC_1_SQRT_2, 0.0);

// ---- Utility Functions ----

/// Return the number of qubits for a state vector of length `vl`.
pub fn nqubits(vl: usize) -> usize {
    assert!(vl > 0, "Vector length must be positive, got {vl}");
    (vl as f64).log2().floor() as usize
}

/// Flip bit `b` in index `i` using XOR.
pub fn conjugate_index(i: usize, b: usize) -> usize {
    i ^ (1 << b)
}

// ---- Gate Matrices ----

pub static I_GATE: LazyLock<Array2<Complex64>> = LazyLock::new(|| {
    array![[ONE, ZERO], [ZERO, ONE]]
});

pub static X_GATE: LazyLock<Array2<Complex64>> = LazyLock::new(|| {
    array![[ZERO, ONE], [ONE, ZERO]]
});

pub static Y_GATE: LazyLock<Array2<Complex64>> = LazyLock::new(|| {
    array![[ZERO, NEG_IM], [IM, ZERO]]
});

pub static Z_GATE: LazyLock<Array2<Complex64>> = LazyLock::new(|| {
    array![[ONE, ZERO], [ZERO, NEG1]]
});

pub static H_GATE: LazyLock<Array2<Complex64>> = LazyLock::new(|| {
    array![[S2, S2], [S2, NEG_S2]]
});

pub static S_GATE: LazyLock<Array2<Complex64>> = LazyLock::new(|| {
    array![[ONE, ZERO], [ZERO, IM]]
});

pub static CNOT_GATE: LazyLock<Array2<Complex64>> = LazyLock::new(|| {
    array![
        [ONE,  ZERO, ZERO, ZERO],
        [ZERO, ONE,  ZERO, ZERO],
        [ZERO, ZERO, ZERO, ONE ],
        [ZERO, ZERO, ONE,  ZERO]
    ]
});

pub static CPHASE_GATE: LazyLock<Array2<Complex64>> = LazyLock::new(|| {
    array![
        [ONE,  ZERO, ZERO, ZERO],
        [ZERO, ONE,  ZERO, ZERO],
        [ZERO, ZERO, ONE,  ZERO],
        [ZERO, ZERO, ZERO, NEG1]
    ]
});

// ---- Kronecker Product ----

fn kron(a: &Array1<Complex64>, b: &Array1<Complex64>) -> Array1<Complex64> {
    let (la, lb) = (a.len(), b.len());
    let mut result = Array1::zeros(la * lb);
    for i in 0..la {
        for j in 0..lb {
            result[i * lb + j] = a[i] * b[j];
        }
    }
    result
}

// ---- Formatting ----

/// Round to `n` significant figures to eliminate floating-point ULP noise.
fn round_sigfigs(x: f64, n: i32) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    let d = x.abs().log10().ceil() as i32;
    let power = 10f64.powi(n - d);
    (x * power).round() / power
}

/// Format a float to always include a decimal point (matching Python's str(float)).
fn format_real(x: f64) -> String {
    let x = round_sigfigs(x, 15);
    let s = format!("{}", x);
    if !s.contains('.') && !s.contains('e') && !s.contains('E') {
        format!("{s}.0")
    } else {
        s
    }
}

/// Format a complex coefficient for display.
/// Returns just the real part if purely real, otherwise the full complex number.
fn qcoef(a: Complex64) -> String {
    let re = round_sigfigs(a.re, 15);
    let im = round_sigfigs(a.im, 15);
    if im.abs() < 1e-8 {
        format_real(re)
    } else {
        format!("{}+{}i", format_real(re), format_real(im))
    }
}

/// Format a single term of a quantum state as "coef|binary>".
fn qterm(i: usize, qi: Complex64, n: usize) -> String {
    format!("{}|{:0>width$b}>", qcoef(qi), i, width = n)
}

// ---- Quantum Register ----

#[derive(Clone)]
pub struct QReg {
    pub v: Array1<Complex64>,
    pub n: usize,
}

impl QReg {
    /// Create a new quantum register from a vector of complex amplitudes.
    /// The vector length must be a power of 2. The state is normalized.
    pub fn new(register: Vec<Complex64>) -> Self {
        assert!(!register.is_empty(), "Register cannot be empty");
        let n = register.len();
        assert!(
            n.is_power_of_two(),
            "Register length must be power of 2, got {n}"
        );
        let v = Array1::from_vec(register);
        let mut qreg = QReg {
            v,
            n: nqubits(n),
        };
        qreg.normalize();
        qreg
    }

    /// Create a quantum register from an existing Array1.
    fn from_array(v: Array1<Complex64>) -> Self {
        let n = v.len();
        assert!(n > 0 && n.is_power_of_two());
        let mut qreg = QReg {
            v,
            n: nqubits(n),
        };
        qreg.normalize();
        qreg
    }

    /// Calculate the L2 norm of the state vector.
    pub fn norm(&self) -> f64 {
        self.v.iter().map(|x| x.norm_sqr()).sum::<f64>().sqrt()
    }

    /// Normalize the state vector in-place.
    pub fn normalize(&mut self) {
        let norm = self.norm();
        assert!(norm > 1e-10, "Cannot normalize zero vector");
        self.v.mapv_inplace(|x| x / norm);
    }

    /// Return string representation of significant terms in the quantum state.
    pub fn terms(&self) -> String {
        self.v
            .iter()
            .enumerate()
            .filter(|(_, qi)| qi.norm() > 1e-8)
            .map(|(i, &qi)| qterm(i, qi, self.n))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Apply a single-qubit gate matrix to the target qubit.
    pub fn apply1q(&mut self, m: &Array2<Complex64>, target: usize) -> &mut Self {
        assert!(
            target < self.n,
            "Invalid target qubit {target}. Must be in [0, {})",
            self.n
        );
        let m00 = m[[0, 0]];
        let m01 = m[[0, 1]];
        let m10 = m[[1, 0]];
        let m11 = m[[1, 1]];
        let len = self.v.len();
        let ptr = SendPtr(self.v.as_mut_ptr());
        // SAFETY: Each (i, j) pair is unique and non-overlapping.
        // For target bit b, pairs are (i, i|(1<<b)) for all i where bit b is 0.
        // No two iterations touch the same array element.
        (0..len).into_par_iter().for_each(move |i| {
            let j = conjugate_index(i, target);
            if i > j {
                return;
            }
            unsafe {
                let qi = ptr.read(i);
                let qj = ptr.read(j);
                if qi.norm() + qj.norm() < 1e-8 {
                    return;
                }
                ptr.write(i, m00 * qi + m01 * qj);
                ptr.write(j, m10 * qi + m11 * qj);
            }
        });
        self
    }

    /// Apply a two-qubit gate matrix to the control and target qubits.
    pub fn apply2q(
        &mut self,
        m: &Array2<Complex64>,
        control: usize,
        target: usize,
    ) -> &mut Self {
        assert!(
            control < self.n,
            "Invalid control qubit {control}. Must be in [0, {})",
            self.n
        );
        assert!(
            target < self.n,
            "Invalid target qubit {target}. Must be in [0, {})",
            self.n
        );
        assert!(control != target, "Control and target must be different qubits");

        let mv: [[Complex64; 4]; 4] = [
            [m[[0, 0]], m[[0, 1]], m[[0, 2]], m[[0, 3]]],
            [m[[1, 0]], m[[1, 1]], m[[1, 2]], m[[1, 3]]],
            [m[[2, 0]], m[[2, 1]], m[[2, 2]], m[[2, 3]]],
            [m[[3, 0]], m[[3, 1]], m[[3, 2]], m[[3, 3]]],
        ];
        let len = self.v.len();
        let ptr = SendPtr(self.v.as_mut_ptr());
        // SAFETY: Each (i, j, k, l) group is unique and non-overlapping.
        // The two bit positions (control, target) partition all 2^n indices
        // into groups of 4 that don't overlap between iterations.
        (0..len).into_par_iter().for_each(move |i| {
            let j = conjugate_index(i, target);
            if i > j {
                return;
            }
            let k = conjugate_index(i, control);
            if i > k {
                return;
            }
            let l = conjugate_index(j, control);

            unsafe {
                let (qi, qj, qk, ql) = (
                    ptr.read(i),
                    ptr.read(j),
                    ptr.read(k),
                    ptr.read(l),
                );
                if qi.norm() + qj.norm() + qk.norm() + ql.norm() < 1e-8 {
                    return;
                }

                ptr.write(i,
                    mv[0][0] * qi + mv[0][1] * qj + mv[0][2] * qk + mv[0][3] * ql);
                ptr.write(j,
                    mv[1][0] * qi + mv[1][1] * qj + mv[1][2] * qk + mv[1][3] * ql);
                ptr.write(k,
                    mv[2][0] * qi + mv[2][1] * qj + mv[2][2] * qk + mv[2][3] * ql);
                ptr.write(l,
                    mv[3][0] * qi + mv[3][1] * qj + mv[3][2] * qk + mv[3][3] * ql);
            }
        });
        self
    }

    /// Check if this quantum state is close to another.
    pub fn isclose(&self, other: &QReg) -> bool {
        if self.v.len() != other.v.len() {
            return false;
        }
        self.v
            .iter()
            .zip(other.v.iter())
            .all(|(a, b)| (a - b).norm() < 1e-5)
    }

    /// Check if this quantum state is close to a slice of f64 values (treated as real).
    pub fn isclose_slice(&self, other: &[f64]) -> bool {
        if self.v.len() != other.len() {
            return false;
        }
        self.v
            .iter()
            .zip(other.iter())
            .all(|(a, b)| (a - b).norm() < 1e-5)
    }

    // ---- Gate methods (consume self for chaining) ----

    /// Apply Pauli-X (NOT) gate to target qubit.
    pub fn x(mut self, target: usize) -> Self {
        self.apply1q(&X_GATE, target);
        self
    }

    /// Apply Pauli-Y gate to target qubit.
    pub fn y(mut self, target: usize) -> Self {
        self.apply1q(&Y_GATE, target);
        self
    }

    /// Apply Pauli-Z gate to target qubit.
    pub fn z(mut self, target: usize) -> Self {
        self.apply1q(&Z_GATE, target);
        self
    }

    /// Apply Hadamard gate to target qubit.
    pub fn h(mut self, target: usize) -> Self {
        self.apply1q(&H_GATE, target);
        self
    }

    /// Apply S (phase) gate to target qubit.
    pub fn s(mut self, target: usize) -> Self {
        self.apply1q(&S_GATE, target);
        self
    }

    /// Apply controlled-NOT gate.
    pub fn cnot(mut self, control: usize, target: usize) -> Self {
        self.apply2q(&CNOT_GATE, control, target);
        self
    }

    /// Apply controlled-phase gate.
    pub fn cphase(mut self, control: usize, target: usize) -> Self {
        self.apply2q(&CPHASE_GATE, control, target);
        self
    }

    /// Measure qubit `i` `ntimes` times, collapsing the state each time.
    pub fn measure(&mut self, i: usize, ntimes: usize, rng: &mut impl Rng) -> Vec<usize> {
        assert!(i < self.n, "Invalid qubit {i}. Must be in [0, {})", self.n);

        let mut results = Vec::with_capacity(ntimes);
        for _ in 0..ntimes {
            // Calculate probability of measuring |0> on qubit i
            let prob0: f64 = self
                .v
                .iter()
                .enumerate()
                .filter(|(idx, _)| (idx >> i) & 1 == 0)
                .map(|(_, amp)| amp.norm_sqr())
                .sum();

            let outcome = if rng.r#gen::<f64>() < prob0 { 0 } else { 1 };
            results.push(outcome);

            // Collapse: zero out amplitudes inconsistent with outcome
            for idx in 0..self.v.len() {
                if (idx >> i) & 1 != outcome {
                    self.v[idx] = Complex64::new(0.0, 0.0);
                }
            }
            self.normalize();
        }
        results
    }
}

impl fmt::Display for QReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.terms())
    }
}

impl Add for QReg {
    type Output = QReg;
    fn add(self, other: QReg) -> QReg {
        let inv_sqrt2 = Complex64::new(std::f64::consts::FRAC_1_SQRT_2, 0.0);
        let v = (&self.v + &other.v).mapv(|x| x * inv_sqrt2);
        QReg::from_array(v)
    }
}

impl Sub for QReg {
    type Output = QReg;
    fn sub(self, other: QReg) -> QReg {
        let inv_sqrt2 = Complex64::new(std::f64::consts::FRAC_1_SQRT_2, 0.0);
        let v = (&self.v - &other.v).mapv(|x| x * inv_sqrt2);
        QReg::from_array(v)
    }
}

impl Mul for QReg {
    type Output = QReg;
    /// Tensor product of two quantum states.
    fn mul(self, other: QReg) -> QReg {
        QReg::from_array(kron(&self.v, &other.v))
    }
}

// ---- Convenience Functions ----

/// Create a quantum ket state from a string specification.
///
/// Characters: '0' = |0>, '1' = |1>, '+' = |+>, '-' = |->
///
/// Examples: "0", "1", "00", "01", "++", "+-", "101"
pub fn ket(vecstring: &str) -> QReg {
    assert!(!vecstring.is_empty(), "vecstring cannot be empty");

    let valid = ['0', '1', '+', '-'];
    for ch in vecstring.chars() {
        assert!(
            valid.contains(&ch),
            "Invalid character '{ch}' in vecstring. Valid: 0, 1, +, -"
        );
    }

    let qvec = |s: char| -> Array1<Complex64> {
        match s {
            '0' => Array1::from_vec(vec![ONE, ZERO]),
            '1' => Array1::from_vec(vec![ZERO, ONE]),
            '+' => Array1::from_vec(vec![S2, S2]),
            '-' => Array1::from_vec(vec![S2, NEG_S2]),
            _ => unreachable!(),
        }
    };

    let mut register = Array1::from_vec(vec![ONE]);
    for ch in vecstring.chars().rev() {
        register = kron(&qvec(ch), &register);
    }
    QReg::from_array(register)
}

// ---- Python Bindings ----

#[cfg(feature = "pyo3")]
mod python;

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    // -- Utility tests --

    #[test]
    fn test_nqubits() {
        assert_eq!(nqubits(2), 1);
        assert_eq!(nqubits(4), 2);
        assert_eq!(nqubits(16), 4);
    }

    #[test]
    fn test_conjugate_index() {
        assert_eq!(conjugate_index(0, 0), 1); // |0> -> |1>
        assert_eq!(conjugate_index(1, 0), 0); // |1> -> |0>
        assert_eq!(conjugate_index(2, 1), 0); // |10> -> |00>
    }

    // -- Ket construction tests --

    #[test]
    fn test_ket_single_qubits() {
        assert_eq!(ket("0").to_string(), "1.0|0>");
        assert_eq!(ket("1").to_string(), "1.0|1>");
    }

    #[test]
    fn test_ket_two_qubits() {
        assert_eq!(ket("00").to_string(), "1.0|00>");
        assert_eq!(ket("10").to_string(), "1.0|10>");
        assert_eq!(ket("11").to_string(), "1.0|11>");
    }

    #[test]
    fn test_ket_three_qubits() {
        assert_eq!(ket("101").to_string(), "1.0|101>");
    }

    // -- Single-qubit gate tests --

    #[test]
    fn test_x_gate() {
        assert_eq!(ket("0").x(0).to_string(), "1.0|1>");
        assert_eq!(ket("1").x(0).to_string(), "1.0|0>");
        assert_eq!(ket("00").x(0).to_string(), "1.0|01>");
        assert_eq!(ket("01").x(1).to_string(), "1.0|11>");
    }

    #[test]
    fn test_chained_gates() {
        assert_eq!(ket("00").x(0).x(1).to_string(), "1.0|11>");
        assert_eq!(ket("+").h(0).x(0).to_string(), "1.0|1>");
    }

    // -- Two-qubit gate tests --

    #[test]
    fn test_cnot_gate() {
        assert_eq!(ket("01").cnot(0, 1).to_string(), "1.0|11>");
        assert_eq!(ket("00").cnot(0, 1).to_string(), "1.0|00>");
    }

    // -- Isclose tests --

    #[test]
    fn test_isclose_plus_plus() {
        assert!(ket("++").isclose_slice(&[0.5, 0.5, 0.5, 0.5]));
    }

    #[test]
    fn test_isclose_minus_minus() {
        assert!(ket("--").isclose_slice(&[0.5, -0.5, -0.5, 0.5]));
    }

    // -- Operator tests --

    #[test]
    fn test_add_gives_plus() {
        assert!(ket("+").isclose(&(ket("0") + ket("1"))));
    }

    #[test]
    fn test_sub_gives_minus() {
        assert!(ket("-").isclose(&(ket("0") - ket("1"))));
    }

    #[test]
    fn test_tensor_product() {
        let q00 = ket("0") * ket("0");
        assert!(q00.isclose(&ket("00")));
    }

    // -- Measurement tests --

    #[test]
    fn test_measure_zero() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut q = ket("0");
        assert_eq!(q.measure(0, 1, &mut rng), vec![0]);
    }

    #[test]
    fn test_measure_one() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut q = ket("1");
        assert_eq!(q.measure(0, 1, &mut rng), vec![1]);
    }

    #[test]
    fn test_measure_collapse() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut q = ket("+");
        let r1 = q.measure(0, 1, &mut rng);
        // After collapse, subsequent measurements are deterministic
        let r2 = q.measure(0, 3, &mut rng);
        assert!(r2.iter().all(|&x| x == r1[0]));
    }

    // -- ket terms() direct call (Python: ket('10').terms() == '1.0|10>') --

    #[test]
    fn test_terms_direct() {
        assert_eq!(ket("10").terms(), "1.0|10>");
    }

    // -- ket('01') missing from construction tests --

    #[test]
    fn test_ket_01() {
        assert_eq!(ket("01").to_string(), "1.0|01>");
    }

    // -- Y gate tests --

    #[test]
    fn test_y_gate() {
        // Y|0> = i|1>
        let q = ket("0").y(0);
        assert!(q.v[0].norm() < 1e-8);
        assert!((q.v[1] - Complex64::new(0.0, 1.0)).norm() < 1e-5);

        // Y|1> = -i|0>
        let q = ket("1").y(0);
        assert!((q.v[0] - Complex64::new(0.0, -1.0)).norm() < 1e-5);
        assert!(q.v[1].norm() < 1e-8);
    }

    // -- Z gate tests --

    #[test]
    fn test_z_gate() {
        // Z|0> = |0>
        assert_eq!(ket("0").z(0).to_string(), "1.0|0>");
        // Z|1> = -|1>
        assert_eq!(ket("1").z(0).to_string(), "-1.0|1>");
    }

    // -- H gate tests --

    #[test]
    fn test_h_gate() {
        // H|0> = |+>
        assert!(ket("0").h(0).isclose(&ket("+")));
        // H|1> = |->
        assert!(ket("1").h(0).isclose(&ket("-")));
        // H|+> = |0>
        assert!(ket("+").h(0).isclose(&ket("0")));
        // H|-> = |1>
        assert!(ket("-").h(0).isclose(&ket("1")));
        // HH = I (self-inverse)
        assert!(ket("0").h(0).h(0).isclose(&ket("0")));
        assert!(ket("1").h(0).h(0).isclose(&ket("1")));
    }

    // -- S gate tests --

    #[test]
    fn test_s_gate() {
        // S|0> = |0>
        assert_eq!(ket("0").s(0).to_string(), "1.0|0>");
        // S|1> = i|1>
        let q = ket("1").s(0);
        assert!(q.v[0].norm() < 1e-8);
        assert!((q.v[1] - Complex64::new(0.0, 1.0)).norm() < 1e-5);
    }

    // -- CNOT on all 2-qubit basis states --

    #[test]
    fn test_cnot_all_basis() {
        // CNOT(control=0, target=1): flips target when control is 1
        assert_eq!(ket("00").cnot(0, 1).to_string(), "1.0|00>");
        assert_eq!(ket("01").cnot(0, 1).to_string(), "1.0|11>");
        assert_eq!(ket("10").cnot(0, 1).to_string(), "1.0|10>");
        assert_eq!(ket("11").cnot(0, 1).to_string(), "1.0|01>");
    }

    // -- CPHASE gate tests --

    #[test]
    fn test_cphase_gate() {
        // CPHASE only flips sign of |11>
        assert_eq!(ket("00").cphase(0, 1).to_string(), "1.0|00>");
        assert_eq!(ket("01").cphase(0, 1).to_string(), "1.0|01>");
        assert_eq!(ket("10").cphase(0, 1).to_string(), "1.0|10>");
        assert_eq!(ket("11").cphase(0, 1).to_string(), "-1.0|11>");
    }

    // -- Tensor product tests --

    #[test]
    fn test_tensor_products() {
        assert!(ket("01").isclose(&(ket("0") * ket("1"))));
        assert!(ket("10").isclose(&(ket("1") * ket("0"))));
        assert!(ket("11").isclose(&(ket("1") * ket("1"))));
    }

    // -- Normalization test --

    #[test]
    fn test_normalization() {
        // QReg normalizes on construction
        let q = QReg::new(vec![
            Complex64::new(3.0, 0.0),
            Complex64::new(4.0, 0.0),
        ]);
        assert!((q.norm() - 1.0).abs() < 1e-10);
    }

    // -- Measurement: repeated after collapse (Python: s.M(0,3) == [0,0,0]) --

    #[test]
    fn test_measure_repeated_after_collapse() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut q = ket("+");
        let first = q.measure(0, 1, &mut rng);
        // After collapse, all further measurements give the same result
        let repeated = q.measure(0, 5, &mut rng);
        assert_eq!(repeated, vec![first[0]; 5]);
    }

    // -- XX = I (Pauli involutions) --

    #[test]
    fn test_pauli_involutions() {
        assert!(ket("0").x(0).x(0).isclose(&ket("0")));
        assert!(ket("0").y(0).y(0).isclose(&ket("0")));
        assert!(ket("0").z(0).z(0).isclose(&ket("0")));
        assert!(ket("1").x(0).x(0).isclose(&ket("1")));
    }

    // -- Bell state test --

    #[test]
    fn test_bell_state() {
        let bell = ket("00").h(0).cnot(0, 1);
        // Bell state |00> + |11> with equal amplitudes
        assert!(bell.isclose_slice(&[
            std::f64::consts::FRAC_1_SQRT_2,
            0.0,
            0.0,
            std::f64::consts::FRAC_1_SQRT_2
        ]));
    }

    // -- GHZ state (3-qubit entanglement) --

    #[test]
    fn test_ghz_state() {
        let ghz = ket("000").h(0).cnot(0, 1).cnot(1, 2);
        // GHZ = (|000> + |111>) / sqrt(2)
        assert!(ghz.isclose_slice(&[
            std::f64::consts::FRAC_1_SQRT_2,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            std::f64::consts::FRAC_1_SQRT_2
        ]));
    }
}
