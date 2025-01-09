use crate::{consts::*, events::*, state::{get_block_time, BondingCurve, PumpupConfiguration}};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata},
    token::{
        mint_to, set_authority, spl_token::instruction::AuthorityType, Mint, MintTo, SetAuthority, Token, TokenAccount
    },
};
use mpl_token_metadata::accounts::Metadata as MetaDataAccount;
use mpl_token_metadata::types::DataV2;

pub fn create(ctx: Context<Create>, name: String, symbol: String, uri: String) -> Result<()> {
    let token_metadata = DataV2 {
        name: name.clone(),
        symbol: symbol.clone(),
        uri: uri.clone(),
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };

    let mint_auth_seeds: &[u8] = MINT_AUTHORITY_SEED.as_bytes();
    let bump = ctx.bumps.mint_authority;
    let authority_signer: &[&[&[u8]]] = &[&[mint_auth_seeds, &[bump]]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_metadata_program.to_account_info(),
        CreateMetadataAccountsV3 {
            metadata: ctx.accounts.metadata_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            mint_authority: ctx.accounts.mint_authority.to_account_info(),
            update_authority: ctx.accounts.creator.to_account_info(),
            payer: ctx.accounts.creator.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        },
        authority_signer,
    );

    create_metadata_accounts_v3(cpi_ctx, token_metadata, true, true, None)?;

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                authority: ctx.accounts.mint_authority.to_account_info(),
                to: ctx.accounts.pool_token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
            authority_signer,
        ),
        INIT_TOKEN_AMOUNT,
    )?;

    set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                current_authority: ctx.accounts.mint_authority.to_account_info(),
                account_or_mint: ctx.accounts.mint.to_account_info(),
            },
            authority_signer,
        ),
        AuthorityType::MintTokens,
        None,
    )?;

    ctx.accounts.pool_sol_account.init();
    let bonding_curve = &ctx.accounts.pool_sol_account;

    let event = CreateMintEvent {
        mint: ctx.accounts.mint.key(),
        name: name.clone(),
        symbol: symbol.clone(),
        uri: uri.clone(),
        pool_sol_address: bonding_curve.key(),
        pool_token_address: ctx.accounts.pool_token_account.key(),
        user: ctx.accounts.creator.key(),
        timestamp: get_block_time(),
        launch_token_surplus: bonding_curve.launch_token_surplus,
        pool_sol_reserves: bonding_curve.pool_sol_reserves,
        pool_token_reserves:bonding_curve.pool_token_reserves,
    };

    emit!(event);
    emit_cpi!(event);

    Ok(())
}


pub fn init_ai_token(_ctx: Context<InitAI>) -> Result<()> {
    Ok(())
}


#[event_cpi]
#[derive(Accounts)]
pub struct Create<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    /// CHECK:
    #[account(
        init,
        payer = creator,
        mint::decimals = 6,
        mint::authority = mint_authority,
    )]
    pub mint: Box<Account<'info, Mint>>,

    /// CHECK:
    #[account(mut, seeds=[MINT_AUTHORITY_SEED.as_bytes()], bump)]
    pub mint_authority: AccountInfo<'info>,

    /// CHECK:
    #[account(
        init,
        payer = creator,
        seeds = [POOL_SOL_SEED.as_bytes(), mint.key().as_ref()],
        bump,
        space = BondingCurve::SIZE,
    )]
    pub pool_sol_account: Box<Account<'info, BondingCurve>>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = pool_sol_account,
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK:
    #[account(
        mut,
        address = MetaDataAccount::find_pda(&mint.key()).0
    )]
    pub metadata_account: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct InitAI<'info> {

    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED.as_bytes()],
        bump = config.load()?.bump,
    )]
    pub config: AccountLoader<'info, PumpupConfiguration>,

    #[account()]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = config,
    )]
    pub ai_token_account: Box<Account<'info, TokenAccount>>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}