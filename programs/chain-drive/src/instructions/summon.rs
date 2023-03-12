use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{Mint, Token, TokenAccount};
use clockwork_sdk::{self, state::InstructionData as ClockworkInstructionData};

use crate::constants::{shdw, SDRIVE_OBJECT_PREFIX};

use super::init::PortalConfig;


pub(crate) fn handler(
    ctx: Context<Summon>,
    storage_account: Pubkey,
    filename: String,
    data_len: usize,
    hash: [u8; 32],
    callback: Option<ClockworkInstructionData>,
    unique_thread: Option<u64>,
    extra_lamports: u64,
) -> Result<()> {
    ctx.accounts.metadata.hash = hash;
    ctx.accounts.metadata.storage_account = storage_account;
    ctx.accounts.metadata.filename = filename.clone();
    ctx.accounts.metadata.time = i64::MAX;
    ctx.accounts.metadata.uploader = Pubkey::default();
    ctx.accounts.metadata.summoner = ctx.accounts.summoner.key();
    ctx.accounts.metadata.extra_lamports = extra_lamports;
    ctx.accounts.metadata.unique_thread = unique_thread;
    ctx.accounts.metadata.data = vec![];
    ctx.accounts.metadata.callback = callback;

    // Transfer extra lamports
    anchor_lang::system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.metadata.to_account_info(),
            },
        ),
        extra_lamports,
    )?;
    msg!("data is being uploaded to: {}", ctx.accounts.metadata.key());

    // Transfer SHDW to metadata pda
    #[cfg(feature = "verbose")]
    msg!("transfering spl from summoner token account to metadata vault");
    anchor_spl::token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.summoner_token_account.to_account_info(),
                to: ctx.accounts.shdw_vault.to_account_info(),
                authority: ctx.accounts.summoner.to_account_info(),
            },
        ),
        data_len as u64 * ctx.accounts.portal_config.shades_per_byte,
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    storage_account: Pubkey,
    filename: String,
    data_len: usize,
    hash: [u8; 32],
    callback: Option<ClockworkInstructionData>,
    unique_thread: Option<u64>,
)]
pub struct Summon<'info> {
    #[account(mut)]
    /// CHECK: doesn't really matter tbh
    pub summoner: Signer<'info>,

    #[account(
        mut,
        token::mint = shdw_mint,
        token::authority = summoner,
    )]
    pub summoner_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = DataToBeSummoned::space(data_len, &filename, &callback),
        seeds = [
            summoner.key().as_ref(),
            storage_account.as_ref(),
            // this is embarassing
            unique_thread.map(|id| 
                id.to_le_bytes().to_vec()
            ).unwrap_or(<str as AsRef<[u8]>>::as_ref(filename.as_ref()).to_vec()).as_ref(),
        ],
        bump,
    )]
    pub metadata: Box<Account<'info, DataToBeSummoned>>,

    #[account(
        init_if_needed,
        payer = payer,
        seeds = [
            metadata.key().as_ref()
        ],
        bump,
        token::mint = shdw_mint,
        token::authority = metadata,
    )]
    pub shdw_vault: Box<Account<'info, TokenAccount>>,

    #[account(address = shdw::ID)]
    pub shdw_mint: Box<Account<'info, Mint>>,

    /// CHECK: there should only be one config account due to const seeds
    pub portal_config: Account<'info, PortalConfig>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

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
    pub extra_lamports: u64,
    pub unique_thread: Option<u64>,
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
    pub fn get_pda(
        summoner: &Pubkey,
        storage_account: &Pubkey,
        name: &str,
        unique_thread: Option<u64>,
    ) -> Pubkey {
        Pubkey::find_program_address(
            &[
                summoner.as_ref(),
                storage_account.as_ref(),
                // this is embarassing
                unique_thread.map(|id| 
                    id.to_le_bytes().to_vec()
                ).unwrap_or(<str as AsRef<[u8]>>::as_ref(name.as_ref()).to_vec()).as_ref(),
            ],
            &crate::ID,
        )
        .0
    }
    pub fn space(
        data_len: usize,
        name: &str,
        callback: &Option<ClockworkInstructionData>,
    ) -> usize {
        8 + core::mem::size_of::<DataToBeSummoned>()
            + 32
            + 4
            + name.len()
            + data_len
            + 1
            + callback
                .as_ref()
                .map(|cb| {
                    let size = 
                        // program id
                        32 
                        // data
                        + (4 + cb.data.len())
                        // accountmetadata
                        + (4 + 34 * cb.accounts.len());

                    round_up_align(size, 8)
                })
                .unwrap_or(0)
    }
}

fn round_up_align(
    number: usize,
    align: usize
) -> usize {
    number + (number % align)
}

#[test]
fn test_round_up_align() {
    assert_eq!(round_up_align(6, 8), 8);
    assert_eq!(round_up_align(12, 8), 16);
    assert_eq!(round_up_align(16, 8), 16);
    assert_eq!(round_up_align(34, 8), 40);
}