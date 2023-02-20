use std::{rc::Rc, str::FromStr};

use anchor_client::{
    anchor_lang::{solana_program, system_program, Id, InstructionData},
    solana_sdk::{
        commitment_config::CommitmentConfig,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::{read_keypair_file, Signature},
        signature::{Keypair, Signer},
        transaction::Transaction,
    },
    Client, Cluster, Program,
};

use chain_drive::{
    constants::TIME_DELAY_SECS, instruction::Upload, instructions::summon::DataToBeSummoned,
};
use chain_drive_demo::ID as PROGRAM_ID;
use clockwork_sdk::state::Thread;
use sha2::{Digest, Sha256};

fn main() {
    // Get dev and mint key.
    let dev_key: Rc<Keypair> =
        Rc::new(read_keypair_file("dev.json").expect("Example requires a keypair file"));

    // Get client, program, and rpc client
    let url: Cluster = Cluster::Localnet;
    let client: Client = Client::new_with_options(
        url,
        Rc::clone(&dev_key) as Rc<dyn Signer>,
        CommitmentConfig::processed(),
    );
    let program: Program = client.program(PROGRAM_ID);

    // Instruction arguments
    let storage_account = Pubkey::from_str("53AqvNpBsk3wci9do6buRwaRr3spLZE1ySNfEYxMZEqG").unwrap();
    println!("storage account {:?}", storage_account.to_bytes());
    let filename = "test.txt";
    let data = reqwest::blocking::get(DataToBeSummoned::build_source(&storage_account, &filename))
        .unwrap()
        .bytes()
        .unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let hash: [u8; 32] = hasher.finalize().try_into().unwrap();
    println!("{hash:?}");
    let slot_delay = 0;
    let data_len = data.len();

    // Get metadata PDA
    let metadata_pda: Pubkey = Pubkey::find_program_address(
        &[
            dev_key.pubkey().as_ref(),
            storage_account.as_ref(),
            filename.as_ref(),
        ],
        &chain_drive::ID,
    )
    .0;
    let automation_id = metadata_pda.to_bytes().to_vec();
    let sdrive_automation: Pubkey = Thread::pubkey(metadata_pda, automation_id);

    // Construct and send summon instruction
    let summon_sig: Signature = program
        .request()
        .accounts(chain_drive_demo::accounts::Initialize {
            signer: dev_key.pubkey(),
            metadata: metadata_pda,
            system_program: system_program::ID,
            portal_program: chain_drive::ID,
        })
        .args(chain_drive_demo::instruction::Initialize {})
        .signer(&*dev_key)
        .send()
        .unwrap();
    println!("summon tx signature: {summon_sig}");
}
