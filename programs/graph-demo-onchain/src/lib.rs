use anchor_lang::{
    prelude::*, solana_program::native_token::LAMPORTS_PER_SOL, system_program,
    InstructionData,
};
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use chain_drive::{
    clockwork_sdk::state::{
        InstructionData as ClockworkInstructionData, ThreadResponse, Trigger,
    },
    instructions::summon::DataToBeSummoned,
    portal_config,
    program::ChainDrive,
    shdw, AccountMetaData, PortalConfig,
};
use graph_demo::*;
use runes::inscribe_runes;
use sha2::Digest;

inscribe_runes!("../nodes.runes");

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnW");

#[program]
pub mod graph_demo_onchain {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        // Initialize machine
        ctx.accounts.machine.admin = ctx.accounts.admin.key();
        ctx.accounts.machine.next = "Alice".to_string();

        // Get runes
        let runes = unsafe { get_runes_unchecked() };

        // Initialize machine with SOL and SHDW
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.admin.to_account_info(),
                    to: ctx.accounts.machine.to_account_info(),
                },
            ),
            10 * LAMPORTS_PER_SOL,
        )?;
        token::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.admin_token_account.to_account_info(),
                    to: ctx.accounts.machine_vault.to_account_info(),
                    authority: ctx.accounts.admin.to_account_info(),
                },
            ),
            1_000_000_000,
        )?;

        // Get Alice rune and summon Alice
        let alice_rune = runes.get_rune("Alice").unwrap();
        let signer_seeds: &[&[&[u8]]] = &[&[
            "state-machine".as_ref(),
            &[*ctx.bumps.get("machine").unwrap()],
        ]];
        let cpi_ctx =
            CpiContext::<chain_drive::cpi::accounts::Summon>::new_with_signer(
                ctx.accounts.portal_program.to_account_info(),
                chain_drive::cpi::accounts::Summon {
                    summoner: ctx.accounts.machine.to_account_info(),
                    payer: ctx.accounts.admin.to_account_info(),
                    metadata: ctx.accounts.metadata.to_account_info(),
                    system_program: ctx
                        .accounts
                        .system_program
                        .to_account_info(),
                    portal_config: ctx.accounts.portal_config.to_account_info(),
                    summoner_token_account: ctx
                        .accounts
                        .machine_vault
                        .to_account_info(),
                    shdw_mint: ctx.accounts.shdw_mint.to_account_info(),
                    shdw_vault: ctx.accounts.metadata_vault.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
                signer_seeds,
            );
        chain_drive::cpi::summon(
            cpi_ctx,
            Pubkey::new_from_array(runes.storage_account),
            alice_rune.name.to_string(),
            alice_rune.len as usize,
            alice_rune.hash,
            Some(get_hash_callback(ctx.accounts.metadata.key())),
            Some(0),    // unique clockwork thread id
            20_000_000, // extra lamports
        )?;
        msg!("successfully summoned Alice");

        Ok(())
    }

    pub fn hash_current(ctx: Context<Hash>) -> Result<ThreadResponse> {
        // pub fn hash_current(ctx: Context<Hash>) -> Result<()> {
        // Mark data for deletion after this ix
        ctx.accounts.metadata.callback.take();

        // Zero-copy deser current node
        let current_node: &ArchivedGraphNode = unsafe {
            rkyv::archived_root::<GraphNode>(&ctx.accounts.metadata.data)
        };

        // Update Hash
        let mut hasher =
            sha2::Sha256::new_with_prefix(ctx.accounts.machine.hash);
        ctx.accounts.machine.hash = {
            hasher.update(current_node.name.as_ref());
            let new_hash = hasher.finalize();
            msg!("hashing {}; new hash {:x}", &current_node.name, &new_hash);
            new_hash.try_into().expect("hash is always 32 bytes")
        };
        ctx.accounts.machine.counter += 1;

        // Determine who to summon next
        msg!("next to hash is {}", current_node.next);
        ctx.accounts.machine.next = current_node.next.to_string();

        let machine_vault =
            Pubkey::find_program_address(&[machine().as_ref()], &crate::ID).0;
        let next_metadata = DataToBeSummoned::get_pda(
            &ctx.accounts.machine.key(),
            &Pubkey::new_from_array(unsafe {
                get_runes_unchecked().storage_account
            }),
            &ctx.accounts.machine.next,
            Some(ctx.accounts.machine.counter),
        );
        let metadata_vault = Pubkey::find_program_address(
            &[next_metadata.as_ref()],
            &chain_drive::ID,
        )
        .0;

        Ok(ThreadResponse {
            next_instruction: Some(ClockworkInstructionData {
                program_id: crate::ID,
                accounts: vec![
                    // machine
                    AccountMetaData::new(ctx.accounts.machine.key(), false),
                    // machine vault
                    AccountMetaData::new(machine_vault, false),
                    // next metadata
                    AccountMetaData::new(next_metadata, false),
                    // payer
                    AccountMetaData::new(
                        chain_drive::clockwork_sdk::utils::PAYER_PUBKEY,
                        true,
                    ),
                    // metadata vault
                    AccountMetaData::new(metadata_vault, false),
                    // portal config
                    AccountMetaData::new_readonly(portal_config(), false),
                    // shdw mint
                    AccountMetaData::new_readonly(shdw::ID, false),
                    // token program
                    AccountMetaData::new_readonly(token::ID, false),
                    // portal program
                    AccountMetaData::new_readonly(chain_drive::ID, false),
                    // system program
                    AccountMetaData::new_readonly(system_program::ID, false),
                ],
                data: crate::instruction::SummonNext {}.data(),
            }),
            // summon next when previous is deleted
            // trigger: Some(Trigger::Account {
            //     address: ctx.accounts.metadata.key(),
            //     offset: 0,
            //     size: 16,
            // }),
            trigger: Some(Trigger::Immediate),
        })
    }

    pub fn summon_next(ctx: Context<SummonNext>) -> Result<()> {
        // Get next rune and summon next
        let runes = unsafe { get_runes_unchecked() };
        let next_rune = runes.get_rune(&ctx.accounts.machine.next).unwrap();
        let signer_seeds: &[&[&[u8]]] = &[&[
            "state-machine".as_ref(),
            &[*ctx.bumps.get("machine").unwrap()],
        ]];
        let cpi_ctx =
            CpiContext::<chain_drive::cpi::accounts::Summon>::new_with_signer(
                ctx.accounts.portal_program.to_account_info(),
                chain_drive::cpi::accounts::Summon {
                    summoner: ctx.accounts.machine.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    metadata: ctx.accounts.next.to_account_info(),
                    system_program: ctx
                        .accounts
                        .system_program
                        .to_account_info(),
                    portal_config: ctx.accounts.portal_config.to_account_info(),
                    summoner_token_account: ctx
                        .accounts
                        .machine_vault
                        .to_account_info(),
                    shdw_mint: ctx.accounts.shdw_mint.to_account_info(),
                    shdw_vault: ctx
                        .accounts
                        .next_token_account
                        .to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
                signer_seeds,
            );
        chain_drive::cpi::summon(
            cpi_ctx,
            Pubkey::new_from_array(runes.storage_account),
            next_rune.name.to_string(),
            next_rune.len as usize,
            next_rune.hash,
            Some(get_hash_callback(ctx.accounts.next.key())),
            Some(ctx.accounts.machine.counter), // unique clockwork thread id
            20_000_000,                         // extra lamports
        )?;

        // SOL TO PAYER, so that the thread doesn't need to pay
        **ctx
            .accounts
            .machine
            .to_account_info()
            .try_borrow_mut_lamports()? -= 10_000_000;
        **ctx
            .accounts
            .payer
            .to_account_info()
            .try_borrow_mut_lamports()? += 10_000_000;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    /// CHECK: can only be shdw token because shdw_mint is checked
    pub admin_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = admin,
        space = 100,
        seeds = [
            "state-machine".as_ref()
        ],
        bump,
    )]
    pub machine: Box<Account<'info, Machine>>,

    #[account(
        init,
        payer = admin,
        seeds = [
            machine.key().as_ref()
        ],
        bump,
        token::mint = shdw_mint,
        token::authority = machine,
    )]
    pub machine_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    /// CHECK: checked by shadow portal program
    pub metadata: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked by shadow portal program
    pub metadata_vault: UncheckedAccount<'info>,

    pub portal_config: Account<'info, PortalConfig>,

    #[account(address = shdw::ID)]
    pub shdw_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub portal_program: Program<'info, ChainDrive>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Hash<'info> {
    #[account(mut)]
    pub machine: Account<'info, Machine>,

    #[account(mut)]
    /// CHECK: checked by shadow portal
    pub metadata: Account<'info, DataToBeSummoned>,

    pub portal_program: Program<'info, ChainDrive>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SummonNext<'info> {
    #[account(
        mut,
        seeds = [
            "state-machine".as_ref()
        ],
        bump,
    )]
    pub machine: Account<'info, Machine>,

    #[account(
        mut,
        seeds = [
            machine.key().as_ref()
        ],
        bump,
        token::mint = shdw_mint,
        token::authority = machine,
    )]
    pub machine_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    /// CHECK: checked by shadow portal
    pub next: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    /// CHECK: checked by shadow portal program
    pub next_token_account: UncheckedAccount<'info>,

    pub portal_config: Account<'info, PortalConfig>,

    #[account(address = shdw::ID)]
    pub shdw_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,

    pub portal_program: Program<'info, ChainDrive>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Machine {
    pub counter: u64,
    pub hash: [u8; 32],
    pub admin: Pubkey,
    pub next: String,
}

fn get_hash_callback(metadata: Pubkey) -> ClockworkInstructionData {
    ClockworkInstructionData {
        program_id: crate::ID,
        accounts: vec![
            AccountMetaData::new(crate::machine(), false),
            AccountMetaData::new(metadata, false),
            AccountMetaData::new_readonly(chain_drive::ID, false),
            AccountMetaData::new_readonly(system_program::ID, false),
        ],
        data: crate::instruction::HashCurrent {}.data(),
    }
}

pub fn machine() -> Pubkey {
    Pubkey::find_program_address(&["state-machine".as_ref()], &crate::ID).0
}
