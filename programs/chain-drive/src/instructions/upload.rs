use anchor_lang::prelude::*;

use super::summon::DataToBeSummoned;

#[derive(Accounts)]
pub struct Upload<'info> {
    #[account()]
    /// CHECK: anyone can upload
    pub uploader: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            metadata.summoner.as_ref(),
            metadata.source.as_ref()
        ],
        bump,
    )]
    pub metadata: Account<'info, DataToBeSummoned>,
}
