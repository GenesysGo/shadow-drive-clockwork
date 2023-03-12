use std::{error::Error, path::PathBuf, rc::Rc, str::FromStr};

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

use anchor_spl::token;
use chain_drive::{
    constants::TIME_DELAY_SECS,
    instructions::{init::portal_config, summon::DataToBeSummoned},
    shdw,
};
use sha2::{Digest, Sha256};
use shadow_portal_tests::mock_shdw_mint;

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
    let data_len = data.len();

    // Get metadata PDA
    let metadata_pda: Pubkey = Pubkey::find_program_address(
        &[
            admin_key.pubkey().as_ref(),
            storage_account.as_ref(),
            filename.as_ref(),
        ],
        &chain_drive::ID,
    )
    .0;
    let metadata_vault: Pubkey = Pubkey::find_program_address(
        &[metadata_pda.as_ref()],
        &chain_drive::ID,
    )
    .0;

    // Construct and send summon instruction
    let summon_sig: Signature = match portal_program
        .request()
        .accounts(chain_drive::accounts::Summon {
            summoner: admin_key.pubkey(),
            payer: admin_key.pubkey(),
            summoner_token_account: admin_ata,
            metadata: metadata_pda,
            system_program: system_program::ID,
            portal_config: portal_config(),
            shdw_vault: metadata_vault,
            shdw_mint: shdw::ID,
            token_program: token::ID,
        })
        .args(chain_drive::instruction::Summon {
            storage_account,
            filename: filename.to_string(),
            callback: None,
            hash,
            data_len,
            extra_lamports: 0,
            unique_thread: None,
        })
        .signer(&*admin_key)
        .send()
    {
        Err(e) => panic!("failed summon tx: {e:#?}"),
        Ok(sig) => sig,
    };
    println!("summon tx signature: {summon_sig}");

    let metadata: DataToBeSummoned =
        portal_program.account(metadata_pda).unwrap();
    assert_eq!(metadata.storage_account, storage_account, "storage_account");
    assert_eq!(metadata.filename, filename, "filename");
    assert_eq!(metadata.hash, hash, "hash");
    println!("\nUser summoned data on-chain");

    loop {
        let metadata = portal_program
            .account::<DataToBeSummoned>(metadata_pda)
            .unwrap();
        if !metadata.data.is_empty() {
            assert_eq!(metadata.data, data, "data");
            println!(
                "\nData uploaded from sdrive to solana by clockwork plugin"
            );
            break;
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(
        TIME_DELAY_SECS as u64 + 5,
    ));
    assert!(
        portal_program
            .account::<DataToBeSummoned>(metadata_pda)
            .is_err(),
        "account should be deleted by clockwork thread"
    );
    println!(
        "\nData successfully deleted from solana by clockwork thread\n
    "
    );

    Ok(())
}
