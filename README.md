# Shadow Portal

Shadow Portal allows developers to summon data from Shadow Drive onto Solana, effectively attaching an archival hard drive on Solana.

- This codebase requires a [modified version of the Clockwork Plugin](https://github.com/genesysgo/clockwork). 
- A modified version of the Shadow Drive CLI and SDK is included

There are two example contracts which use the shadow portal contract. These are `chain-drive-demo`, and `graph-demo-onchain`. The code that was used to generate the graph nodes is in the `graph-demo/` directory.


# Using Shadow Portal on a Localnet
To run the shadow portal demo on a localnet
- Install solana 1.14.12
- Clone the GenesysGo/clockwork repo, switch to `sdrive` branch, and build the programs and plugin with `./scripts/build-all.sh`
- Clone this repo and `anchor build` all smart contracts
- Spin up a localnet with the relevant programs via
```
 clockwork localnet \
    --bpf-program G6xPudzNNM8CwfLHC9ByzrF67LcwyiRe4t9vHg34eqpR ./target/deploy/chain_drive.so \
    --bpf-program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS ./target/deploy/chain_drive_demo.so \
    --bpf-program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnW ./target/deploy/graph_demo.so
```
- Run the graph demo via `cargo run --release --bin graph`
