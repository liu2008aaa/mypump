use anchor_lang::prelude::*;

#[event]
pub struct TradeEvent {
    pub mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub ai_token_amount: u64,
    pub event_type: TradeEventType,
    pub user: Pubkey,
    pub timestamp: i64,
    pub pumpup_fee: u64,
    pub pool_real_sol_amount: u64,
    pub pool_sol_reserves: u64,
    pub pool_token_reserves: u64
}

#[event]
pub struct CreateMintEvent {
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub pool_sol_address: Pubkey,
    pub pool_token_address: Pubkey,
    pub user: Pubkey,
    pub timestamp: i64,
    pub launch_token_surplus: u64,
    pub pool_sol_reserves: u64,
    pub pool_token_reserves: u64
}

#[event]
pub struct MintLeverageEvent {
    pub mint: Pubkey,
    pub period: u64,
    pub regulated: bool,
    pub timestamp: i64,
    pub launch_token_surplus: u64,
    pub pool_sol_reserves: u64,
    pub pool_token_reserves: u64,
    pub leverage: Vec<[u64; 3]>,
}

#[event]
pub struct FeedEvent {
    pub period: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub enum TradeEventType {

    BUY, 

    SELL, 

    WITHDRAW, 
    
}
