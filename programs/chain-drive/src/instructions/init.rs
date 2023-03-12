use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Init<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        space = 8 + ::core::mem::size_of::<PortalConfig>(),
        seeds = [
            "portal-room".as_ref()
        ],
        bump,
        payer = payer,
    )]
    pub config: Account<'info, PortalConfig>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct PortalConfig {
    pub admin: Pubkey,
    pub shades_per_byte: u64,
}

pub fn portal_config() -> Pubkey {
    Pubkey::find_program_address(&["portal-room".as_ref()], &crate::ID).0
}
