use anchor_lang::prelude::*;
use sha2::{Digest, Sha256};

declare_id!("DaWtvgVmLrXKxEN7M7XVrSppsjwN8Pn8MXRiDdcW2a1V");

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
        ctx.accounts.metadata.source = source;
        ctx.accounts.metadata.slot = clock.slot + slot_delay;
        ctx.accounts.metadata.uploader = Pubkey::default();
        ctx.accounts.metadata.summoner = ctx.accounts.summoner.key();
        ctx.accounts.metadata.data = vec![];

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
