use super::summon::DataToBeSummoned;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Delete<'info> {
    #[account()]
    pub uploader: SystemAccount<'info>,

    #[account()]
    pub summoner: SystemAccount<'info>,

    #[account(
        mut,
        close = summoner,
        seeds = [
            metadata.summoner.as_ref(),
            metadata.source.as_ref()
        ],
        bump,
    )]
    pub metadata: Account<'info, DataToBeSummoned>,
}
