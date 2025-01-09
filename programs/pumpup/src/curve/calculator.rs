//! Swap calculations
use anchor_lang::prelude::*;
use crate::{consts::*, curve::fees::Fees, errors::CustomError};
use std::fmt::Debug;

/// Concrete struct to wrap around the trait object which performs calculation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CurveCalculator {}

impl CurveCalculator {
     
    pub fn buy_token_calculate(
        token_amount: u64,
        pool_sol_amount:u64,
        pool_token_amount: u64,
        pumpup_fee_rate: u64,
        max_sol_amount: u64,
        current_leverage_index: u8,
        leverage: Vec<[u64;3]>,
    ) -> Result<SwapResult> {

        let token_amount_128 = u128::from(token_amount);
        let pool_sol_amount_128 = u128::from(pool_sol_amount);
        let pool_token_amount_128 = u128::from(pool_token_amount);
        let max_sol_amount_128 = u128::from(max_sol_amount);
     
        let mut user_token_amount: u128 = 0;
        let mut user_sol_amount: u128 = 0;
        let mut ai_token_amount: u128 = 0;
        let mut ai_sol_amount: u128 = 0;
        let mut new_pool_sol_amount = pool_sol_amount_128;
        let mut new_pool_token_amount = pool_token_amount_128;
        let mut leverage_index = current_leverage_index;
        let rate_denominator = u128::try_from(RATE_DENOMINATOR_VALUE).unwrap();
       
       let mut swap_token_amount = token_amount_128;
       while user_token_amount < token_amount_128 && swap_token_amount > 0 && leverage_index < leverage.len() as u8 {
            let leverage_info = leverage[leverage_index as usize];
            let leverage_rate = u128::try_from(leverage_info[0]).unwrap();
            let end_token = u128::try_from(leverage_info[2]).unwrap();
            let swap_sol_amount: u128;
            let tradeable_token = new_pool_token_amount.saturating_sub(end_token - 1);

            let mut st = swap_token_amount.saturating_mul(leverage_rate).saturating_div(rate_denominator);
            if tradeable_token <= st{
                st = tradeable_token;   
                leverage_index += 1;
            }

            swap_sol_amount = calculate_swap_amount_out(
                st,
                new_pool_sol_amount,
                new_pool_token_amount,
            );

            let user_token = ceil_div(st.saturating_mul(rate_denominator), leverage_rate);
            let user_sol = ceil_div(swap_sol_amount.saturating_mul(rate_denominator), leverage_rate);

            user_sol_amount += user_sol;
            user_token_amount += user_token;
            ai_sol_amount += swap_sol_amount.saturating_sub(user_sol);
            ai_token_amount += st.saturating_sub(user_token);
            new_pool_sol_amount += swap_sol_amount;
            new_pool_token_amount -= st;
            swap_token_amount = token_amount_128.saturating_sub(user_token_amount);
           
       }
       
        // calculate pumpup trade fees
        let pumpup_fee = Fees::pumpup_fee(user_sol_amount, pumpup_fee_rate).unwrap();
        let swap_sol_amount_with_fee = user_sol_amount.checked_add(pumpup_fee).unwrap();

        if swap_sol_amount_with_fee > max_sol_amount_128 {
            return Err(error!(CustomError::TooMuchSolSpendToBuyToken));
        }

        Ok(SwapResult {
            new_pool_sol_amount: u64::try_from(new_pool_sol_amount)?,
            new_pool_token_amount:u64::try_from(new_pool_token_amount)?,
            user_swap_sol_amount:u64::try_from(user_sol_amount)?,
            user_swap_token_amount:u64::try_from(user_token_amount)?,
            ai_swap_sol_amount:u64::try_from(ai_sol_amount)?,
            ai_swap_token_amount:u64::try_from(ai_token_amount)?,
            pumpup_fee:u64::try_from(pumpup_fee)?,
            leverage_index,
        })
    }


