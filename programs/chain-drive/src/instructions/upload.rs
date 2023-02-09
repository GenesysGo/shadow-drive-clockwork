use anchor_lang::prelude::*;
use clockwork_sdk::{state::Thread, ThreadProgram};

use super::summon::DataToBeSummoned;

#[derive(Accounts)]
pub struct Upload<'info> {
    #[account(mut)]
    pub uploader: Signer<'info>,

    #[account(
        mut,
        seeds = [
            metadata.summoner.as_ref(),
            metadata.storage_account.as_ref(),
            metadata.filename.as_ref(),
        ],
        bump,
    )]
    pub metadata: Account<'info, DataToBeSummoned>,

    #[account(
        mut,
        address = Thread::pubkey(metadata.key(), metadata.key().to_bytes().to_vec())
    )]
    pub sdrive_automation: SystemAccount<'info>,

    #[account(address = clockwork_sdk::ThreadProgram::id())]
    pub automation_program: Program<'info, ThreadProgram>,

    pub system_program: Program<'info, System>,
}
