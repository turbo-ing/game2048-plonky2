use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::iop::witness::PartialWitness;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2::plonk::circuit_data::CircuitConfig;

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = GoldilocksField;

/// Build a circuit for validating a 2048 game move
pub struct Game2048Circuit;

impl Game2048Circuit {
    /// Creates a circuit for validating the 2048 game logic
    pub fn build_circuit(
        builder: &mut CircuitBuilder<F, D>,
        before_board: &[F],
        after_board: &[F],
        direction: &str,
    ) {
        match direction {
            "up" => Self::add_constraints_up(builder, before_board, after_board),
            "down" => Self::add_constraints_down(builder, before_board, after_board),
            "left" => Self::add_constraints_left(builder, before_board, after_board),
            "right" => Self::add_constraints_right(builder, before_board, after_board),
            _ => panic!("Invalid move direction"),
        }
    }

    /// Add constraints for the "up" move
    fn add_constraints_up(builder: &mut CircuitBuilder<F, D>, before: &[F], after: &[F]) {
        for col in 0..4 {
            let before_col: Vec<F> = (0..4).map(|row| before[row * 4 + col]).collect();
            let after_col: Vec<F> = (0..4).map(|row| after[row * 4 + col]).collect();
            Self::add_tile_constraints(builder, &before_col, &after_col);
        }
    }

    /// Add constraints for the "down" move
    fn add_constraints_down(builder: &mut CircuitBuilder<F, D>, before: &[F], after: &[F]) {
        for col in 0..4 {
            let before_col: Vec<F> = (0..4).map(|row| before[row * 4 + col]).collect();
            let after_col: Vec<F> = (0..4).map(|row| after[row * 4 + col]).collect();
            Self::add_tile_constraints(builder, &before_col.into_iter().rev().collect::<Vec<_>>(), &after_col.into_iter().rev().collect::<Vec<_>>());
        }
    }

    /// Add constraints for the "left" move
    fn add_constraints_left(builder: &mut CircuitBuilder<F, D>, before: &[F], after: &[F]) {
        for row in 0..4 {
            let before_row: Vec<F> = (0..4).map(|col| before[row * 4 + col]).collect();
            let after_row: Vec<F> = (0..4).map(|col| after[row * 4 + col]).collect();
            Self::add_tile_constraints(builder, &before_row, &after_row);
        }
    }

    /// Add constraints for the "right" move
    fn add_constraints_right(builder: &mut CircuitBuilder<F, D>, before: &[F], after: &[F]) {
        for row in 0..4 {
            let before_row: Vec<F> = (0..4).map(|col| before[row * 4 + col]).collect();
            let after_row: Vec<F> = (0..4).map(|col| after[row * 4 + col]).collect();
            Self::add_tile_constraints(builder, &before_row.into_iter().rev().collect::<Vec<_>>(), &after_row.into_iter().rev().collect::<Vec<_>>());
        }
    }

    /// Add constraints for tile movement and merging
    fn add_tile_constraints(builder: &mut CircuitBuilder<F, D>, before_tiles: &[F], after_tiles: &[F]) {
        let mut merged: Vec<F> = Vec::new();
        let mut last: Option<F> = None;

        for &tile in before_tiles {
            if tile == F::ZERO {
                // Skip zero tiles
                continue;
            }

            if let Some(last_tile) = last {
                if last_tile == tile {
                    // Merge tiles if they are equal
                    merged.push(last_tile + tile);
                    last = None; // Clear the last tile after merging
                } else {
                    // Push the last tile to the result and set the current tile as last
                    merged.push(last_tile);
                    last = Some(tile);
                }
            } else {
                // No last tile, set the current tile as last
                last = Some(tile);
            }
        }

        // Add the last unmerged tile if any
        if let Some(last_tile) = last {
            merged.push(last_tile);
        }

        // Pad the merged vector with zeros to ensure it has exactly 4 elements
        while merged.len() < 4 {
            merged.push(F::ZERO);
        }

        // Connect the merged result to the after_tiles
        for (merged_tile, &after_tile) in merged.iter().zip(after_tiles) {
            let merged_target = builder.constant(*merged_tile);
            let after_target = builder.constant(after_tile);
            builder.connect(merged_target, after_target);
        }

        // for (i, (merged_tile, &after_tile)) in merged.iter().zip(after_tiles).enumerate() {
        //     let merged_target = builder.constant(*merged_tile);
        //     let after_target = builder.constant(after_tile);
        //     println!(
        //         "Row {}, Expected: {}, Got: {}",
        //         i,
        //         after_target,
        //         merged_target
        //     );
        //     builder.connect(merged_target, after_target);
        // }
    }
}

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
    let mut builder = CircuitBuilder::<F, 2>::new(config);

    // Add constraints for the game logic directly using F values
    Game2048Circuit::build_circuit(&mut builder, &before_board, &after_board, direction);

    // Build the circuit
    let circuit = builder.build::<C>();

    // Create a partial witness (empty in this case, as we're directly using F)
    let pw = PartialWitness::<F>::new();

    // Generate the proof
    let proof = circuit.prove(pw).expect("Proof generation failed");

    // Verify the proof
    let verified = circuit.verify(proof).is_ok();
    println!("Proof verified: {}", verified);
}