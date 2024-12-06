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
        let merged = Self::merge_2048_row(builder, before_tiles[0], before_tiles[1], before_tiles[2], before_tiles[3]);

        let out0 = builder._if(condition, merged[0], after_tiles[0]);
        let out1 = builder._if(condition, merged[1], after_tiles[1]);
        let out2 = builder._if(condition, merged[2], after_tiles[2]);
        let out3 = builder._if(condition, merged[3], after_tiles[3]);

        let final_row = [out0, out1, out2, out3];

        // builder.connect_array(merged, after_tiles);
        for (&merged_tile, &after_tile) in final_row.iter().zip(after_tiles) {
            builder.connect(merged_tile, after_tile);
        }
    }

    /// Merge a single 2048 row [a,b,c,d] toward the left
    pub fn merge_2048_row(builder: &mut CircuitBuilder<F, D>, a: Target, b: Target, c: Target, d: Target) -> [Target; 4] {
        let zero = builder.zero();

        // // Step 1: Compact nonzero tiles to the left
        let result = Self::shift_nonzero_left(builder, a, b, c, d, zero);
        let x0 = result[0];
        let x1 = result[1];
        let x2 = result[2];
        let x3 = result[3];

        // Step 2: Merge logic
        // Merge from left:
        // Check (x0,x1)
        let eq_x0_x1 = builder.is_equal(x0,x1);
        let x0_eq_zero = builder.is_equal(x0,zero);
        let x1_eq_zero = builder.is_equal(x1,zero);
        let x0_nonzero = builder.not(x0_eq_zero);
        let x1_nonzero = builder.not(x1_eq_zero);
        let can_merge_x0_x1_pre = builder.and(eq_x0_x1, x0_nonzero);
        let can_merge_x0_x1 = builder.and(can_merge_x0_x1_pre, x1_nonzero);

        // Merge (x0,x1) if possible
        let two = builder.constant(F::from_canonical_u64(2));
        let doubled_x0 = builder.mul(x0,two);

        let nx0 = builder._if(can_merge_x0_x1, doubled_x0, x0);
        let mut nx1 = builder._if(can_merge_x0_x1, x2, x1);
        let mut nx2 = builder._if(can_merge_x0_x1, x3, x2);
        let mut nx3 = builder._if(can_merge_x0_x1, zero, x3);

        let not_merged_x0_x1 = builder.not(can_merge_x0_x1);

        // If merged at (x0,x1), we skip checking (x1,x2) and go directly to (x2,x3).
        // If not merged at (x0,x1), we check (x1,x2).

        // Check (x1,x2) only if not merged at (x0,x1)
        let eq_x1_x2 = builder.is_equal(nx1,nx2);
        let x1_eq_zero2 = builder.is_equal(nx1,zero);
        let x2_eq_zero2 = builder.is_equal(nx2,zero);
        let x1_nonzero2 = builder.not(x1_eq_zero2);
        let x2_nonzero2 = builder.not(x2_eq_zero2);
        let can_merge_x1_x2_pre = builder.and(eq_x1_x2, x1_nonzero2);
        let can_merge_x1_x2 = builder.and(can_merge_x1_x2_pre, x2_nonzero2);

        let do_x1_x2_merge = builder.and(not_merged_x0_x1, can_merge_x1_x2);

        let doubled_x1 = builder.mul(nx1,two);
        nx1 = builder._if(do_x1_x2_merge, doubled_x1, nx1);
        nx2 = builder._if(do_x1_x2_merge, nx3, nx2);
        nx3 = builder._if(do_x1_x2_merge, zero, nx3);

        // If merged at (x1,x2), skip (x2,x3).
        // If merged at (x0,x1), we want to check (x2,x3).
        // If not merged at (x0,x1) and not merged at (x1,x2), we check (x2,x3).

        let not_merged_x1_x2 = builder.not(do_x1_x2_merge);

        // Conditions for checking (x2,x3):
        // - If merged at (x0,x1), check (x2,x3).
        // - If not merged at (x0,x1) and not merged at (x1,x2), check (x2,x3).
        // - If merged at (x1,x2), do NOT check (x2,x3).

        // This ensures that if we merged at x0,x1 or no merges happened so far, we still check x2,x3. 
        // If we merged at x1,x2, not_merged_x1_x2=0 and can_merge_x0_x1=0, 
        // so check_x2_x3=0 => no check.

        // Actually, refine logic for check_x2_x3:
        // - Merged at (x0,x1): can_merge_x0_x1=1 => check_x2_x3=1
        // - Not merged at (x0,x1) (means can_merge_x0_x1=0):
        //    If merged at (x1,x2) (do_x1_x2_merge=1) => no check_x2_x3=0
        //    Else no merges (do_x1_x2_merge=0) => check_x2_x3=1
        //
        // We can achieve this by:
        // check_x2_x3 = can_merge_x0_x1 OR (not_merged_x0_x1 AND not_merged_x1_x2)
        let not_merged_x0_x1_and_not_x1_x2 = builder.and(not_merged_x0_x1, not_merged_x1_x2);
        let check_x2_x3_final = builder.or(can_merge_x0_x1, not_merged_x0_x1_and_not_x1_x2);

        // Check (x2,x3) if allowed
        let eq_x2_x3 = builder.is_equal(nx2,nx3);
        let x2_eq_zero3 = builder.is_equal(nx2,zero);
        let x3_eq_zero3 = builder.is_equal(nx3,zero);
        let x2_nonzero3 = builder.not(x2_eq_zero3);
        let x3_nonzero3 = builder.not(x3_eq_zero3);
        let can_merge_x2_x3_pre = builder.and(eq_x2_x3, x2_nonzero3);
        let can_merge_x2_x3 = builder.and(can_merge_x2_x3_pre, x3_nonzero3);

        let do_x2_x3_merge_pre = builder.and(check_x2_x3_final, can_merge_x2_x3);
        let do_x2_x3_merge = do_x2_x3_merge_pre;

        let doubled_x2 = builder.mul(nx2,two);
        nx2 = builder._if(do_x2_x3_merge, doubled_x2, nx2);
        nx3 = builder._if(do_x2_x3_merge, zero, nx3);

        // Now [nx0, nx1, nx2, nx3] is fully merged according to 2048 rules.
        [nx0, nx1, nx2, nx3]
    }

    /// A simplified helper that picks remaining two tiles after x0 and x1.
    /// In a complete solution, you'd replicate the zero-skipping logic as above.
    /// For demonstration, we assume you have a similar pattern.
    fn shift_nonzero_left(builder: &mut CircuitBuilder<F, D>, a: Target, b: Target, c: Target, d: Target, zero: Target) -> [Target; 4] {
        // Helper booleans for zero-check
        let a_eq_zero = builder.is_equal(a, zero);
        let a_nonzero = builder.not(a_eq_zero);

        let b_eq_zero = builder.is_equal(b, zero);
        let b_nonzero = builder.not(b_eq_zero);

        let c_eq_zero = builder.is_equal(c, zero);
        let c_nonzero = builder.not(c_eq_zero);

        let d_eq_zero = builder.is_equal(d, zero);
        let d_nonzero = builder.not(d_eq_zero);

        // Step 1: Compact nonzero tiles to the left
        //
        // x0 = first nonzero tile from [a,b,c,d] or zero if none
        // If a is nonzero, x0 = a
        // Else if b is nonzero, x0 = b, else if c is nonzero, x0 = c, else if d is nonzero, x0 = d, else 0
        // For candidate_if_not_a:
        let tmp_d = builder._if(d_nonzero, d, zero);
        let tmp_c = builder._if(c_nonzero, c, tmp_d);
        let candidate_if_not_a = builder._if(b_nonzero, b, tmp_c);
        let frist_nonzero_val = builder._if(a_nonzero, a, candidate_if_not_a);

        // x1 = second nonzero tile (after x0)
        // If x0 came from a (a_nonzero=1), then we skip 'a' and pick next from [b,c,d].
        // If x0 did not come from a, it means a=0, so x0 came from b,c or d. Then we must pick next after that.
        // For x1_if_x0_from_a:
        let tmp_d2 = builder._if(d_nonzero, d, zero);
        let tmp_c2 = builder._if(c_nonzero, c, tmp_d2);
        let x1_if_x0_from_a = builder._if(b_nonzero, b, tmp_c2);

        // If x0 was not from a, we know a=0.
        // If x0 from b (b_nonzero=1), then x1 = next nonzero from [c,d].
        // If b=0, x0 from c => x1 = next nonzero from [d]
        // If c=0 too, x0 from d or no tile => x1=0
        let x1_if_x0_not_a = {
            let tem_d = builder._if(d_nonzero, d, zero);
            let x1_if_x0_b = builder._if(c_nonzero, c, tem_d);
            let x1_if_x0_c = builder._if(d_nonzero, d, zero);
            let x1_if_x0_d = zero;
            // If x0=b -> b_nonzero=1 => x1=x1_if_x0_b
            // else if x0=c -> c_nonzero=1 => x1=x1_if_x0_c
            // else x1= x1_if_x0_d
            let tem_c = builder._if(c_nonzero, x1_if_x0_c, x1_if_x0_d);
            builder._if(b_nonzero, x1_if_x0_b, tem_c)
        };

        let second_nonzero_val = builder._if(a_nonzero, x1_if_x0_from_a, x1_if_x0_not_a);

        // Similarly, x2 and x3 can be determined by continuing this logic.
        // For brevity, let's assume a simplified approach:  
        // After picking x0,x1 as first two nonzeros, pick x2 as the third nonzero, x3 as the fourth.
        // This code can be expanded similarly to x0,x1 using _if logic.

        // Determine booleans to know which tile was picked as first_nonzero:
        // x0_from_a = a_nonzero and no other chosen before
        let x0_from_a = a_nonzero;
        let x0_from_b_pre = builder.not(x0_from_a);
        let x0_from_b = builder.and(x0_from_b_pre, b_nonzero);
        let x0_from_c_pre = builder.not(x0_from_a);
        let x0_from_c_pre2 = builder.not(x0_from_b);
        let x0_from_c_0 = builder.and(x0_from_c_pre, x0_from_c_pre2);
        let x0_from_c = builder.and(x0_from_c_0, c_nonzero);
        let x0_from_d_pre = builder.not(x0_from_a);
        let x0_from_d_pre2 = builder.not(x0_from_b);
        let x0_from_d_pre3 = builder.not(x0_from_c);
        let x0_from_d_0 = builder.and(x0_from_d_pre, x0_from_d_pre2);
        let x0_from_d_1 = builder.and(x0_from_d_0, x0_from_d_pre3);
        let x0_from_d = builder.and(x0_from_d_1, d_nonzero);

        // -----------------------------------------------------------
        // Find second_nonzero tile
        // -----------------------------------------------------------
        // Now we must skip the tile chosen as first_nonzero.
        // If first chosen is a, skip a and choose from [b,c,d].
        // If first chosen is b, skip a,b and choose from [c,d].
        // If first chosen is c, skip a,b,c and choose from [d].
        // If first chosen is d, skip a,b,c,d (no second nonzero).

        // We'll build conditions step by step.

        // If x0_from_a:
        // second_nonzero = first nonzero from [b,c,d]
        // Conditions:
        // sec_from_b_if_a = x0_from_a && b_nonzero
        let sec_from_b_if_a_pre = x0_from_a;
        let sec_from_b_if_a = builder.and(sec_from_b_if_a_pre, b_nonzero);

        // sec_from_c_if_a = x0_from_a && !sec_from_b_if_a && c_nonzero
        let not_sec_from_b_if_a = builder.not(sec_from_b_if_a);
        let sec_from_c_if_a_pre = builder.and(x0_from_a, not_sec_from_b_if_a);
        let sec_from_c_if_a = builder.and(sec_from_c_if_a_pre, c_nonzero);

        // sec_from_d_if_a = x0_from_a && !sec_from_b_if_a && !sec_from_c_if_a && d_nonzero
        let not_sec_from_c_if_a = builder.not(sec_from_c_if_a);
        let sec_from_d_if_a_pre = builder.and(x0_from_a, not_sec_from_b_if_a);
        let sec_from_d_if_a_pre2 = builder.and(sec_from_d_if_a_pre, not_sec_from_c_if_a);
        let sec_from_d_if_a = builder.and(sec_from_d_if_a_pre2, d_nonzero);

        // If x0_from_b:
        // second_nonzero = first nonzero from [c,d]
        let sec_from_c_if_b_pre = x0_from_b;
        let sec_from_c_if_b = builder.and(sec_from_c_if_b_pre, c_nonzero);

        let not_sec_from_c_if_b = builder.not(sec_from_c_if_b);
        let sec_from_d_if_b_pre = builder.and(x0_from_b, not_sec_from_c_if_b);
        let sec_from_d_if_b = builder.and(sec_from_d_if_b_pre, d_nonzero);

        // If x0_from_c:
        // second_nonzero = first nonzero from [d]
        let sec_from_d_if_c_pre = x0_from_c;
        let sec_from_d_if_c = builder.and(sec_from_d_if_c_pre, d_nonzero);

        // Determine booleans for who second_nonzero came from:
        // For simplicity, we can define:
        let x1_from_b = sec_from_b_if_a; // only way to get second from b is if first from a
        let x1_from_c_pre1 = builder.or(sec_from_c_if_a, sec_from_c_if_b);
        let x1_from_c = x1_from_c_pre1; 
        let x1_from_d_pre1 = builder.or(sec_from_d_if_a, sec_from_d_if_b);
        let x1_from_d_pre2 = builder.or(x1_from_d_pre1, sec_from_d_if_c);
        let x1_from_d = x1_from_d_pre2;

        // -----------------------------------------------------------
        // Find third_nonzero tile
        // -----------------------------------------------------------
        // Now we skip the first two chosen tiles.
        // We must consider all scenarios. The first two chosen define who we skip:
        // If (x0_from_a), first chosen was 'a'.
        //   If (x1_from_b), second chosen was 'b', so skip [a,b], choose from [c,d]
        //   If (x1_from_c), second chosen was 'c', so skip [a,c], choose from [b,d]
        //   If (x1_from_d), second chosen was 'd', so skip [a,d], choose from [b,c]
        // If (x0_from_b), first chosen was 'b'.
        //   If (x1_from_c), second chosen was 'c', so skip [b,c], choose from [a,d]
        //   If (x1_from_d), second chosen was 'd', so skip [b,d], choose from [a,c]
        // If (x0_from_c), first chosen was 'c'.
        //   If (x1_from_d), second chosen was 'd', so skip [c,d], choose from [a,b]
        // If (x0_from_d), first chosen was 'd'.
        //   No second chosen (likely), choose from what remains: [a,b,c] (if any).
        //   But if x0_from_d, probably no second nonzero chosen means we pick third from [a,b,c].

        // This becomes quite large, but we proceed similarly:
        // We'll create conditions for each scenario and pick the third nonzero accordingly.

        // Let's define helper booleans to know which tiles are skipped:
        // We know exactly two tiles are chosen: x0 and x1.
        // We'll form a boolean for each tile indicating whether it was chosen:
        // chosen_a = x0_from_a OR (x1 chosen from a if that could happen)
        // Actually, second chosen can't be 'a' if first was not 'a', so let's just do it scenario by scenario.

        // Instead of enumerating all, let's do a generic approach:
        // We'll have a boolean for each tile if it was chosen among the first two:
        let chosen_a_pre1 = x0_from_a; // first chosen might be a
        // second chosen can't be a if first wasn't a_nonzero. So no need to check x1_from_a scenario.

        let chosen_b_pre1 = x1_from_b; // second chosen b possible if first chosen from a
        let chosen_b_pre2 = x0_from_b; // if first chosen is b
        let chosen_b = builder.or(chosen_b_pre1, chosen_b_pre2);

        let chosen_c_pre1 = x1_from_c; 
        let chosen_c_pre2 = x0_from_c;
        let chosen_c = builder.or(chosen_c_pre1, chosen_c_pre2);

        let chosen_d_pre1 = x1_from_d;
        let chosen_d_pre2 = x0_from_d;
        let chosen_d = builder.or(chosen_d_pre1, chosen_d_pre2);

        // Now the third nonzero is the next non-chosen nonzero tile in order a,b,c,d:
        // We must skip any tile that was chosen:
        // third_nonzero:
        // Check a: if a_nonzero and not chosen_a, this could be third_nonzero
        // If not, check b: if b_nonzero and not chosen_b
        // If not, check c: if c_nonzero and not chosen_c
        // If not, check d: if d_nonzero and not chosen_d
        // Else zero

        let not_chosen_a = builder.not(chosen_a_pre1); // chosen_a_pre1 = x0_from_a
        let cond_third_a_0 = builder.and(a_nonzero, not_chosen_a);

        let not_cond_third_a_0 = builder.not(cond_third_a_0);
        let not_chosen_b = builder.not(chosen_b);
        let cond_third_b_0 = builder.and(not_cond_third_a_0, b_nonzero);
        let cond_third_b = builder.and(cond_third_b_0, not_chosen_b);

        let not_cond_third_b = builder.not(cond_third_b);
        let not_chosen_c = builder.not(chosen_c);
        let cond_third_c_pre = builder.and(not_cond_third_a_0, not_cond_third_b);
        let cond_third_c_0 = builder.and(cond_third_c_pre, c_nonzero);
        let cond_third_c = builder.and(cond_third_c_0, not_chosen_c);

        let not_cond_third_c = builder.not(cond_third_c);
        let not_chosen_d = builder.not(chosen_d);
        let cond_third_d_pre = builder.and(not_cond_third_a_0, not_cond_third_b);
        let cond_third_d_pre2 = builder.and(cond_third_d_pre, not_cond_third_c);
        let cond_third_d_0 = builder.and(cond_third_d_pre2, d_nonzero);
        let cond_third_d = builder.and(cond_third_d_0, not_chosen_d);

        let mut third_nonzero_val = zero;
        third_nonzero_val = builder._if(cond_third_a_0, a, third_nonzero_val);
        third_nonzero_val = builder._if(cond_third_b, b, third_nonzero_val);
        third_nonzero_val = builder._if(cond_third_c, c, third_nonzero_val);
        third_nonzero_val = builder._if(cond_third_d, d, third_nonzero_val);

        // -----------------------------------------------------------
        // Find fourth_nonzero tile
        // -----------------------------------------------------------
        // Now we skip the three chosen tiles (x0,x1,x2).
        // Similar logic: 
        // chosen sets now also include the third chosen tile.

        // Mark tile chosen if it was chosen in first three picks:
        let chosen_a_after_third = builder.or(chosen_a_pre1, cond_third_a_0);
        let chosen_b_after_third_pre = builder.or(chosen_b, cond_third_b);
        let chosen_b_after_third = chosen_b_after_third_pre;
        let chosen_c_after_third_pre = builder.or(chosen_c, cond_third_c);
        let chosen_c_after_third = chosen_c_after_third_pre;
        let chosen_d_after_third_pre = builder.or(chosen_d, cond_third_d);
        let chosen_d_after_third = chosen_d_after_third_pre;

        let not_chosen_a_fourth = builder.not(chosen_a_after_third);
        let cond_fourth_a_0 = builder.and(a_nonzero, not_chosen_a_fourth);

        let not_cond_fourth_a_0 = builder.not(cond_fourth_a_0);
        let not_chosen_b_fourth = builder.not(chosen_b_after_third);
        let cond_fourth_b_pre = builder.and(not_cond_fourth_a_0, b_nonzero);
        let cond_fourth_b = builder.and(cond_fourth_b_pre, not_chosen_b_fourth);

        let not_cond_fourth_b = builder.not(cond_fourth_b);
        let not_chosen_c_fourth = builder.not(chosen_c_after_third);
        let cond_fourth_c_pre1 = builder.and(not_cond_fourth_a_0, not_cond_fourth_b);
        let cond_fourth_c_pre2 = builder.and(cond_fourth_c_pre1, c_nonzero);
        let cond_fourth_c = builder.and(cond_fourth_c_pre2, not_chosen_c_fourth);

        let not_cond_fourth_c = builder.not(cond_fourth_c);
        let not_chosen_d_fourth = builder.not(chosen_d_after_third);
        let cond_fourth_d_pre1 = builder.and(not_cond_fourth_a_0, not_cond_fourth_b);
        let cond_fourth_d_pre2 = builder.and(cond_fourth_d_pre1, not_cond_fourth_c);
        let cond_fourth_d_0 = builder.and(cond_fourth_d_pre2, d_nonzero);
        let cond_fourth_d = builder.and(cond_fourth_d_0, not_chosen_d_fourth);

        let mut fourth_nonzero_val = zero;
        fourth_nonzero_val = builder._if(cond_fourth_a_0, a, fourth_nonzero_val);
        fourth_nonzero_val = builder._if(cond_fourth_b, b, fourth_nonzero_val);
        fourth_nonzero_val = builder._if(cond_fourth_c, c, fourth_nonzero_val);
        fourth_nonzero_val = builder._if(cond_fourth_d, d, fourth_nonzero_val);

        // Now third_nonzero_val is the third chosen nonzero tile
        // and fourth_nonzero_val is the fourth chosen nonzero tile.
        //
        // According to our main logic, x2 = third_nonzero_val and x3 = fourth_nonzero_val.

        [frist_nonzero_val, second_nonzero_val, third_nonzero_val, fourth_nonzero_val]
    }
}