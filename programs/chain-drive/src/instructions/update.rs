use anchor_lang::prelude::*;

use super::init::PortalConfig;

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            "portal-room".as_ref()
        ],
        bump,
    )]
    pub config: Account<'info, PortalConfig>,

    pub system_program: Program<'info, System>,
}
