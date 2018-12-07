#[macro_use]
extern crate serde_derive;

extern crate bincode;
extern crate rand;
extern crate ring;
extern crate secp256k1;
extern crate serde;
extern crate sha3;

#[cfg(test)]
mod tests;

pub mod eth;

pub fn run() {
    println!("Hello, world!");
}
