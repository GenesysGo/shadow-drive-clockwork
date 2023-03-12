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
    token::spl_token::instruction::{
        initialize_account3, initialize_mint, mint_to,
    },
};

pub fn mock_shdw_mint(key: Rc<dyn Signer>) -> Result<(), Box<dyn Error>> {
    let rpc = RpcClient::new(Cluster::Localnet);

    // Get token keypair and admin ata
    let token_keypair: Keypair =
        read_keypair_file(PathBuf::from("../../key.json"))?;
    let admin_ata: Pubkey =
        get_associated_token_address(&key.pubkey(), &token_keypair.pubkey());

    // Create instructions to initialize mint and mint to admin key
    let init_mint_ix: Instruction = initialize_mint(
        &anchor_spl::token::ID,
        &token_keypair.pubkey(),
        &key.pubkey(),
        None,
        9,
    )?;
    let account_init_ix: Instruction = initialize_account3(
        &anchor_spl::token::ID,
        &admin_ata,
        &token_keypair.pubkey(),
        &key.pubkey(),
    )?;
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
        &[init_mint_ix, account_init_ix, mint_to_ix],
        Some(&key.pubkey()),
    );
    let mut transaction: Transaction = Transaction::new_unsigned(message);
    transaction.sign(&[&*key], rpc.get_latest_blockhash()?);

    // Send transaction
    rpc.send_and_confirm_transaction(&transaction)?;

    Ok(())
}
