use std::rc::Rc;

use anchor_client::{
    anchor_lang::system_program,
    solana_client::rpc_config::RpcSendTransactionConfig,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{read_keypair_file, Signature},
        signature::{Keypair, Signer},
    },
    Client, Cluster, Program,
};

use chain_drive::{
    clockwork_sdk::state::Thread, instructions::summon::DataToBeSummoned,
};
use graph_demo::machine;

runes::inscribe_runes!("../../../graph-demo/nodes/nodes.runes");

fn main() {
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

    // Get first metadata PDA
    let metadata_pda: Pubkey = Pubkey::find_program_address(
        &[
            machine().as_ref(),
            // dev_key.pubkey().as_ref(),
            storage_account.as_ref(),
            "Alice".as_ref(),
        ],
        &chain_drive::ID,
    )
    .0;

    // Construct and send summon instruction
    let summon_sig: Signature = match program
        .request()
        .accounts(graph_demo::accounts::Initialize {
            admin: dev_key.pubkey(),
            machine: machine(),
            metadata: metadata_pda,
            // thread: Thread::pubkey(machine(), b"graph".to_vec()),
            // thread_program: clockwork_sdk::ID,
            portal_program: chain_drive::ID,
            system_program: system_program::ID,
        })
        .args(graph_demo::instruction::Initialize {})
        .signer(&*dev_key)
        .send()
        // .send_with_spinner_and_config(RpcSendTransactionConfig {
        //     skip_preflight: true,
        //     ..RpcSendTransactionConfig::default()
        // }) 
        {
            Ok(sig) => sig,
            Err(e) => panic!("{e:#?}"),
        };
    let metadata: DataToBeSummoned = program.account(metadata_pda).unwrap();
    println!("{metadata:#?}");
}
