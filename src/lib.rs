#[macro_use]
extern crate serde_derive;

extern crate ring;
extern crate secp256k1;
extern crate rand;
extern crate serde;
extern crate bincode;
extern crate sha3;

#[cfg(test)]
mod tests;

pub mod eth;

pub fn run() {
    println!("Hello, world!");
}
