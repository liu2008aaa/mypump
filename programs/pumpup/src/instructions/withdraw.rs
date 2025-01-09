use std::str::FromStr;

use crate::{
    consts::*,
    events::{TradeEvent, TradeEventType},
    state::*,
    errors::CustomError,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{set_authority, SetAuthority, spl_token::instruction::AuthorityType, transfer_checked, Mint, Token, TokenAccount, TransferChecked},
};

pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    let mint = &ctx.accounts.mint;
    let pumpup_config = ctx.accounts.config.load()?;
    let token_amount = ctx.accounts.pool_token_account.amount;
    require!(token_amount == ctx.accounts.pool_sol_account.launch_token_surplus, CustomError::BondingCurveNotCompletedOrWithdrawCompleted);

    let bump = ctx.bumps.pool_sol_account;
    let mint_pubkey = mint.to_account_info().key();
    let signer: &[&[&[u8]]] = &[&[POOL_SOL_SEED.as_ref(), mint_pubkey.as_ref(), &[bump]]];

    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.pool_token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.to_token_account.to_account_info(),
                authority: ctx.accounts.pool_sol_account.to_account_info(),
            },
            signer,
        ),
        token_amount,
        mint.decimals,
    )?;

    let config_bump = pumpup_config.bump;
    let config_signer: &[&[&[u8]]] = &[&[CONFIG_SEED.as_ref(), &[config_bump]]];


    let black_hole = Pubkey::from_str(BLACK_HOLE).unwrap();
    set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                current_authority: ctx.accounts.config.to_account_info(),
                account_or_mint: ctx.accounts.ai_token_account.to_account_info(),
            },
            config_signer,
        ),
        AuthorityType::AccountOwner,
        Some(black_hole),
    )?;

    let withdrow_lamports = ctx.accounts.pool_sol_account.real_sol;
    let pool_sol_account = ctx.accounts.pool_sol_account.to_account_info();
    let to_sol_account = ctx.accounts.to_sol_account.to_account_info();

    let pumpup_fee_address = ctx.accounts.pumpup_fee.to_account_info();
    **pool_sol_account.try_borrow_mut_lamports()? -= withdrow_lamports;
    **pumpup_fee_address.try_borrow_mut_lamports()? += PUMPUP_MEGRATION_FEE;
    **to_sol_account.try_borrow_mut_lamports()? += withdrow_lamports.saturating_sub(PUMPUP_MEGRATION_FEE);

    ctx.accounts.pool_sol_account.pool_sol_reserves = 0;
    ctx.accounts.pool_sol_account.pool_token_reserves = 0;
    ctx.accounts.pool_sol_account.real_sol = 0;

    let event = TradeEvent {
        mint: mint.key(),
        pumpup_fee: 0,
        sol_amount: withdrow_lamports,
        ai_token_amount:0,
        token_amount,
        event_type: TradeEventType::WITHDRAW,
        user: ctx.accounts.to_sol_account.key(),
        timestamp: get_block_time(),
        pool_real_sol_amount: 0,
        pool_sol_reserves: 0,
        pool_token_reserves: 0,
    };

    emit!(event);
    emit_cpi!(event);

    Ok(())
}

#[event_cpi]
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        seeds = [CONFIG_SEED.as_bytes()],
        bump = config.load()?.bump,
    )]
    pub config: AccountLoader<'info, PumpupConfiguration>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = pool_sol_account,
        associated_token::token_program = token_program,
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK:
    #[account(
        mut,
        seeds = [POOL_SOL_SEED.as_bytes(), mint.key().as_ref()],
        bump
    )]
    pub pool_sol_account: Box<Account<'info, BondingCurve>>,

    #[account(
        init_if_needed,
        payer = to_sol_account,
        associated_token::mint = mint,
        associated_token::authority = to_sol_account,
        associated_token::token_program = token_program,
    )]
    pub to_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = config,
    )]
    pub ai_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK:
    #[account(
        mut,
        address = config.load()?.fee_address
    )]
    pub pumpup_fee: AccountInfo<'info>,

    /// CHECK:
    #[account(
        mut,
        signer,
        address =  config.load()?.migration_address
    )]
    pub to_sol_account: AccountInfo<'info>,
    
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
