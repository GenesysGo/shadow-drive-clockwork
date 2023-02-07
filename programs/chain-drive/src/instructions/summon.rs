use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(source: String, data_len: usize)]
pub struct Summon<'info> {
    #[account(mut)]
    pub summoner: Signer<'info>,

    #[account(
        init,
        payer = summoner,
        space = 8 + core::mem::size_of::<DataToBeSummoned>() + source.as_bytes().len() + data_len,
        seeds = [
            summoner.key().as_ref(),
            source.as_ref()
        ],
        bump,
    )]
    pub metadata: Account<'info, DataToBeSummoned>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Debug)]
pub struct DataToBeSummoned {
    pub source: String,
    pub hash: [u8; 32],
    /// TODO: think about u16::MAX, and time units
    pub slot: u64,
    pub uploader: Pubkey,
    pub summoner: Pubkey,
    pub data: Vec<u8>,
}
