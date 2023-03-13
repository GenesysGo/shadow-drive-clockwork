# Shadow Portal

Shadow Portal allows developers to summon data from Shadow Drive onto Solana, effectively attaching an archival hard drive on Solana.

- This codebase requires a [modified version of the Clockwork Plugin](https://github.com/genesysgo/clockwork). 
- The shadow drive CLI, which facilitates the creation of summoning runes, and the `runes` library can be found [here](https://github.com/genesysgo/shadow-drive-rust)

There are two example contracts which use the shadow portal contract. These are `chain-drive-demo`, and `graph-demo-onchain`. The code that was used to generate the graph nodes is in the `graph-demo/` directory.