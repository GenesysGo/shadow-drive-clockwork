use anchor_lang::prelude::*;
use clockwork_sdk::{self, state::Thread, ThreadProgram};

use crate::constants::SDRIVE_OBJECT_PREFIX;

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

    #[account(
        address = Thread::pubkey(metadata.key(), source)
    )]
    pub sdrive_automation: SystemAccount<'info>,

    #[account(address = clockwork_sdk::ThreadProgram::id())]
    pub automation_program: Program<'info, ThreadProgram>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Debug)]
pub struct DataToBeSummoned {
    pub source: String,
    pub hash: [u8; 32],
    pub slot: u64,
    pub uploader: Pubkey,
    pub summoner: Pubkey,
    pub data: Vec<u8>,
}

impl DataToBeSummoned {
    pub fn get_source(&self) -> String {
        format!("{SDRIVE_OBJECT_PREFIX}/{}", &self.source)
    }
}
