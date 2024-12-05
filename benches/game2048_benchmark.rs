use criterion::{criterion_group, criterion_main, Criterion};
use game2048_plonky2::game2048::{Game2048Circuit, F};
use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::config::PoseidonGoldilocksConfig;

fn game2048_generate_proof(c: &mut Criterion) {
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

    let direction = F::from_canonical_u32(0); // Direction: "up"

    c.bench_function("game2048_prove_and_verify", |b| {
        b.iter(|| {
            let (builder, targets) = Game2048Circuit::build_circuit();

            let circuit = builder.build::<PoseidonGoldilocksConfig>();
            let mut pw = PartialWitness::<F>::new();
            let before_board_targets = &targets[0..16];
            let after_board_targets = &targets[16..32];
            let direction_target = targets[32];

            // Assign values for before_board
            for (i, &target) in before_board_targets.iter().enumerate() {
                pw.set_target(target, before_board[i]).expect("Failed to set target");
            }

            // Assign values for after_board
            for (i, &target) in after_board_targets.iter().enumerate() {
                pw.set_target(target, after_board[i]).expect("Failed to set target");
            }

            // Assign the direction
            pw.set_target(direction_target, direction).expect("Failed to set target");

            let proof = circuit.prove(pw);
            assert!(circuit.verify(proof.unwrap()).is_ok(), "Proof verification failed");
        });
    });
}

criterion_group!(game2048_benchmark, game2048_generate_proof);
criterion_main!(game2048_benchmark);