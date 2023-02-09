use super::summon::DataToBeSummoned;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Delete<'info> {
    #[account(mut)]
    pub uploader: SystemAccount<'info>,

    #[account(mut)]
    pub summoner: SystemAccount<'info>,

    #[account(
        mut,
        close = summoner,
        seeds = [
            metadata.summoner.as_ref(),
            metadata.storage_account.as_ref(),
            metadata.filename.as_ref(),
        ],
        bump,
    )]
    pub metadata: Account<'info, DataToBeSummoned>,
}
