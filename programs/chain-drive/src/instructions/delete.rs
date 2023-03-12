use super::summon::DataToBeSummoned;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Delete<'info> {
    #[account(mut)]
    pub uploader: SystemAccount<'info>,

    #[account(mut)]
    /// CHECK: must match key in metadata
    pub summoner: AccountInfo<'info>,

    #[account(
        mut,
        close = summoner,
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
}
