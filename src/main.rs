use rvecsim::ket;
use std::time::Instant;

fn main() {
    println!("rvecsim - Quantum Vector State Simulator\n");

    // Demo
    println!("|0>         = {}", ket("0"));
    println!("|0>.X(0)    = {}", ket("0").x(0));
    println!("|0>.H(0)    = {}", ket("0").h(0));
    println!("|+>         = {}", ket("+"));
    let bell = ket("00").h(0).cnot(0, 1);
    println!("Bell state  = {}", bell);

    // Benchmark
    println!("\n--- Benchmark ---\n");

    for n in [10, 15, 18, 20, 22] {
        let zeros: String = "0".repeat(n);
        let t0 = Instant::now();
        let q = ket(&zeros);
        let setup = t0.elapsed();

        // Apply H to all qubits, then CNOT chain -> GHZ state
        let t0 = Instant::now();
        let mut q = q.h(0);
        for i in 0..n - 1 {
            q = q.cnot(i, i + 1);
        }
        let gates = t0.elapsed();

        let total_amps = 1usize << n;
        println!(
            "{:2} qubits ({:>8} amps): setup {:>8.2?}, gates {:>8.2?} (H + {} CNOTs)",
            n,
            total_amps,
            setup,
            gates,
            n - 1,
        );
    }
}
