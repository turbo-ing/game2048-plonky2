use criterion::{criterion_group, criterion_main, Criterion};
use game2048_plonky2::game2048::{Game2048Circuit, F, D};
use plonky2::field::types::Field;
use plonky2::iop::witness::PartialWitness;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
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

    let direction = "up";

    c.bench_function("game2048_prove_and_verify", |b| {
        b.iter(|| {
            let config = CircuitConfig::standard_recursion_config();
            let mut builder = CircuitBuilder::<F, D>::new(config);

            Game2048Circuit::build_circuit(&mut builder, &before_board, &after_board, direction);

            let circuit = builder.build::<PoseidonGoldilocksConfig>();
            let pw = PartialWitness::<F>::new();

            let proof = circuit.prove(pw);
            assert!(circuit.verify(proof.unwrap()).is_ok(), "Proof verification failed");
        });
    });
}

criterion_group!(game2048_benchmark, game2048_generate_proof);
criterion_main!(game2048_benchmark);