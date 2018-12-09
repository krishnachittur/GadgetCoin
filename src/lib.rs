#[macro_use]
extern crate serde_derive;

extern crate bincode;
extern crate rand;
extern crate ring;
extern crate secp256k1;
extern crate serde;
extern crate sha3;

pub mod eth;

pub fn run() {
    eth::benchutils::run_benchmarks();
}
