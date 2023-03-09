use anchor_lang::{prelude::*, InstructionData};
use chain_drive::program::ChainDrive;
use graph_demo::*;
use runes::{inscribe_runes, ClockworkInstructionData};
use sha2::Digest;

inscribe_runes!("../nodes.runes");

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod graph_demo_onchain {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let hash_callback = ClockworkInstructionData {
            program_id: crate::ID,
            accounts: vec![/*todo */],
            data: crate::instruction::HashCurrent {}.data(),
        };
        get_runes().summon(
            "Alice",
            ctx.accounts.admin,
            ctx.accounts.metadata,
            ctx.accounts.system_program,
            ctx.accounts.portal_program,
            None,
            Some(hash_callback),
        );
        Ok(())
    }
    pub fn hash_current(ctx: Context<Hash>) -> Result<()> {
        let current_node;

        // TODO: should start hasher with last hash;
        let hasher = sha2::Sha256::new();
        // ctx.hasher.hash = hasher.update(&ctx.accounts.metadata.data)

        // summon next + hash callback

        let hash_callback = ClockworkInstructionData {
            program_id: crate::ID,
            accounts: vec![/*todo */],
            data: crate::instruction::HashCurrent {}.data(),
        };
        get_runes().summon(
            current_node.next.clone(),
            ctx.accounts.admin,
            ctx.accounts.metadata,
            ctx.accounts.system_program,
            ctx.accounts.portal_program,
            None,
            Some(hash_callback),
        );
        Ok(())
    }

    pub fn summon_next(ctx: Context<Hash>) -> Result<()> {
        let hash_callback = ClockworkInstructionData {
            program_id: crate::ID,
            accounts: vec![/*todo */],
            data: crate::instruction::Hash {}.data(),
        };
        get_runes().summon(
            current_node.next.clone(),
            ctx.summoner,
            ctx.metadata,
            ctx.system_program,
            ctx.portal_program,
            None,
            Some(hash_callback),
        );
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

    pub system_program: Program<'info, System>,
    pub portal_program: Program<'info, ChainDrive>,
}

#[derive(Accounts)]
pub struct Hash<'info> {
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

    pub system_program: Program<'info, System>,
    pub portal_program: Program<'info, ChainDrive>,
}

#[account]
pub struct Machine {
    next: String,
    hash: [u8; 32],
    counter: usize,
}
