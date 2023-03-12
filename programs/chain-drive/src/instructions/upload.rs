use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use clockwork_sdk::{state::Thread, ThreadProgram};

use super::summon::DataToBeSummoned;

#[derive(Accounts)]
pub struct Upload<'info> {
    #[account(mut)]
    /// CHECK: Anyone willing to upload
    pub uploader: Signer<'info>,

    #[account(
        mut,
        seeds = [
            metadata.summoner.key().as_ref(),
            metadata.storage_account.as_ref(),
            // this is embarassing
            metadata.unique_thread.map(|id| 
                id.to_le_bytes().to_vec()
            ).unwrap_or(<str as AsRef<[u8]>>::as_ref(metadata.filename.as_ref()).to_vec()).as_ref(),
        ],
        bump,
    )]
    pub metadata: Account<'info, DataToBeSummoned>,

    #[account(
        mut,
        seeds = [
            metadata.key().as_ref()
        ],
        bump,
    )]
    pub metadata_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::authority = crate::constants::payout_authority::ID,
    )]
    pub payout_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = Thread::pubkey(
            metadata.key(),
            metadata.unique_thread.map(|id| id.to_le_bytes().to_vec())
                .unwrap_or_else(|| <str as AsRef<[u8]>>::as_ref(metadata.filename.as_ref()).to_vec())
        )
    )]
    pub sdrive_automation: SystemAccount<'info>,

    #[account(address = clockwork_sdk::ThreadProgram::id())]
    pub automation_program: Program<'info, ThreadProgram>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
