use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::iop::target::{BoolTarget, Target};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;

pub const D: usize = 2;
pub type F = GoldilocksField;

pub struct Game2048Circuit;

impl Game2048Circuit {
    /// Build the circuit for validating a 2048 game move
    pub fn build_circuit() -> (CircuitBuilder<F, D>, Vec<Target>) {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(config);

        // Create targets for before_board, after_board, and direction
        let before_board_targets: Vec<_> = (0..16).map(|_| builder.add_virtual_target()).collect();
        let after_board_targets: Vec<_> = (0..16).map(|_| builder.add_virtual_target()).collect();
        let direction_target = builder.add_virtual_target();

        // Register public inputs for before_board, after_board, and direction
        for &target in &before_board_targets {
            builder.register_public_input(target);
        }
        for &target in &after_board_targets {
            builder.register_public_input(target);
        }
        builder.register_public_input(direction_target);

        // Add constraints for each move direction
        Self::add_constraints(
            &mut builder,
            &before_board_targets,
            &after_board_targets,
            direction_target,
        );

        // Combine all targets into a single vector
        let mut targets = before_board_targets;
        targets.extend(after_board_targets);
        targets.push(direction_target);

        (builder, targets)
    }

    /// Add constraints dynamically based on the move direction
    fn add_constraints(
        builder: &mut CircuitBuilder<F, D>,
        before_board: &[Target],
        after_board: &[Target],
        direction_target: Target,
    ) {
        // Constants for move directions
        let up_const = builder.constant(F::from_canonical_u32(0));
        let down_const = builder.constant(F::from_canonical_u32(1));
        let left_const = builder.constant(F::from_canonical_u32(2));
        let right_const = builder.constant(F::from_canonical_u32(3));

        // Boolean flags for each direction
        let is_up = builder.is_equal(direction_target, up_const);
        let is_down = builder.is_equal(direction_target, down_const);
        let is_left = builder.is_equal(direction_target, left_const);
        let is_right = builder.is_equal(direction_target, right_const);

        // Add constraints for each direction conditionally
        Self::add_constraints_up(builder, before_board, after_board, is_up);
        Self::add_constraints_down(builder, before_board, after_board, is_down);
        Self::add_constraints_left(builder, before_board, after_board, is_left);
        Self::add_constraints_right(builder, before_board, after_board, is_right);
    }

    /// Add constraints for "up" direction
    fn add_constraints_up(
        builder: &mut CircuitBuilder<F, D>,
        before_board: &[Target],
        after_board: &[Target],
        condition: BoolTarget,
    ) {
        for col in 0..4 {
            let before_col: Vec<_> = (0..4).map(|row| before_board[row * 4 + col]).collect();
            let after_col: Vec<_> = (0..4).map(|row| after_board[row * 4 + col]).collect();
            Self::validate_tiles(builder, &before_col, &after_col, condition);
        }
    }

    /// Add constraints for "down" direction
    fn add_constraints_down(
        builder: &mut CircuitBuilder<F, D>,
        before_board: &[Target],
        after_board: &[Target],
        condition: BoolTarget,
    ) {
        for col in 0..4 {
            let before_col: Vec<_> = (0..4).map(|row| before_board[row * 4 + col]).collect();
            let mut reversed_before_col = before_col.clone();
            reversed_before_col.reverse();

            let after_col: Vec<_> = (0..4).map(|row| after_board[row * 4 + col]).collect();
            let mut reversed_after_col = after_col.clone();
            reversed_after_col.reverse();

            Self::validate_tiles(builder, &reversed_before_col, &reversed_after_col, condition);
        }
    }

    /// Add constraints for "left" direction
    fn add_constraints_left(
        builder: &mut CircuitBuilder<F, D>,
        before_board: &[Target],
        after_board: &[Target],
        condition: BoolTarget,
    ) {
        for row in 0..4 {
            let before_row: Vec<_> = (0..4).map(|col| before_board[row * 4 + col]).collect();
            let after_row: Vec<_> = (0..4).map(|col| after_board[row * 4 + col]).collect();
            Self::validate_tiles(builder, &before_row, &after_row, condition);
        }
    }

    /// Add constraints for "right" direction
    fn add_constraints_right(
        builder: &mut CircuitBuilder<F, D>,
        before_board: &[Target],
        after_board: &[Target],
        condition: BoolTarget,
    ) {
        for row in 0..4 {
            let before_row: Vec<_> = (0..4).map(|col| before_board[row * 4 + col]).collect();
            let mut reversed_before_row = before_row.clone();
            reversed_before_row.reverse();

            let after_row: Vec<_> = (0..4).map(|col| after_board[row * 4 + col]).collect();
            let mut reversed_after_row = after_row.clone();
            reversed_after_row.reverse();

            Self::validate_tiles(builder, &reversed_before_row, &reversed_after_row, condition);
        }
    }

    /// Validate the tiles for a single row or column
    fn validate_tiles(
        builder: &mut CircuitBuilder<F, D>,
        before_tiles: &[Target],
        after_tiles: &[Target],
        condition: BoolTarget,
    ) {
        let mut merged = Vec::new();
        let mut last = None;

        // Use a constant target for zero
        let zero_target = builder.constant(F::ZERO);

        for &tile in before_tiles {
            let is_zero = builder.is_equal(tile, zero_target); // Check if the tile is zero
            let is_non_zero = builder.not(is_zero).target;

            if let Some(last_tile) = last {
                let are_equal = builder.is_equal(last_tile, tile);
                let merged_tile = builder.add(last_tile, tile);
                let should_merge = builder.and(condition, are_equal);
                merged.push(builder.mul(should_merge.target, merged_tile));

                let not_merged = builder.not(are_equal).target;
                last = Some(builder.mul(not_merged, tile));
            } else {
                last = Some(builder.mul(is_non_zero, tile));
            }
        }

        if let Some(last_tile) = last {
            merged.push(last_tile);
        }    

        while merged.len() < 4 {
            merged.push(zero_target);
        }

        for (merged_tile, &after_tile) in merged.iter().zip(after_tiles) {
            builder.connect(*merged_tile, after_tile);
        }
    }
}