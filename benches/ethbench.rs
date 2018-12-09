#[macro_use]
extern crate criterion;
extern crate gadgetcoin;

use criterion::{Criterion, Fun};

use gadgetcoin::eth::benchutils::*;
use gadgetcoin::eth::ETHBlock;

fn thread_sweep(c: &mut Criterion) {
    c.bench_function_over_inputs("Parallel Hashing - Different # of Threads", |b, num_threads| {
        let actors_par = generate_actors();
        let mut parallel_blockchain = generate_blockchain(&actors_par);
        b.iter(|| {
            make_blockchain(
                &actors_par,
                &mut parallel_blockchain,
                |blk: &mut ETHBlock| hash_block_parallel(blk, *num_threads),
            );
        })
    }, 1..=24);
}

fn sequential_parallel_comparision(c: &mut Criterion) {
    let sequential_run = Fun::new("Sequential Hashing", |b, _| {
        let actors_seq = generate_actors();
        let mut sequential_blockchain = generate_blockchain(&actors_seq);
        b.iter(|| {
            make_blockchain(
                &actors_seq,
                &mut sequential_blockchain,
                hash_block_sequential,
            );
        })
    });

    let parallel_run = Fun::new("Parallel Hashing", |b, _| {
        let actors_par = generate_actors();
        let mut parallel_blockchain = generate_blockchain(&actors_par);
        b.iter(|| {
            make_blockchain(
                &actors_par,
                &mut parallel_blockchain,
                |blk: &mut ETHBlock| hash_block_parallel(blk, NUM_THREADS),
            );
        })
    });

    let functions = vec![sequential_run, parallel_run];
    c.bench_functions("Sequential vs. Parallel Hashing", functions, &20);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(2);
    targets = thread_sweep, sequential_parallel_comparision
}
criterion_main!(benches);
