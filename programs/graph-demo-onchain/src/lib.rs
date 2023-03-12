use anchor_lang::{
    prelude::*,
    solana_program::native_token::LAMPORTS_PER_SOL,
    system_program,
    system_program::{transfer, Transfer},
    InstructionData,
};
use graph_demo::*;
use runes::chain_drive::clockwork_sdk::state::{ThreadResponse, Trigger};
use runes::{
    chain_drive::{
        self, instructions::summon::DataToBeSummoned, AccountMetaData,
    },
    inscribe_runes, ChainDrive, ClockworkInstructionData,
};
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

        // Signer --> Machine
        transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.admin.to_account_info(),
                    to: ctx.accounts.machine.to_account_info(),
                },
            ),
            10 * LAMPORTS_PER_SOL,
        )?;

        let hash_callback = get_hash_callback(ctx.accounts.metadata.key());
        msg!("{:?}", &hash_callback);
        runes.summon(
            "Alice",
            &ctx.accounts.machine,
            &ctx.accounts.admin,
            &ctx.accounts.metadata,
            &ctx.accounts.system_program,
            &ctx.accounts.portal_program,
            None,
            Some(hash_callback),
            20_000_000,
            0,
        );
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

        // Determine who to summon next
        msg!("next to hash is {}", current_node.next);
        ctx.accounts.machine.next = current_node.next.to_string();
        ctx.accounts.machine.counter += 1;

        Ok(ThreadResponse {
            next_instruction: Some(ClockworkInstructionData {
                program_id: crate::ID,
                accounts: vec![
                    AccountMetaData::new(ctx.accounts.machine.key(), false),
                    AccountMetaData::new(
                        DataToBeSummoned::get_pda(
                            &ctx.accounts.machine.key(),
                            &Pubkey::new_from_array(unsafe {
                                get_runes_unchecked().storage_account
                            }),
                            &ctx.accounts.machine.next,
                        ),
                        false,
                    ),
                    AccountMetaData::new(
                        chain_drive::clockwork_sdk::utils::PAYER_PUBKEY,
                        true,
                    ),
                    AccountMetaData::new_readonly(chain_drive::ID, false),
                    AccountMetaData::new_readonly(system_program::ID, false),
                ],
                data: crate::instruction::SummonNext {}.data(),
            }),
            trigger: Some(Trigger::Immediate),
        })
    }

    pub fn summon_next(ctx: Context<SummonNext>) -> Result<()> {
        let runes = unsafe { get_runes_unchecked() };
        let hash_callback = get_hash_callback(ctx.accounts.next.key());

        let signer_seeds: &[&[&[u8]]] = &[&[
            "state-machine".as_ref(),
            &[*ctx.bumps.get("machine").unwrap()],
        ]];

        runes.summon(
            &ctx.accounts.machine.next,
            &ctx.accounts.machine,
            &ctx.accounts.payer,
            &ctx.accounts.next,
            &ctx.accounts.system_program,
            &ctx.accounts.portal_program,
            Some(signer_seeds),
            Some(hash_callback),
            10_000_000,
            ctx.accounts.machine.counter,
        );

        // SOL TO PAYER
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

    #[account(
        init,
        payer = admin,
        space = 100,
        seeds = ["state-machine".as_ref()],
        bump,
    )]
    pub machine: Account<'info, Machine>,

    #[account(mut)]
    /// CHECK: checked by shadow portal
    pub metadata: AccountInfo<'info>,

    // pub thread_program: Program<'info, ThreadProgram>,
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
        seeds = ["state-machine".as_ref()],
        bump,
    )]
    pub machine: Account<'info, Machine>,

    #[account(mut)]
    /// CHECK: checked by shadow portal
    pub next: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub portal_program: Program<'info, ChainDrive>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Machine {
    counter: u64,
    hash: [u8; 32],
    admin: Pubkey,
    next: String,
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
