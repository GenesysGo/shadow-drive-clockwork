use std::{error::Error, path::PathBuf, rc::Rc, str::FromStr};

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
use chain_drive::{instructions::init::portal_config, PortalConfig};
use shadow_portal_tests::mock_shdw_mint;

fn main() -> Result<(), Box<dyn Error>> {
    // Get dev and mint key.
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
    let user_program: Program = client.program(chain_drive_demo::ID);

    if let Err(e) = portal_program
        .request()
        .accounts(chain_drive::accounts::Init {
            payer: admin_key.pubkey(),
            config: portal_config(),
            system_program: system_program::ID,
        })
        .args(chain_drive::instruction::Init {})
        .send()
    {
        panic!("failed to initialize portal config {e:#?}");
    }
    let portal_config_account: PortalConfig =
        portal_program.account(portal_config())?;
    assert_eq!(
        portal_config_account.admin,
        chain_drive::payout_authority::ID
    );
    assert_eq!(portal_config_account.shades_per_byte, chain_drive::INIT_FEE);

    // Instruction arguments
    let storage_account =
        Pubkey::from_str("53AqvNpBsk3wci9do6buRwaRr3spLZE1ySNfEYxMZEqG")
            .unwrap();
    println!("storage account {:?}", storage_account.to_bytes());
    let filename = "test.txt";

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
    let shdw_vault: Pubkey = Pubkey::find_program_address(
        &[metadata_pda.as_ref()],
        &chain_drive::ID,
    )
    .0;

    // Construct and send summon instruction
    let portal_config_info = user_program.rpc().get_account(&portal_config())?;
    println!("portal config is owned by {}", portal_config_info.owner);
    if let Err(e) = user_program
        .request()
        .accounts(chain_drive_demo::accounts::Initialize {
            summoner: admin_key.pubkey(),
            summoner_token_account: admin_ata,
            metadata: metadata_pda,
            config: portal_config(),
            shdw_vault,
            shdw_mint: chain_drive::shdw::ID,
            portal_program: chain_drive::ID,
            token_program: token::ID,
            system_program: system_program::ID,
        })
        .args(chain_drive_demo::instruction::Initialize {})
        .signer(&*admin_key)
        .send()
    {
        panic!("kickoff ix failed: {e:#?}");
    }
    Ok(())
}
