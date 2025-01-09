use std::ops::Add;

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::Token;
use anchor_spl::token::{TokenAccount, Mint, transfer_checked, TransferChecked};
use crate::curve::CurveCalculator;
use crate::state::{get_block_time, BondingCurve, PumpupConfiguration};
use crate::{consts::*, events::*, errors::CustomError};

pub fn sell(ctx: Context<Sell>, token_amount: u64, min_sol_amount: u64) -> Result<()> {

    require!(token_amount > 0 && min_sol_amount > 0, CustomError::InvalidAmount);

    let mint = ctx.accounts.mint.clone();
    let pumpup_config = ctx.accounts.config.load()?;
    let bonding_curve = &mut ctx.accounts.pool_sol_account;
    let pool_account = bonding_curve.to_account_info();
    let launch_token_surplus = bonding_curve.launch_token_surplus;

    require!(token_amount<=ctx.accounts.user_token_account.amount, CustomError::InvalidAmount);
    require!(ctx.accounts.pool_token_account.amount > launch_token_surplus, CustomError::BondingCurveCompleted);



    let result = CurveCalculator::sell_token_calculate(
        token_amount,
        bonding_curve.pool_sol_reserves, 
        bonding_curve.pool_token_reserves, 
        pumpup_config.fee_rate, 
        min_sol_amount, 
        bonding_curve.real_sol, 
        bonding_curve.current_leverage_index, 
        bonding_curve.leverage.clone())?;

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.user_token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.pool_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            }
        ),
        result.user_swap_token_amount,
        mint.decimals
    )?;

    let config_bump = pumpup_config.bump;
    let config_signer: &[&[&[u8]]] = &[&[CONFIG_SEED.as_ref(), &[config_bump]]];

    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.ai_token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.pool_token_account.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            config_signer,
        ),
        result.ai_swap_token_amount,
        mint.decimals
    )?;

    let pumpup_fee = result.pumpup_fee;
    let user_sol_amount = result.user_swap_sol_amount;

    bonding_curve.pool_sol_reserves= result.new_pool_sol_amount;
    bonding_curve.pool_token_reserves= result.new_pool_token_amount;
    bonding_curve.real_sol = bonding_curve.real_sol - (user_sol_amount + pumpup_fee);
    bonding_curve.virtual_sol -= result.ai_swap_sol_amount;
    bonding_curve.current_leverage_index = result.leverage_index;

    **pool_account.try_borrow_mut_lamports()? -= pumpup_fee;
    **ctx.accounts.pumpup_fee.to_account_info().try_borrow_mut_lamports()? += pumpup_fee;
    
    **pool_account.try_borrow_mut_lamports()? -= user_sol_amount;
    **ctx.accounts.user.try_borrow_mut_lamports()? += user_sol_amount;

    let event = TradeEvent {
        mint: ctx.accounts.mint.key(),
        sol_amount: user_sol_amount.add(pumpup_fee),
        token_amount: result.user_swap_token_amount,
        ai_token_amount: result.ai_swap_token_amount,
        event_type: TradeEventType::SELL,
        user: ctx.accounts.user.key(),
        timestamp: get_block_time(),
        pumpup_fee,
        pool_real_sol_amount: bonding_curve.real_sol,
        pool_sol_reserves: bonding_curve.pool_sol_reserves,
        pool_token_reserves: bonding_curve.pool_token_reserves,
    };

    emit!(event);
    emit_cpi!(event);

    Ok(())
}

#[event_cpi]
#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(
        seeds = [CONFIG_SEED.as_bytes()],
        bump = config.load()?.bump,
    )]
    pub config: AccountLoader<'info, PumpupConfiguration>,

    #[account()]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = pool_sol_account,
    )]
    pub pool_token_account:Box<Account<'info, TokenAccount>>,

    /// CHECK:
    #[account(
        mut,
        seeds = [POOL_SOL_SEED.as_bytes(), mint.to_account_info().key.as_ref()],
        bump,
    )]
    pub pool_sol_account: Box<Account<'info, BondingCurve>>,

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

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
