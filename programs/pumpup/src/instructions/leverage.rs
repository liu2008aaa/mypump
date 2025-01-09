use crate::{
    ai, consts::{AI_ORACLE_DATA_SEED, CONFIG_SEED, POOL_SOL_SEED}, events::FeedEvent, errors::LeverageError, events::MintLeverageEvent, state::*
};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;


pub fn feed(ctx: Context<FeedOriginalData>, datas: [f64; 33]) -> Result<()> {
    let original_account: &mut Account<'_, AIOriginalDataAccount> = &mut ctx.accounts.original;
    if original_account.to_account_info().data_is_empty() {
        original_account.init();
    }
    require!(!original_account.regulated, LeverageError::MintAlreadyLeverage);
    original_account.feed(&datas)?;
    msg!("feed data successfully.");

    let event = FeedEvent{ 
        period: original_account.period, 
    };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}

pub fn inference(ctx: Context<Inference>, period :u64) -> Result<()> {
    let  original_account = &mut ctx.accounts.original;
    require!(!original_account.regulated && !original_account.period_done && original_account.period  == period, LeverageError::OriginalDataStatusInvalied);
    let regulated = ai::inference(original_account);

    original_account.period_done = true;
    original_account.regulated = regulated;
    if regulated {
        ctx.accounts.pool_sol_account.leverage()?;
    }
    let bonding_curve = &ctx.accounts.pool_sol_account;
    let event = MintLeverageEvent{ 
        mint: ctx.accounts.mint.key(), 
        period: original_account.period, 
        regulated: original_account.regulated, 
        timestamp: get_block_time(), 
        launch_token_surplus: bonding_curve.launch_token_surplus, 
        pool_sol_reserves: bonding_curve.pool_sol_reserves, 
        pool_token_reserves: bonding_curve.pool_token_reserves,
        leverage: bonding_curve.leverage.clone(),
    };
    emit!(event);
    emit_cpi!(event);
    
    Ok(())
}

#[event_cpi]
#[derive(Accounts)]
pub struct FeedOriginalData<'info> {
    #[account(
        seeds = [CONFIG_SEED.as_bytes()],
        bump = config.load()?.bump,
    )]
    pub config: AccountLoader<'info, PumpupConfiguration>,

    #[account()]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed, 
        space = AIOriginalDataAccount::ORIGINAL_INIT_SIZE, 
        payer = migration_address, 
        seeds=[AI_ORACLE_DATA_SEED.as_bytes(), mint.key().as_ref()], 
        bump
    )]
    pub original: Account<'info, AIOriginalDataAccount>,

    /// CHECK: migration_address
    #[account(mut, address =  config.load()?.migration_address, signer)]
    pub migration_address: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct Inference<'info> {
    #[account(
        seeds = [CONFIG_SEED.as_bytes()],
        bump = config.load()?.bump,
    )]
    pub config: AccountLoader<'info, PumpupConfiguration>,

    #[account()]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut, 
        seeds=[AI_ORACLE_DATA_SEED.as_bytes(), mint.key().as_ref()], 
        bump
    )]
    pub original: Account<'info, AIOriginalDataAccount>,

    /// CHECK:
    #[account(
        mut,
        seeds = [POOL_SOL_SEED.as_bytes(), mint.to_account_info().key.as_ref()],
        bump,
    )]
    pub pool_sol_account: Box<Account<'info, BondingCurve>>,

    /// CHECK: sucking_address
    #[account(mut, address =  config.load()?.migration_address, signer)]
    pub migration_address: AccountInfo<'info>,
}