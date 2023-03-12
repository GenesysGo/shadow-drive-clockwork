use std::{error::Error, rc::Rc, time::Duration};

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::read_keypair_file,
        signature::{Keypair, Signer},
    },
    Client, Cluster, Program,
};
use base58::ToBase58;
use graph_demo::{machine, Machine};
use runes::inscribe_runes;

inscribe_runes!("../../../graph-demo/nodes/nodes.runes");

fn main() -> Result<(), Box<dyn Error>> {
    // Get dev and mint key.
    let dev_key: Rc<Keypair> = Rc::new(
        read_keypair_file("dev.json").expect("Example requires a keypair file"),
    );

    // Get client, program, and rpc client
    let url: Cluster = Cluster::Localnet;
    let client: Client = Client::new_with_options(
        url,
        Rc::clone(&dev_key) as Rc<dyn Signer>,
        CommitmentConfig::processed(),
    );
    let program: Program = client.program(dbg!(graph_demo::ID));

    // Instruction arguments
    let storage_account = unsafe { get_runes().storage_account };

    // Get first metadata PDA (for Alice)
    let metadata_pda: Pubkey = Pubkey::find_program_address(
        &[
            machine().as_ref(),
            storage_account.as_ref(),
            "Alice".as_ref(),
        ],
        &chain_drive::ID,
    )
    .0;

    // Construct and send initial kickoff instruction
    program
        .request()
        .accounts(graph_demo::accounts::Initialize {
            admin: dev_key.pubkey(),
            machine: machine(),
            metadata: metadata_pda,
            portal_program: chain_drive::ID,
            system_program: system_program::ID,
        })
        .args(graph_demo::instruction::Initialize {})
        .signer(&*dev_key)
        .send()?;

    let mut counter = 0;
    let mut last_counter = u64::MAX;
    while counter < 100 {
        let machine: Machine = program.account(machine()).unwrap();
        counter = machine.counter;
        if counter != last_counter {
            last_counter = counter;
            println!(
                "state: counter = {}; hash = {}; next = {}; ",
                machine.counter,
                machine.hash.to_base58(),
                &machine.next
            );
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    println!("exiting");

    Ok(())
}
