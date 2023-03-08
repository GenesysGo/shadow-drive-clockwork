use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::InstructionData;
use clockwork_sdk::{cpi::ThreadCreate, state::Trigger};
use sha2::{Digest, Sha256};

pub use clockwork_sdk::state::InstructionData as ClockworkInstructionData;

declare_id!("G6xPudzNNM8CwfLHC9ByzrF67LcwyiRe4t9vHg34eqpR");

pub mod instructions;
use instructions::delete::*;
use instructions::summon::*;
use instructions::upload::*;

pub mod constants;

#[program]
pub mod chain_drive {

    use crate::constants::TIME_DELAY_SECS;

    use super::*;

    #[allow(unused)]
    pub fn summon(
        ctx: Context<Summon>,
        storage_account: Pubkey,
        filename: String,
        data_len: usize,
        hash: [u8; 32],
        callback: Option<ClockworkInstructionData>,
    ) -> Result<()> {
        // Get solana clock
        let clock = Clock::get()?;

        ctx.accounts.metadata.hash = hash;
        ctx.accounts.metadata.storage_account = storage_account;
        ctx.accounts.metadata.filename = filename.clone();
        ctx.accounts.metadata.time = i64::MAX;
        ctx.accounts.metadata.uploader = Pubkey::default();
        ctx.accounts.metadata.summoner = ctx.accounts.summoner.key();
        ctx.accounts.metadata.data = vec![];
        ctx.accounts.metadata.callback = callback;

        // transfer SHDW to GG wallet
        // fn(data_len * 2)

        Ok(())
    }

    pub fn upload(ctx: Context<Upload>, data: Vec<u8>) -> Result<()> {
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

        let callback_present = ctx.accounts.metadata.callback.is_some();
        let mut instructions = if callback_present {
            vec![ctx.accounts.metadata.callback.take().unwrap()]
        } else {
            vec![]
        };

        let metadata_bump: u8 = *ctx.bumps.get("metadata").unwrap();
        let metadata_seeds: &[&[u8]] = &[
            ctx.accounts.metadata.summoner.as_ref(),
            ctx.accounts.metadata.storage_account.as_ref(),
            ctx.accounts.metadata.filename.as_ref(),
            &[metadata_bump],
        ];
        let signer_seeds: &[&[&[u8]]] = &[metadata_seeds];

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
        drop(signer_seeds);

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

        let delete_trigger = Trigger::Cron {
            schedule: get_next_n_seconds_schedule(clock.unix_timestamp, TIME_DELAY_SECS),
            skippable: false,
        };

        let metadata_key = ctx.accounts.metadata.key().to_bytes().to_vec();

        const SOL_TX_FEE: u64 = 5_000;
        const CW_TX_FEE: u64 = 1_000;
        const DELETE_TX_FEE: u64 = SOL_TX_FEE + CW_TX_FEE;

        clockwork_sdk::cpi::thread_create(
            cpi_ctx,
            DELETE_TX_FEE,
            metadata_key,
            instructions,
            delete_trigger,
        )?;

        Ok(())
    }

    pub fn delete(ctx: Context<Delete>) -> Result<()> {
        // Get solana clock
        let clock = Clock::get()?;

        if clock.unix_timestamp < ctx.accounts.metadata.time.saturating_add(TIME_DELAY_SECS) {
            return Err(PortalError::EarlyDelete.into());
        }
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
