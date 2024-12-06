mod game2048;

use game2048::{Game2048Circuit, F};
use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::config::PoseidonGoldilocksConfig;

fn main() {
    // Input boards and direction
    let before_board: Vec<F> = vec![
        F::from_canonical_u32(2), F::from_canonical_u32(2), F::from_canonical_u32(4), F::from_canonical_u32(8),
        F::from_canonical_u32(2), F::from_canonical_u32(0), F::from_canonical_u32(4), F::from_canonical_u32(4),
        F::from_canonical_u32(2), F::from_canonical_u32(2), F::from_canonical_u32(2), F::from_canonical_u32(4),
        F::from_canonical_u32(0), F::from_canonical_u32(2), F::from_canonical_u32(4), F::from_canonical_u32(4),
    ];

    let after_board: Vec<F> = vec![
        F::from_canonical_u32(0), F::from_canonical_u32(4), F::from_canonical_u32(4), F::from_canonical_u32(8),
        F::from_canonical_u32(0), F::from_canonical_u32(0), F::from_canonical_u32(2), F::from_canonical_u32(8),
        F::from_canonical_u32(0), F::from_canonical_u32(2), F::from_canonical_u32(4), F::from_canonical_u32(4),
        F::from_canonical_u32(0), F::from_canonical_u32(0), F::from_canonical_u32(2), F::from_canonical_u32(8),
    ];

    // Direction: "right"
    let direction = F::from_canonical_u32(3); // 0 for "right", as defined in the circuit

    // Build the circuit
    let (circuit_builder, targets) = Game2048Circuit::build_circuit();

    // Unpack targets
    let (before_targets, after_targets, direction_target) =
        (&targets[0..16], &targets[16..32], targets[32]);

    // Set the witness values
    let mut pw = PartialWitness::<F>::new();
    for (i, &target) in before_targets.iter().enumerate() {
        pw.set_target(target, before_board[i]).unwrap();
    }
    for (i, &target) in after_targets.iter().enumerate() {
        pw.set_target(target, after_board[i]).unwrap();
    }
    pw.set_target(direction_target, direction).unwrap();

    // Build and generate the proof
    let circuit = circuit_builder.build::<PoseidonGoldilocksConfig>();
    let proof = circuit.prove(pw).unwrap();

    // Verify the proof
    let verified = circuit.verify(proof).is_ok();
    println!("Proof verified: {}", verified);
}