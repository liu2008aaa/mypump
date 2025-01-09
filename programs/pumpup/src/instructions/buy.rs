use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::transfer_checked;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::TransferChecked;
use crate::{consts::*, events::*, errors::CustomError, state::*, curve::CurveCalculator};

pub fn buy(ctx: Context<Buy>, token_amount: u64, max_sol_amount: u64) -> Result<()> {

    require!(token_amount > 0 && max_sol_amount > 0, CustomError::InvalidAmount);

    let bonding_curve = &mut ctx.accounts.pool_sol_account;
    let launch_token_surplus = bonding_curve.launch_token_surplus;

    require!(ctx.accounts.pool_token_account.amount > launch_token_surplus, CustomError::BondingCurveCompleted);

    let pumpup_config = ctx.accounts.config.load()?;
    let pool_sol_reserves =  bonding_curve.real_sol.saturating_add(bonding_curve.virtual_sol);

    // calculate swap token and sol amount
    let results = CurveCalculator::buy_token_calculate(
        token_amount, 
        pool_sol_reserves, 
        ctx.accounts.pool_token_account.amount, 
        pumpup_config.fee_rate,
        max_sol_amount,
        bonding_curve.current_leverage_index,
        bonding_curve.leverage.clone())?;

    let supply_token = results.new_pool_token_amount.saturating_sub(launch_token_surplus);
    require!(supply_token == 0 || supply_token > MINIMUM_TOKEN_OF_TRADE, CustomError::TooLittleTokenToBuyLaunch);

    let user_swap_sol_amount = results.user_swap_sol_amount; // exclude fee
    let user_swap_token_amount = results.user_swap_token_amount;
    let ai_swap_token_amount = results.ai_swap_token_amount;
    let pumpup_fee = results.pumpup_fee;
    let new_pool_sol_amount = results.new_pool_sol_amount;
    let new_pool_token_amount = results.new_pool_token_amount;
    let new_virtual_sol = bonding_curve.virtual_sol.saturating_add(results.ai_swap_sol_amount);

    let bump = ctx.bumps.pool_sol_account;
    let mint_key = ctx.accounts.mint.key();
    let signer: &[&[&[u8]]] = &[&[POOL_SOL_SEED.as_ref(),  mint_key.as_ref(), &[bump]]];

    // transfer token to user from pool
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.pool_token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.pool_sol_account.to_account_info(),
            },
            signer
        ),
        user_swap_token_amount,
        ctx.accounts.mint.decimals,
    )?;

    // if ai_swap_token_amount > 0 then transfer token to ai account from pool
    if ai_swap_token_amount > 0 {
        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.pool_token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.ai_token_account.to_account_info(),
                    authority: ctx.accounts.pool_sol_account.to_account_info(),
                },
                signer
            ),
            ai_swap_token_amount,
            ctx.accounts.mint.decimals,
        )?;
    }
    

    if pumpup_fee > 0 {
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.user.to_account_info(),
                    to: ctx.accounts.pumpup_fee.to_account_info(),
                },
            ),
            pumpup_fee,
        )?;
    } else {
        msg!("pumpup fee is 0");
    }
    
    // transfer sol to pool from user
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.pool_sol_account.to_account_info(),
            },
        ),
        user_swap_sol_amount,
    )?;

    ctx.accounts.pool_sol_account.pool_sol_reserves = new_pool_sol_amount;
    ctx.accounts.pool_sol_account.pool_token_reserves = new_pool_token_amount;
    ctx.accounts.pool_sol_account.current_leverage_index = results.leverage_index;
    ctx.accounts.pool_sol_account.real_sol += user_swap_sol_amount;
    ctx.accounts.pool_sol_account.virtual_sol = new_virtual_sol;

    let event = TradeEvent {
        mint: mint_key,
        sol_amount: user_swap_sol_amount,
        token_amount: user_swap_token_amount,
        ai_token_amount:ai_swap_token_amount,
        event_type: TradeEventType::BUY,
        user: ctx.accounts.user.key(),
        timestamp: get_block_time(),
        pumpup_fee: results.pumpup_fee,
        pool_real_sol_amount: ctx.accounts.pool_sol_account.real_sol,
        pool_sol_reserves: new_pool_sol_amount,
        pool_token_reserves: new_pool_token_amount,
    };

    emit!(event);
    emit_cpi!(event);

    Ok(())
}


#[event_cpi]
#[derive(Accounts)]
pub struct Buy<'info> {
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
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

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