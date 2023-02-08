use std::rc::Rc;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{read_keypair_file, Signature},
        signature::{Keypair, Signer},
    },
    Client, Cluster, Program,
};

use anchor_lang::Id;
use chain_drive::{instructions::summon::DataToBeSummoned, ID as PROGRAM_ID};
use clockwork_sdk::state::Thread;
use sha2::{Digest, Sha256};

#[test]
fn test_summon() {
    // Get dev and mint key.
    let dev_key: Rc<Keypair> =
        Rc::new(read_keypair_file("../../dev.json").expect("Example requires a keypair file"));

    // Get client, program, and rpc client
    let url: Cluster = Cluster::Devnet;
    let client: Client = Client::new_with_options(
        url,
        Rc::clone(&dev_key) as Rc<dyn Signer>,
        CommitmentConfig::processed(),
    );
    let program: Program = client.program(PROGRAM_ID);

    // Instruction arguments
    let source = String::from("google.com");
    let data = vec![1; 16];
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let hash: [u8; 32] = hasher.finalize().try_into().unwrap();
    let slot_delay = 0;
    let data_len = data.len();

    // Get metadata PDA
    let metadata_pda: Pubkey = {
        let mut bump: u8 = 255;
        loop {
            if let Ok(pda) = Pubkey::create_program_address(
                &[dev_key.pubkey().as_ref(), source.as_ref(), &[bump]],
                &program.id(),
            ) {
                break pda;
            } else {
                bump -= 1;
            }
        }
    };
    let sdrive_automation: Pubkey = Thread::pubkey(metadata_pda, source.clone());

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
            source: source.clone(),
            slot_delay,
            hash,
            data_len,
        })
        .signer(&*dev_key)
        .send()
        .unwrap();
    println!("summon tx signature: {summon_sig}");

    let metadata: DataToBeSummoned = program.account(metadata_pda).unwrap();

    assert_eq!(metadata.source, source, "source");
    assert_eq!(metadata.hash, hash, "hash");

    // Construct and send upload instruction
    let upload_sig: Signature = program
        .request()
        .accounts(chain_drive::accounts::Upload {
            uploader: dev_key.pubkey(),
            metadata: metadata_pda,
        })
        .args(chain_drive::instruction::Upload { data: data.clone() })
        .send()
        .unwrap();
    println!("upload tx signature: {upload_sig}");

    let metadata: DataToBeSummoned = program.account(metadata_pda).unwrap();

    assert_eq!(metadata.data, data, "data");
    assert_eq!(metadata.uploader, dev_key.pubkey(), "uploader");

    while program.rpc().get_slot().unwrap() < metadata.slot {}

    // Construct and send delete instruction
    let delete_sig: Signature = program
        .request()
        .accounts(chain_drive::accounts::Delete {
            uploader: dev_key.pubkey(),
            summoner: dev_key.pubkey(),
            metadata: metadata_pda,
        })
        .args(chain_drive::instruction::Delete {})
        .send()
        .unwrap();
    println!("delete tx signature: {delete_sig}");

    assert!(program.account::<DataToBeSummoned>(metadata_pda).is_err());
}
