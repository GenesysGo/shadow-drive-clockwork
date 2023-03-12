use std::{error::Error, path::PathBuf, rc::Rc};

use anchor_client::{
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        instruction::Instruction,
        message::Message,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair},
        signer::Signer,
        transaction::Transaction,
    },
    Cluster,
};
use anchor_spl::{
    associated_token::get_associated_token_address,
    token::{
        self,
        spl_token::instruction::{initialize_mint, mint_to},
        Mint,
    },
};

pub fn mock_shdw_mint(key: Rc<dyn Signer>) -> Result<Pubkey, Box<dyn Error>> {
    let rpc = RpcClient::new(Cluster::Localnet.url());

    // Get token keypair and admin ata
    let token_keypair: Keypair = read_keypair_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("RUNEkHeD5P8DsSpuDwxyZZKsj3T9e1ooMiaXL9H71yc.json"),
    )?;
    let admin_ata: Pubkey =
        get_associated_token_address(&key.pubkey(), &token_keypair.pubkey());

    // Create instructions to initialize mint and mint to admin key
    let pay_rent_and_create_account_ix: Instruction =
        anchor_client::solana_sdk::system_instruction::create_account(
            &key.pubkey(),
            &token_keypair.pubkey(),
            rpc.get_minimum_balance_for_rent_exemption(Mint::LEN)?,
            Mint::LEN as u64,
            &token::ID,
        );
    let init_mint_ix: Instruction = initialize_mint(
        &anchor_spl::token::ID,
        &token_keypair.pubkey(),
        &key.pubkey(),
        None,
        9,
    )?;
    let account_init_ix: Instruction = spl_associated_token_account::instruction::create_associated_token_account(
        &key.pubkey(),
        &key.pubkey(),
        &token_keypair.pubkey(),
        &anchor_spl::token::ID,
    );
    let mint_to_ix: Instruction = mint_to(
        &anchor_spl::token::ID,
        &token_keypair.pubkey(),
        &admin_ata,
        &key.pubkey(),
        &[&key.pubkey()],
        1_000_000_000,
    )?;

    // Build and sign transaction
    let message: Message = Message::new(
        &[
            pay_rent_and_create_account_ix,
            init_mint_ix,
            account_init_ix,
            mint_to_ix,
        ],
        Some(&key.pubkey()),
    );
    let mut transaction: Transaction = Transaction::new_unsigned(message);
    transaction.sign(&[&token_keypair, &*key], rpc.get_latest_blockhash()?);

    // Send transaction
    if let Err(e) = rpc.send_and_confirm_transaction(&transaction) {
        panic!("failed to initialize mint {e:#?}");
    }

    Ok(admin_ata)
}
