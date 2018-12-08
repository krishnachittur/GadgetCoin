# GadgetCoin

GadgetCoin is a modified partial implementation of the Ethereum specification in Rust. The original specification is available via the Ethereum [white paper](https://github.com/ethereum/wiki/wiki/White-Paper) and [yellow paper](https://ethereum.github.io/yellowpaper/paper.pdf).

We have added some instructions to the Ethereum ISA via previously unused opcodes. For convenience, stack items are only 8 bits rather than the standard 256, and smart contracts have been removed - instead, arbitrary Turing-complete code can execute directly from a user-supplied transaction.

## Goal
The goal of this project was to learn about Rust and Ethereum while benchmarking the performance of different concurrent and sequential implementations of nonce computation and blockchain verification.