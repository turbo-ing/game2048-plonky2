use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::plonk::circuit_builder::CircuitBuilder;

pub const D: usize = 2;
pub type F = GoldilocksField;

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
    }
}