//! All fee information, to be used for validation currently

use crate::consts::RATE_DENOMINATOR_VALUE;

pub struct Fees {}

/// Helper function for calculating swap fee
pub fn floor_div(token_amount: u128, fee_numerator: u128, fee_denominator: u128) -> Option<u128> {
    Some(
        token_amount
            .checked_mul(fee_numerator)?
            .checked_div(fee_denominator)?,
    )
}

impl Fees {

    /// Calculate the solana trading fee in trading tokens
    pub fn trade_fee(amount: u128, trade_fee_rate: u128) -> Option<u128> {
        floor_div(
            amount,
            u128::from(trade_fee_rate),
            u128::from(RATE_DENOMINATOR_VALUE),
        )
    }

    /// Calculate the pumpup platform trading fee in trading tokens
    pub fn pumpup_fee(amount: u128, pumpup_fee_rate: u64) -> Option<u128> {
        floor_div(
            amount,
            u128::from(pumpup_fee_rate),
            u128::from(RATE_DENOMINATOR_VALUE),
        )
    }

    // max_amount = swap_sol_amount + swap_sol_amount * pumpup_fee_rate/FEE_RATE_DENOMINATOR_VALUE
    // swap_sol_amount = max_amount/(1 + pumpup_fee_rate/FEE_RATE_DENOMINATOR_VALUE)
    // return swap_sol_amount and pumpup_fee
    pub fn pumpup_fee_with_max_pay_amount(max_amount: u128, pumpup_fee_rate: u64) -> (u128, u128) {
        let denminator = u128::from(RATE_DENOMINATOR_VALUE);
        let fee_rate = denminator.checked_add(u128::from(pumpup_fee_rate)).unwrap();
        let swap_sol_amount = max_amount.checked_mul(denminator).unwrap().checked_div(fee_rate).unwrap();
        (swap_sol_amount, max_amount.checked_sub(swap_sol_amount).unwrap())
    }
}