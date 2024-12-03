mod game2048;

use game2048::{Game2048Circuit, F, D};
use plonky2::field::types::Field;
use plonky2::iop::witness::PartialWitness;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2::plonk::circuit_data::CircuitConfig;

type C = PoseidonGoldilocksConfig;

fn main() {
    // Example board states as inputs (using F type)
    let before_board: Vec<F> = vec![
        F::from_canonical_u32(2), F::from_canonical_u32(2), F::ZERO,                    F::from_canonical_u32(4),
        F::ZERO,                  F::ZERO,                  F::from_canonical_u32(4),   F::ZERO,
        F::from_canonical_u32(2), F::ZERO,                  F::ZERO,                    F::from_canonical_u32(2),
        F::ZERO,                  F::from_canonical_u32(4), F::ZERO,                    F::ZERO,
    ];
    
    let after_board: Vec<F> = vec![
        F::from_canonical_u32(4),   F::from_canonical_u32(2),   F::from_canonical_u32(4),   F::from_canonical_u32(4),
        F::ZERO,                    F::from_canonical_u32(4),   F::ZERO,                    F::from_canonical_u32(2),
        F::ZERO,                    F::ZERO,                    F::ZERO,                    F::ZERO,
        F::ZERO,                    F::ZERO,                    F::ZERO,                    F::ZERO,
    ];

    let direction = "up";

    // Initialize the circuit builder
    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);

    // Add constraints for the game logic directly using F values
    Game2048Circuit::build_circuit(&mut builder, &before_board, &after_board, direction);

    // Build the circuit
    let circuit = builder.build::<C>();

    // Create a partial witness (empty in this case, as we're directly using F)
    let pw = PartialWitness::<F>::new();

    // Generate the proof
    let proof = circuit.prove(pw);

    // Verify the proof
    let verified = circuit.verify(proof.unwrap()).is_ok();
    println!("Proof verified: {}", verified);
}