    pub fn sell_token_calculate(
        token_amount: u64,
        pool_sol_amount: u64,
        pool_token_amount: u64,
        pumpup_fee_rate: u64,
        min_sol_amount: u64,
        pool_real_sol: u64,
        current_leverage_index: u8,
        leverage: Vec<[u64;3]>,
    ) -> Result<SwapResult> {
        let token_amount_128 = u128::from(token_amount);
        let pool_real_sol_128 = u128::from(pool_real_sol);
        let min_sol_amount_128 = u128::from(min_sol_amount);
        let mut ai_token_amount: u128 = 0;
        let mut user_token_amount: u128 = 0;
        let mut ai_sol_amount: u128 = 0;
        let mut user_sol_amount: u128 = 0;
        let mut new_pool_sol_amount: u128 = u128::from(pool_sol_amount);
        let mut new_pool_token_amount = u128::from(pool_token_amount);
        let mut leverage_index = current_leverage_index;

        let rate_denominator = u128::from(RATE_DENOMINATOR_VALUE);
        while user_token_amount < token_amount_128 {
            let current_leverage = leverage[leverage_index as usize];
            let leverage_rate = u128::from(current_leverage[0]);
            let start_token = u128::from(current_leverage[1]);

            let mut st = u128::from(token_amount_128.checked_sub(user_token_amount).unwrap());
            st = st.checked_mul(leverage_rate).unwrap().checked_div(rate_denominator).unwrap();
            let tradeable_token = start_token.checked_sub(new_pool_token_amount).unwrap();
            if tradeable_token < st {
                st = tradeable_token;
                leverage_index = leverage_index.saturating_sub(1);
                if tradeable_token == 0 {
                    continue;
                }
            }
            let result = calculate_swap_amount_in(st,new_pool_token_amount , new_pool_sol_amount);
            let user_token = ceil_div(st.checked_mul(rate_denominator).unwrap(),leverage_rate);
            let ai_token = st.checked_sub(user_token).unwrap();
            user_token_amount = user_token_amount.checked_add(user_token).unwrap();
            ai_token_amount = ai_token_amount.checked_add(ai_token).unwrap();
            let user_sol = ceil_div(result.checked_mul(rate_denominator).unwrap(),leverage_rate);
            let ai_sol = result.checked_sub(user_sol).unwrap();
            user_sol_amount = user_sol_amount.checked_add(user_sol).unwrap();
            ai_sol_amount = ai_sol_amount.checked_add(ai_sol).unwrap();
          
            new_pool_sol_amount = new_pool_sol_amount.checked_sub(result).unwrap();
            new_pool_token_amount = new_pool_token_amount.checked_add(st).unwrap();
        }

       
        if user_sol_amount == 0 {
            return Err(error!(CustomError::ZeroTradingTokens));
        } 

        if user_sol_amount > pool_real_sol_128 {
            user_sol_amount = pool_real_sol_128;
        }
        
        if user_sol_amount < min_sol_amount_128 {
            return Err(error!(CustomError::TooLittleSolReceiveToSellToken));
        }
        
        // calculate pumpup trade fees
        let pumpup_fee = Fees::pumpup_fee(user_sol_amount, pumpup_fee_rate).unwrap();
        let swap_sol_amount_less_fee = u64::try_from(user_sol_amount.checked_sub(pumpup_fee).unwrap())?;

        let pumpup_fee_u64 = u64::try_from(pumpup_fee)?;
        let user_token_amount_u64 = u64::try_from(user_token_amount)?;
        let ai_token_amount_u64 = u64::try_from(ai_token_amount)?;
        let ai_sol_amount_u64 = u64::try_from(ai_sol_amount)?;
        let new_pool_sol_amount_u64 = u64::try_from(new_pool_sol_amount)?;
        let new_pool_token_amount_u64 = u64::try_from(new_pool_token_amount)?;

        Ok(SwapResult {
            new_pool_sol_amount: new_pool_sol_amount_u64,
            new_pool_token_amount: new_pool_token_amount_u64,
            user_swap_sol_amount: swap_sol_amount_less_fee,
            user_swap_token_amount: user_token_amount_u64,
            ai_swap_sol_amount: ai_sol_amount_u64,
            ai_swap_token_amount: ai_token_amount_u64,
            pumpup_fee: pumpup_fee_u64,
            leverage_index,
        })
    }

}


pub fn calculate_swap_amount_in(
    source_amount: u128,
    swap_source_amount: u128,
    swap_destination_amount: u128,
) -> u128 {
    // (x + delta_x) * (y - delta_y) = x * y
    // delta_y = (delta_x * y) / (x + delta_x)
    let numerator = source_amount.checked_mul(swap_destination_amount).unwrap();
    let denominator = swap_source_amount.checked_add(source_amount).unwrap();
    let destinsation_amount_swapped = numerator.checked_div(denominator).unwrap();
    destinsation_amount_swapped
}

pub fn calculate_swap_amount_out(
    destinsation_amount: u128,
    swap_source_amount: u128,
    swap_destination_amount: u128,
) -> u128 {
    // (x + delta_x) * (y - delta_y) = x * y
    // delta_x = (x * delta_y) / (y - delta_y)
    let numerator = swap_source_amount.checked_mul(destinsation_amount).unwrap();
    let denominator = swap_destination_amount
        .checked_sub(destinsation_amount)
        .unwrap();
    // round up
    let numerator_up = numerator.checked_add(denominator.checked_sub(1).unwrap()).unwrap();
    let source_amount_swapped = numerator_up.checked_div(denominator).unwrap();
    source_amount_swapped
}
// a / b = (a + b - 1) / b
pub fn ceil_div (a: u128, b: u128) -> u128 {
    a.saturating_add(b.saturating_sub(1)).saturating_div(b)
}

/// Encodes all results of swapping from a source token to a destination token
#[derive(Debug, PartialEq)]
pub struct SwapResult {
    pub new_pool_sol_amount: u64,
    pub new_pool_token_amount: u64,
    pub user_swap_sol_amount: u64,
    pub user_swap_token_amount: u64,
    pub ai_swap_sol_amount: u64,
    pub ai_swap_token_amount: u64,
    pub pumpup_fee: u64,
    pub leverage_index: u8,
}