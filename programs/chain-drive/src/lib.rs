use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::InstructionData;
use clockwork_sdk::{cpi::ThreadCreate, state::Trigger};
use sha2::{Digest, Sha256};

declare_id!("G6xPudzNNM8CwfLHC9ByzrF67LcwyiRe4t9vHg34eqpR");

pub mod instructions;
use instructions::delete::*;
use instructions::summon::*;
use instructions::upload::*;

pub mod constants;

#[program]
pub mod chain_drive {

    use super::*;

    #[allow(unused)]
    pub fn summon(
        ctx: Context<Summon>,
        source: String,
        data_len: usize,
        slot_delay: u64,
        hash: [u8; 32],
    ) -> Result<()> {
        // Get solana clock
        let clock = Clock::get()?;

        ctx.accounts.metadata.hash = hash;
        ctx.accounts.metadata.source = source.clone();
        ctx.accounts.metadata.slot = clock.slot + slot_delay;
        ctx.accounts.metadata.uploader = Pubkey::default();
        ctx.accounts.metadata.summoner = ctx.accounts.summoner.key();
        ctx.accounts.metadata.data = vec![];

        // AIDAN BARRIER
        let metadata_bump: u8 = *ctx.bumps.get("metadata").unwrap();
        let metadata_seeds: &[&[u8]] = &[
            ctx.accounts.metadata.summoner.as_ref(),
            source.as_ref(),
            &[metadata_bump],
        ];
        let signer_seeds: &[&[&[u8]]] = &[metadata_seeds];

        // ThreadCreate accounts: authority, payer, sys program, thread
        let accounts = ThreadCreate {
            authority: ctx.accounts.metadata.to_account_info(),
            payer: ctx.accounts.summoner.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            thread: ctx.accounts.sdrive_automation.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.automation_program.to_account_info(),
            accounts,
            signer_seeds,
        );
        drop(signer_seeds);

        // Construct kickoff ix
        let upload_ix_data = crate::instruction::Upload { data: vec![] }.data();
        let upload_ix = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(clockwork_sdk::utils::PAYER_PUBKEY, true),
                AccountMeta::new(ctx.accounts.metadata.key(), false),
            ],
            data: upload_ix_data,
        };
        let upload_trigger = Trigger::Immediate;

        clockwork_sdk::cpi::thread_create(
            cpi_ctx,
            source.clone(),
            upload_ix.into(),
            upload_trigger,
        );

        Ok(())
    }

    pub fn upload(ctx: Context<Upload>, data: Vec<u8>) -> Result<()> {
        // TODO: CHECK HASH
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash = hasher.finalize();

        if *hash != ctx.accounts.metadata.hash {
            return Err(PortalError::InvalidHash.into());
        }

        ctx.accounts.metadata.data = data;
        ctx.accounts.metadata.uploader = ctx.accounts.uploader.key();
        Ok(())
    }
    pub fn delete(ctx: Context<Delete>) -> Result<()> {
        // Get solana clock
        let clock = Clock::get()?;

        if clock.slot < ctx.accounts.metadata.slot {
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
