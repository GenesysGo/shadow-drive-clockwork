use anchor_lang::{prelude::*, Discriminator};
use clockwork_sdk::{
    self,
    state::{InstructionData as ClockworkInstructionData, Thread},
    ThreadProgram,
};

use crate::constants::SDRIVE_OBJECT_PREFIX;

#[derive(Accounts)]
#[instruction(storage_account: Pubkey, filename: String, data_len: usize, hash: [u8; 32])]
pub struct Summon<'info> {
    #[account(mut)]
    pub summoner: Signer<'info>,

    #[account(
        init,
        payer = summoner,
        space = 8 + core::mem::size_of::<DataToBeSummoned>() + 32 + 4 + filename.len() + data_len,
        seeds = [
            summoner.key().as_ref(),
            storage_account.as_ref(),
            filename.as_ref(),
        ],
        bump,
    )]
    pub metadata: Account<'info, DataToBeSummoned>,

    pub system_program: Program<'info, System>,
}

// pub fn make_thread_id(metadata_account_key: &Pubkey) -> String {
//     let mut automation_id = metadata_account_key.to_string();
//     automation_id.truncate(16);
//     automation_id
// }

#[account]
#[derive(Debug)]
pub struct DataToBeSummoned {
    pub storage_account: Pubkey,
    pub filename: String,
    pub hash: [u8; 32],
    pub time: i64,
    pub uploader: Pubkey,
    pub summoner: Pubkey,
    pub uploaded: bool,
    pub callback: Option<ClockworkInstructionData>,
    pub data: Vec<u8>,
}

impl DataToBeSummoned {
    pub fn get_source(&self) -> String {
        Self::build_source(&self.storage_account, &self.filename)
    }
    pub fn build_source(storage_account: &Pubkey, filename: &str) -> String {
        format!("{SDRIVE_OBJECT_PREFIX}/{}/{}", storage_account, filename)
    }
}
