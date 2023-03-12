use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use anchor_spl::token::Token;
use chain_drive::instructions::summon::DataToBeSummoned;
use clockwork_sdk::state::AccountMetaData;
use clockwork_sdk::state::InstructionData as ClockworkInstructionData;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// TODO: write helper function that automatically generates "TEST" vars from local data (automatically uploads to gg,
// retrieves the TEST vars, and summons)

static TEST_HASH: [u8; 32] = [
    209, 188, 141, 59, 164, 175, 199, 225, 9, 97, 44, 183, 58, 203, 221, 218,
    192, 82, 201, 48, 37, 170, 31, 130, 148, 46, 218, 187, 125, 235, 130, 161,
];

static TEST_ACCOUNT: Pubkey = Pubkey::new_from_array([
    59, 253, 10, 18, 239, 63, 40, 166, 47, 100, 57, 4, 43, 249, 250, 182, 166,
    163, 114, 130, 137, 30, 240, 193, 124, 9, 70, 43, 214, 226, 155, 163,
]);

static TEST_FILE: &'static str = "test.txt";
static TEST_LEN: usize = 5;

#[program]
pub mod chain_drive_demo {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let cpi_ctx = CpiContext::<chain_drive::cpi::accounts::Summon>::new(
            ctx.accounts.portal_program.to_account_info(),
            chain_drive::cpi::accounts::Summon {
                summoner: ctx.accounts.summoner.to_account_info(),
                summoner_token_account: ctx
                    .accounts
                    .summoner_token_account
                    .to_account_info(),
                payer: ctx.accounts.summoner.to_account_info(),
                metadata: ctx.accounts.metadata.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                portal_config: ctx.accounts.config.to_account_info(),
                shdw_mint: ctx.accounts.shdw_mint.to_account_info(),
                shdw_vault: ctx.accounts.shdw_vault.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
        );

        let callback = ClockworkInstructionData {
            program_id: crate::ID,
            accounts: vec![AccountMetaData::new_readonly(
                ctx.accounts.metadata.key(),
                false,
            )],
            data: crate::instruction::Print {}.data(),
        };

        msg!("summoning");
        chain_drive::cpi::summon(
            cpi_ctx,
            TEST_ACCOUNT,
            TEST_FILE.to_string(),
            TEST_LEN,
            TEST_HASH,
            Some(callback),
            None,
            0,
        )?;

        Ok(())
    }

    pub fn print(ctx: Context<Print>) -> Result<()> {
        msg!("{}", String::from_utf8_lossy(&ctx.accounts.metadata.data));

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub summoner: Signer<'info>,

    #[account(mut)]
    /// CHECK: Portal Program will do checks
    pub summoner_token_account: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Portal Program will do checks
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Portal Program will do checks
    pub config: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Portal Program will do checks
    pub shdw_vault: UncheckedAccount<'info>,

    /// CHECK: Portal Program will do checks
    pub shdw_mint: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub portal_program: Program<'info, chain_drive::program::ChainDrive>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Print<'info> {
    #[account()]
    pub metadata: Account<'info, DataToBeSummoned>,
}
