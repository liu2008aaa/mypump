use crate::{consts::*, state::*, errors::CustomError};
use anchor_lang::system_program;
use anchor_lang::prelude::*;
pub fn initialize(ctx: Context<Initialize>, fee_rate: u64) -> Result<()> {

    require!(fee_rate > 0 && fee_rate < RATE_DENOMINATOR_VALUE, CustomError::InvalidFeeRate);

    let mut config =  ctx.accounts.config.load_init()?;
    let authority_address = ctx.accounts.authority_address.key();
    let fee_address = ctx.accounts.fee_address.key();
    let migration_address = ctx.accounts.migration_address.key();
    let rent = &ctx.accounts.rent;
    config.bump = ctx.bumps.config;

    let account_size = 165;
    let rent_exemption_balance = rent.minimum_balance(account_size);
    let accounts = [
        ctx.accounts.fee_address.to_account_info(),
        ctx.accounts.migration_address.to_account_info(),
    ];
    for account in accounts.iter() {
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.authority_address.to_account_info(),
                    to: account.clone(),
                },
            ),
            rent_exemption_balance,
        )?;
    }
    config.initialize(fee_rate, authority_address, fee_address, migration_address);
    Ok(())
}


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        space = PumpupConfiguration::ACCOUNT_SIZE,
        payer = authority_address,
        seeds = [CONFIG_SEED.as_bytes()],
        bump)]
    pub config: AccountLoader<'info, PumpupConfiguration>,
    /// CHECK
    #[account(mut)]
    pub fee_address: AccountInfo<'info>,

    /// CHECK
    #[account(mut)]
    pub migration_address: AccountInfo<'info>,

    /// CHECK:
    #[account(init, space=165, payer=authority_address, seeds=[MINT_AUTHORITY_SEED.as_bytes()], bump)]
    pub mint_authority: AccountInfo<'info>,

    #[account(mut)]
    pub authority_address: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
