use anchor_lang::{prelude::*};

pub mod errors;
pub mod instructions;
pub mod state;
pub mod consts;
pub mod events;
pub mod curve;
pub mod ai;

use crate::instructions::*;
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    // Required fields
    name: "Pumpup.ai",
    project_url: "https://pumpup.ai",
    contacts: "link:https://x.com/Pumpup_ai",
    policy: "https://pumpup-ai-1.gitbook.io/pumpup.ai/term-of-use/terms-of-use",

    // Optional Fields
    preferred_languages: "en",
    auditors: "https://www.beosin.com",
    acknowledgements: "
All contracts of Pumpup.ai have been thoroughly audited.
Just have fun!
auditor website: https://www.beosin.com/resources/pumpupai-completes-security-audit-with-beosin-ensure-smart-contract-safety
auditor x: https://x.com/Beosin_com/status/1871843361034027249
auditor binance square: https://www.binance.com/en/square/post/18035791821666
auditor cmc: https://coinmarketcap.com/community/post/348089719
Special thanks to Uswap.ai for their generous support!
"
}

declare_id!("PdMDrKEMaX8q7CCJb7NvUCxerBCcsFUa4LjBEynTtEd");

#[program]
pub mod pumpup {


    use super::*;
    pub fn initialize(ctx: Context<Initialize>, fee_rate: u64) -> Result<()> {
        instructions::initialize(ctx, fee_rate)
    }

    pub fn create(ctx: Context<Create>, name:String, symbol:String, uri:String) -> Result<()> {
        instructions::create(ctx, name, symbol, uri)
    }

    pub fn init_ai_token(ctx: Context<InitAI>)-> Result<()> {
        instructions::init_ai_token(ctx)
    }

    pub fn buy(ctx: Context<Buy>, token_amount: u64, max_sol_amount: u64) -> Result<()> {
        instructions::buy(ctx, token_amount, max_sol_amount)
    }

    pub fn sell(ctx: Context<Sell>, token_amount: u64, min_sol_amount: u64) -> Result<()> {
        instructions::sell(ctx, token_amount, min_sol_amount)
    }

    pub fn feed(ctx: Context<FeedOriginalData>, datas:[f64;33]) -> Result<()> {
        instructions::feed(ctx, datas)
    }

    pub fn inference(ctx: Context<Inference>, period:u64) -> Result<()> {
        instructions::inference(ctx, period)
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        instructions::withdraw(ctx)
    }
    
}
