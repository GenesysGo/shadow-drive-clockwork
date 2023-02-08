use std::{rc::Rc, str::FromStr};

use anchor_client::{
    anchor_lang::{system_program, Id},
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{read_keypair_file, Signature},
        signature::{Keypair, Signer},
    },
    Client, Cluster, Program,
};

use chain_drive::{instructions::summon::DataToBeSummoned, ID as PROGRAM_ID};
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
    let filename = "test.txt";
    let data = reqwest::blocking::get(DataToBeSummoned::build_source(&storage_account, &filename))
        .unwrap()
        .bytes()
        .unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let hash: [u8; 32] = hasher.finalize().try_into().unwrap();
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
    let automation_id = metadata_pda.to_bytes().to_vec();
    let sdrive_automation: Pubkey = Thread::pubkey(metadata_pda, automation_id);

    // Construct and send summon instruction
    let summon_sig: Signature = program
        .request()
        .accounts(chain_drive::accounts::Summon {
            summoner: dev_key.pubkey(),
            metadata: metadata_pda,
            system_program: system_program::ID,
            sdrive_automation,
            automation_program: clockwork_sdk::ThreadProgram::id(),
        })
        .args(chain_drive::instruction::Summon {
            storage_account,
            filename: filename.to_string(),
            slot_delay,
            hash,
            data_len,
        })
        .signer(&*dev_key)
        .send()
        .unwrap();
    println!("summon tx signature: {summon_sig}");

    std::thread::sleep(std::time::Duration::from_secs(1));
    let metadata: DataToBeSummoned = program.account(metadata_pda).unwrap();

    assert_eq!(metadata.storage_account, storage_account, "storage_account");
    assert_eq!(metadata.filename, filename, "filename");
    assert_eq!(metadata.hash, hash, "hash");

    // // Construct and send upload instruction
    // let upload_sig: Signature = program
    //     .request()
    //     .accounts(chain_drive::accounts::Upload {
    //         uploader: dev_key.pubkey(),
    //         metadata: metadata_pda,
    //     })
    //     .args(chain_drive::instruction::Upload { data: data.clone() })
    //     .send()
    //     .unwrap();
    // println!("upload tx signature: {upload_sig}");

    std::thread::sleep(std::time::Duration::from_secs(5));
    let metadata: DataToBeSummoned = program.account(metadata_pda).unwrap();

    assert_eq!(metadata.data, data, "data");
    // assert_eq!(metadata.uploader, dev_key.pubkey(), "uploader");

    // while program.rpc().get_slot().unwrap() < metadata.slot {}

    // // Construct and send delete instruction
    // let delete_sig: Signature = program
    //     .request()
    //     .accounts(chain_drive::accounts::Delete {
    //         uploader: dev_key.pubkey(),
    //         summoner: dev_key.pubkey(),
    //         metadata: metadata_pda,
    //     })
    //     .args(chain_drive::instruction::Delete {})
    //     .send()
    //     .unwrap();
    // println!("delete tx signature: {delete_sig}");

    // assert!(program.account::<DataToBeSummoned>(metadata_pda).is_err());
}
