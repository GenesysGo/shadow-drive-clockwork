use std::{error::Error, path::PathBuf, rc::Rc, time::Duration};

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
use anchor_spl::token;
use base58::ToBase58;
use chain_drive::instructions::{
    init::portal_config, summon::DataToBeSummoned,
};
use graph_demo::{machine, Machine};
use runes::inscribe_runes;
use shadow_portal_tests::mock_shdw_mint;

inscribe_runes!("../../../graph-demo/nodes/nodes.runes");

fn main() -> Result<(), Box<dyn Error>> {
    // Get admin and mint key.
    let admin_key: Rc<Keypair> = Rc::new(
        read_keypair_file(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("admin.json"),
        )
        .expect("example requires a keypair"),
    );
    let admin_ata = mock_shdw_mint(Rc::clone(&admin_key) as Rc<dyn Signer>)?;

    // Get client, program, and rpc client
    let url: Cluster = Cluster::Localnet;
    let client: Client = Client::new_with_options(
        url,
        Rc::clone(&admin_key) as Rc<dyn Signer>,
        CommitmentConfig::processed(),
    );
    let portal_program: Program = client.program(chain_drive::ID);
    let user_program: Program = client.program(graph_demo::ID);

    portal_program
        .request()
        .accounts(chain_drive::accounts::Init {
            payer: admin_key.pubkey(),
            config: portal_config(),
            system_program: system_program::ID,
        })
        .args(chain_drive::instruction::Init {})
        .send()
        .unwrap();

    // Instruction arguments
    let storage_account = unsafe { get_runes_unchecked().storage_account };

    // Get first metadata PDA (for Alice)
    let metadata_pda: Pubkey = DataToBeSummoned::get_pda(
        &machine(),
        &Pubkey::new_from_array(storage_account),
        "Alice",
        Some(0),
    );
    let machine_vault =
        Pubkey::find_program_address(&[machine().as_ref()], &graph_demo::ID).0;
    let metadata_vault = Pubkey::find_program_address(
        &[metadata_pda.as_ref()],
        &chain_drive::ID,
    )
    .0;

    // Construct and send initial kickoff instruction
    if let Err(e) = user_program
        .request()
        .accounts(graph_demo::accounts::Initialize {
            admin: admin_key.pubkey(),
            admin_token_account: admin_ata,
            machine: machine(),
            machine_vault,
            metadata_vault: metadata_vault,
            metadata: metadata_pda,
            portal_config: portal_config(),
            shdw_mint: chain_drive::shdw::ID,
            token_program: token::ID,
            portal_program: chain_drive::ID,
            system_program: system_program::ID,
        })
        .args(graph_demo::instruction::Initialize {})
        .signer(&*admin_key)
        .send()
    {
        panic!("failed to kickoff graph with {e:#?}");
    }

    let mut counter = 0;
    let mut last_counter = u64::MAX;
    while counter < 100 {
        let machine: Machine = user_program.account(machine()).unwrap();
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
