use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use anchor_lang::{
    solana_program::instruction::Instruction, system_program::Transfer,
};
use clockwork_sdk::{cpi::ThreadCreate, state::Trigger};
use sha2::{Digest, Sha256};

pub use clockwork_sdk::{
    self,
    state::{AccountMetaData, InstructionData as ClockworkInstructionData},
};

use crate::constants::TIME_DELAY_SECS;

declare_id!("G6xPudzNNM8CwfLHC9ByzrF67LcwyiRe4t9vHg34eqpR");

pub mod constants;
pub mod instructions;
pub use constants::*;
use instructions::delete::*;
use instructions::init::*;
use instructions::summon::*;
use instructions::update::*;
use instructions::upload::*;

pub use instructions::init::{portal_config, PortalConfig};

#[program]
pub mod chain_drive {

    use super::*;

    #[allow(unused)]
    pub fn summon(
        ctx: Context<Summon>,
        storage_account: Pubkey,
        filename: String,
        data_len: usize,
        hash: [u8; 32],
        callback: Option<ClockworkInstructionData>,
        unique_thread: Option<u64>,
        extra_lamports: u64,
    ) -> Result<()> {
        instructions::summon::handler(
            ctx,
            storage_account,
            filename,
            data_len,
            hash,
            callback,
            unique_thread,
            extra_lamports,
        )
    }

    /// NOTE: this instruction is executed with a worker (clockwork or otherwise)
    /// as a payer. We must redeem all SOL paid out by the worker + their fee.
    pub fn upload(ctx: Context<Upload>, data: Vec<u8>) -> Result<()> {
        msg!(
            "uploader before: {}",
            ctx.accounts
                .uploader
                .to_account_info()
                .try_borrow_mut_lamports()?
        );

        // Check hash
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash = hasher.finalize();
        if *hash != ctx.accounts.metadata.hash {
            return Err(PortalError::InvalidHash.into());
        }

        // Get solana clock, and record slot and uploader
        let clock = Clock::get()?;
        ctx.accounts.metadata.time = clock.unix_timestamp;
        ctx.accounts.metadata.uploader = ctx.accounts.uploader.key();
        ctx.accounts.metadata.data = data;
        ctx.accounts.metadata.uploaded = true;

        // If there is a callback present, initializes the kickoff instructions vector with it
        // This happens way up here due to borrow rules when constructing signer_seeds
        let callback_present = ctx.accounts.metadata.callback.is_some();
        let mut instructions = if callback_present {
            vec![ctx.accounts.metadata.callback.take().unwrap()]
        } else {
            vec![]
        };

        // Transfer SHDW to payout account and close metadata pda
        let metadata_bump: u8 = *ctx.bumps.get("metadata").unwrap();
        let last_seed: Vec<u8> = ctx
            .accounts
            .metadata
            .unique_thread
            .map(|id| id.to_le_bytes().to_vec())
            .unwrap_or(
                <str as AsRef<[u8]>>::as_ref(
                    ctx.accounts.metadata.filename.as_ref(),
                )
                .to_vec(),
            );
        let metadata_seeds: &[&[u8]] = &[
            ctx.accounts.metadata.summoner.as_ref(),
            ctx.accounts.metadata.storage_account.as_ref(),
            last_seed.as_ref(),
            &[metadata_bump],
        ];
        let signer_seeds: &[&[&[u8]]] = &[metadata_seeds];
        #[cfg(feature = "verbose")]
        msg!("transfering portal token pda");
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.metadata_token_account.to_account_info(),
                    to: ctx.accounts.payout_account.to_account_info(),
                    authority: ctx.accounts.metadata.to_account_info(),
                },
                signer_seeds,
            ),
            ctx.accounts.metadata_token_account.amount,
        )?;
        #[cfg(feature = "verbose")]
        msg!("closing portal token pda");
        anchor_spl::token::close_account(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::CloseAccount {
                account: ctx.accounts.metadata_token_account.to_account_info(),
                destination: ctx.accounts.payout_account.to_account_info(),
                authority: ctx.accounts.metadata.to_account_info(),
            },
            signer_seeds,
        ))?;

        // ThreadCreate accounts: authority, payer, sys program, thread
        let accounts = ThreadCreate {
            authority: ctx.accounts.metadata.to_account_info(),
            payer: ctx.accounts.uploader.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            thread: ctx.accounts.sdrive_automation.to_account_info(),
        };
        let cpi_ctx = CpiContext::<ThreadCreate>::new_with_signer(
            ctx.accounts.automation_program.to_account_info(),
            accounts,
            signer_seeds,
        );

        // TODO SOL FROM METADATA TO UPLOADER

        // Construct kickoff ix
        let delete_ix_data = crate::instruction::Delete {}.data();
        let delete_ix = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(clockwork_sdk::utils::PAYER_PUBKEY, true),
                AccountMeta::new(ctx.accounts.metadata.summoner, false),
                AccountMeta::new(ctx.accounts.metadata.key(), false),
            ],
            data: delete_ix_data,
        };
        let clockwork_delete_ix: ClockworkInstructionData = delete_ix.into();
        instructions.push(clockwork_delete_ix);

        // let metadata_key = ctx.accounts.metadata.key().to_bytes().to_vec();

        const SOL_TX_FEE: u64 = 5_000;
        const CW_TX_FEE: u64 = 1_000;
        const DELETE_TX_FEE: u64 = SOL_TX_FEE + CW_TX_FEE;

        #[cfg(feature = "verbose")]
        msg!("creating thread");
        clockwork_sdk::cpi::thread_create(
            cpi_ctx,
            DELETE_TX_FEE + ctx.accounts.metadata.extra_lamports,
            // Vec<u8> id
            ctx.accounts
                .metadata
                .unique_thread
                .map(|id| id.to_le_bytes().to_vec())
                .unwrap_or_else(|| {
                    <str as AsRef<[u8]>>::as_ref(
                        ctx.accounts.metadata.filename.as_ref(),
                    )
                    .to_vec()
                }),
            instructions,
            Trigger::Immediate,
        )?;

        // xfer extra lamports
        // TODO: redeem for all costs
        **ctx
            .accounts
            .metadata
            .to_account_info()
            .try_borrow_mut_lamports()? -= ctx.accounts.metadata.extra_lamports;
        **ctx
            .accounts
            .uploader
            .to_account_info()
            .try_borrow_mut_lamports()? += ctx.accounts.metadata.extra_lamports;
        msg!(
            "thread after: {}",
            ctx.accounts
                .sdrive_automation
                .to_account_info()
                .try_borrow_mut_lamports()?
        );
        msg!(
            "uploader after: {}",
            ctx.accounts
                .uploader
                .to_account_info()
                .try_borrow_mut_lamports()?
        );

        Ok(())
    }

    pub fn delete(ctx: Context<Delete>) -> Result<()> {
        // Get solana clock
        let clock = Clock::get()?;

        if clock.unix_timestamp
            < ctx.accounts.metadata.time.saturating_add(TIME_DELAY_SECS)
            || ctx.accounts.metadata.callback.is_some()
        {
            return Err(PortalError::EarlyDelete.into());
        }
        Ok(())
    }

    pub fn init(ctx: Context<Init>) -> Result<()> {
        msg!("Initializing portal program with {} as admin and with a {} shades per byte fee", ADMIN, INIT_FEE);
        ctx.accounts.config.admin = Pubkey::from_str(ADMIN).unwrap();
        ctx.accounts.config.shades_per_byte = INIT_FEE;

        Ok(())
    }

    pub fn update(ctx: Context<Update>, fee: u64) -> Result<()> {
        msg!("updating fee to {} shades per byte", fee);
        ctx.accounts.config.shades_per_byte = fee;

        Ok(())
    }
}

