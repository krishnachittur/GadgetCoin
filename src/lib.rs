extern crate ring;

#[cfg(test)]
mod tests;
mod ethtypes;

pub use ethtypes::*;

pub fn run() {
    println!("Hello, world!");
}