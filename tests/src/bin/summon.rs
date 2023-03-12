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
    constants::TIME_DELAY_SECS, instruction::Upload,
    instructions::summon::DataToBeSummoned, ID as PROGRAM_ID,
};
use clockwork_sdk::state::Thread;
use sha2::{Digest, Sha256};

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
    let program: Program = client.program(PROGRAM_ID);

    // Instruction arguments
    let storage_account =
        Pubkey::from_str("53AqvNpBsk3wci9do6buRwaRr3spLZE1ySNfEYxMZEqG")
            .unwrap();
    println!("storage account {:?}", storage_account.to_bytes());
    let filename = "test.txt";
    let data = reqwest::blocking::get(DataToBeSummoned::build_source(
        &storage_account,
        &filename,
    ))
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
        &program.id(),
    )
    .0;

    // Construct and send summon instruction
    let summon_sig: Signature = program
        .request()
        .accounts(chain_drive::accounts::Summon {
            summoner: dev_key.pubkey(),
            payer: dev_key.pubkey(),
            metadata: metadata_pda,
            system_program: system_program::ID,
        })
        .args(chain_drive::instruction::Summon {
            storage_account,
            filename: filename.to_string(),
            callback: None,
            hash,
            data_len,
            extra_lamports: 0,
            unique_thread: 0,
        })
        .signer(&*dev_key)
        .send()
        .unwrap();
    println!("summon tx signature: {summon_sig}");

    let metadata: DataToBeSummoned = program.account(metadata_pda).unwrap();
    assert_eq!(metadata.storage_account, storage_account, "storage_account");
    assert_eq!(metadata.filename, filename, "filename");
    assert_eq!(metadata.hash, hash, "hash");
    println!("\nUser summoned data on-chain");

    loop {
        let metadata =
            program.account::<DataToBeSummoned>(metadata_pda).unwrap();
        if !metadata.data.is_empty() {
            assert_eq!(metadata.data, data, "data");
            println!(
                "\nData uploaded from sdrive to solana by clockwork plugin"
            );
            break;
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(
        TIME_DELAY_SECS as u64 + 1,
    ));
    assert!(
        program.account::<DataToBeSummoned>(metadata_pda).is_err(),
        "account should be deleted by clockwork thread"
    );
    println!(
        "\nData successfully deleted from solana by clockwork thread\n
    "
    );
}