#[error_code]
pub enum PortalError {
    #[msg("you tried to delete the data too early")]
    EarlyDelete,

    #[msg("you tried to upload data with incorrect hash")]
    InvalidHash,
}

#[test]
#[allow(deprecated)]
fn try_cron_seconds() {
    use chrono::*;
    use clockwork_cron::*;
    use std::str::FromStr;
    for time in (0..1_000_000_000).step_by(10_000_000) {
        for offset in 1..10 {
            let schedule = get_next_n_seconds_schedule(time, offset);
            fn next_timestamp(after: i64, schedule: String) -> Option<i64> {
                Schedule::from_str(&schedule)
                    .unwrap()
                    .next_after(&DateTime::<Utc>::from_utc(
                        NaiveDateTime::from_timestamp(after, 0),
                        Utc,
                    ))
                    .take()
                    .map(|datetime| datetime.timestamp())
            };

            let expected = time + offset;

            assert_eq!(
                expected,
                next_timestamp(time, schedule).unwrap(),
                "failed at time = {time}, offset = {offset}"
            )
        }
    }
}

#[inline(always)]
fn get_next_n_seconds_schedule(unix_timestamp: i64, n_seconds: i64) -> String {
    let later = unix_timestamp + n_seconds;
    let second_place = later % 60;
    let minute_place = (later / 60) % 60;
    format!("{second_place} {minute_place} * * * * *")
}
