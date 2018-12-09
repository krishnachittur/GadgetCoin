use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use super::*;

pub const TXNS_PER_BLOCK: usize = 1;
pub const TOTAL_TXNS: usize = 200;
pub const DIFFICULTY: u32 = 18;
pub const NUM_THREADS: u32 = 12;
pub const HASH_GRANULARITY: u32 = 50;

pub struct TxnGenerator<'a> {
    remaining: usize,
    lastindex: usize,
    actors: &'a Actors,
    possibilities: Vec<ETHTxn>,
}

// for simulation purposes
pub struct Actor {
    pub secretkey: secp256k1::SecretKey,
    pub pubkey: secp256k1::PublicKey,
    pub address: ETHAddress,
}

pub type Actors = Vec<Actor>;

pub fn generate_actors() -> Actors {
    let mut all_actors = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..3 {
        let secretkey = secp256k1::SecretKey::random(&mut rng);
        let pubkey = secp256k1::PublicKey::from_secret_key(&secretkey);
        let address = ETHTxn::get_address_from_public_key(&pubkey).unwrap();
        all_actors.push(Actor {
            secretkey,
            pubkey,
            address,
        });
    }
    all_actors
}

impl<'a> TxnGenerator<'a> {
    pub fn new(actors: &Actors, maxiter: usize) -> TxnGenerator {
        // possible code segments
        // 0 -> empty (costs almost no gas)
        // 1 -> adds 2 to value
        // 2 -> sets value to 6

        let codes = vec![
            // empty code
            vec![],
            // [Op::PUSH1(2), Op::ADDVAL, Op::STOP];
            vec![0x60, 2, 0xb1, 0x00],
            // [PUSH1(2), PUSH1(3), PUSH1(4), PUSH1(7), PUSH1(1), ADD, SUB, MUL, DIV, SETVAL, STOP]
            vec![
                0x60, 2, 0x60, 3, 0x60, 4, 0x60, 7, 0x60, 1, 0x01, 0x03, 0x02, 0x04, 0xb0, 0x00,
            ],
        ];

        TxnGenerator {
            lastindex: 0,
            remaining: maxiter,
            actors,
            possibilities: vec![
                // from actor 0 to actor 1
                ETHTxn {
                    nonce: 0,
                    gasprice: Wei::from_wei(2),
                    gaslimit: 10,
                    recipient: actors[1].address.clone(),
                    value: Wei::from_wei(100),
                    code: codes[0].clone(),
                    ecdsa_fields: ethtxn::utils::get_bs_ecsda_field(&actors[0].secretkey.clone()),
                },
                // from actor 1 to actor 2
                ETHTxn {
                    nonce: 0,
                    gasprice: Wei::from_wei(1),
                    gaslimit: 10,
                    recipient: actors[2].address.clone(),
                    value: Wei::from_wei(76), // incremented to 78 by code
                    code: codes[1].clone(),
                    ecdsa_fields: ethtxn::utils::get_bs_ecsda_field(&actors[1].secretkey.clone()),
                },
                // from actor 2 to actor 0
                ETHTxn {
                    nonce: 0,
                    gasprice: Wei::from_wei(1),
                    gaslimit: 30,
                    recipient: actors[0].address.clone(),
                    value: Wei::from_wei(0), // set to 6 by code
                    code: codes[2].clone(),
                    ecdsa_fields: ethtxn::utils::get_bs_ecsda_field(&actors[2].secretkey.clone()),
                },
            ],
        }
    }
}

impl<'a> Iterator for TxnGenerator<'a> {
    type Item = ETHTxn;
    fn next(&mut self) -> Option<Self::Item> {
        self.remaining = match self.remaining.checked_sub(1) {
            Some(val) => val,
            None => return None,
        };

        self.possibilities[self.lastindex].nonce += 1;
        self.possibilities[self.lastindex].sign_transaction(&self.actors[self.lastindex].secretkey);

        let txn = Some(self.possibilities[self.lastindex].clone());
        self.lastindex = (self.lastindex + 1) % self.possibilities.len();
        txn
    }
}

pub fn hash_block_sequential(block: &mut ETHBlock) {
    let mut rng = rand::thread_rng();
    while !block.is_valid() {
        block.randomize_nonce(&mut rng);
    }
}

pub fn hash_block_parallel(block: &mut ETHBlock, num_threads: u32) {
    // channel for child to indicate success
    let (found_tx, found_rx) = mpsc::sync_channel(num_threads as usize);
    // channels for parent to indicate child success
    let mut done_txs: Vec<mpsc::Sender<bool>> = vec![];
    let mut done_rxs: Vec<Option<mpsc::Receiver<bool>>> = vec![];

    for _ in 0..num_threads {
        let (done_tx, done_rx) = mpsc::channel();
        done_rxs.push(Some(done_rx));
        done_txs.push(done_tx);
    }

    for i in 0..num_threads {
        let found_tx = found_tx.clone();
        let mut block = block.clone();
        let done_rx = done_rxs[i as usize].take().unwrap();
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            loop {
                for _ in 0..HASH_GRANULARITY {
                    block.randomize_nonce(&mut rng);
                    if block.is_valid() {
                        match found_tx.send(block.get_nonce()) {
                            _ => {}
                        };
                        return;
                    }
                }
                if let Ok(_) = done_rx.try_recv() {
                    // some other child has succeeded
                    return;
                }
            }
        });
    }
    let nonce = found_rx.recv().unwrap();

    for i in 0..num_threads {
        // terminate other children
        match done_txs[i as usize].send(true) {
            _ => {}
        };
    }
    block.set_nonce(nonce);
}

pub fn make_blockchain<F>(actors: &Actors, blockchain: &mut ETHBlockchain, mut block_hasher: F)
where
    F: FnMut(&mut ETHBlock),
{
    let mut txns = TxnGenerator::new(actors, TOTAL_TXNS);
    loop {
        match txns.next() {
            Some(txn) => {
                let blk = match blockchain.process_transaction(txn) {
                    Some(mut blk) => {
                        block_hasher(&mut blk);
                        blk
                    }
                    None => continue,
                };
                assert!(blockchain.add_block(blk));
            }
            None => break,
        }
    }
}

// generates new blockchain with a single block (so the miner has a block reward)
pub fn generate_blockchain(actors: &Actors) -> ETHBlockchain {
    let mut chain = ETHBlockchain::new(TXNS_PER_BLOCK, DIFFICULTY, actors[0].address);
    let mut block = chain.flush_txns();
    hash_block_sequential(&mut block);
    chain.add_block(block);
    chain
}

pub fn run_benchmarks() {
    let actors = generate_actors();
    let mut blockchain = generate_blockchain(&actors);
    let start = Instant::now();
    make_blockchain(&actors, &mut blockchain, hash_block_sequential);
    let duration = start.elapsed();
    let seconds = (duration.as_secs() as f64) + (duration.subsec_nanos() as f64 / 1_000_000_000.0);
    println!("Sequential hashing benchmark ran in {} seconds", seconds);

    let actors = generate_actors();
    let mut blockchain = generate_blockchain(&actors);
    let start = Instant::now();
    make_blockchain(&actors, &mut blockchain, |blk: &mut ETHBlock| {
        hash_block_parallel(blk, NUM_THREADS)
    });
    let duration = start.elapsed();
    let seconds = (duration.as_secs() as f64) + (duration.subsec_nanos() as f64 / 1_000_000_000.0);
    println!("Parallel hashing benchmark ran in {} seconds", seconds);
}
