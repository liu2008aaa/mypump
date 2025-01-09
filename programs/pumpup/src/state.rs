use std::ops::{Div, Mul, Sub};
use anchor_lang::prelude::*;
use crate::{consts::*, errors::LeverageError};

#[account(zero_copy(unsafe))]
pub struct PumpupConfiguration {
    pub bump: u8,
    pub fee_rate: u64,
    pub authority_address: Pubkey,
    pub fee_address: Pubkey,
    pub migration_address: Pubkey,
}

impl PumpupConfiguration {
    // Discriminator (8) + f64 (8) + Pubkey (32) + Pubkey (32) + Pubkey (32)
    pub const ACCOUNT_SIZE: usize = 8 + 1 + 8 + 32 + 32 + 32;

    pub fn initialize(
        &mut self,
        fee_rate: u64,
        authority_address: Pubkey,
        fee_address: Pubkey,
        migration_address: Pubkey,
    ) {
        self.fee_rate = fee_rate;
        self.authority_address = authority_address;
        self.fee_address = fee_address;
        self.migration_address = migration_address;
    }
}

#[account]
pub struct AIOriginalDataAccount {
    pub bump: u8,
    pub period: u64,
    pub period_done: bool,
    pub regulated: bool,
    pub sol: f64,
    pub datas: [f64; 42],
}

impl AIOriginalDataAccount {
    pub const ORIGINAL_INIT_SIZE: usize = 8 + 1 + 8 + 2 + 8 + 8 * 42;

    pub fn init(&mut self) {
        self.period = 0;
        self.period_done = false;
        self.regulated = false;
        self.sol = 0.0;
        self.datas = [0.0; 42];
    }

    pub fn feed(&mut self, datas: &[f64; 33]) -> Result<()> {
        self.period += 1;
        self.period_done = false;

        let populate_datas = populate_data(&datas);
        let mut normalize_data = [0.0; 42];
        for i in 0..=41 {
            normalize_data[i] = normalize_value(&populate_datas[i]);
        }
        self.sol = datas[2];
        self.datas = normalize_data;
        Ok(())
    }
}

fn populate_data(datas: &[f64; 33]) -> [f64;42]{
    let mut result_data = [0.0; 42];
    result_data[..33].copy_from_slice(datas);
    result_data[33] = datas[4] - datas[5];
    result_data[34] = datas[4] + datas[5];
    result_data[35] = datas[8] + datas[9];
    result_data[36] = 0.0;
    result_data[37] = 0.0;
    result_data[38] = 0.0;
    result_data[39] = 0.0;
    result_data[40] = 0.0;
    result_data[41] = 0.0;
    if datas[5] > 0.0{
        result_data[36] = datas[4].div(datas[5]);
    }else if datas[5]==0.0 && datas[4]> 0.0{
        result_data[36] = datas[4];
    }
    if datas[4]!=0.0 && datas[6]!=0.0 && datas[6]!=1.0{
        result_data[37] = (datas[4].ln()).div(datas[6].ln());
    }
    if datas[5]!=0.0 && datas[7]!=0.0 && datas[7]!=1.0{
        result_data[38] = (datas[5].ln()).div(datas[7].ln());
    }
    if datas[32]!=0.0{
        result_data[39] = datas[31].div(datas[32]).sub(1.0);
    }
    if datas[30]!=0.0{
        result_data[40] = datas[29].div(datas[30]).sub(1.0);
    }else if datas[29]>0.0{
        result_data[40] = datas[29];
    }
    if datas[1]!=0.0{
        result_data[41] = datas[0].div(datas[1]).sub(1.0);
    }else if datas[0]>0.0{
        result_data[41] = datas[0];
    }
    result_data
}


fn normalize_value(value: &f64) -> f64 {
    let mut tanvalue = 0.0;
    if 0.0.ne(value){
        let sign = value.signum();
        let logvalue = value.abs().ln_1p();
        tanvalue = (logvalue/10.0).tanh().mul(sign);
    }
    return (tanvalue+1.0)/2.0;
}

pub fn get_block_time() -> i64 {
    Clock::get()
        .unwrap()
        .unix_timestamp
        .checked_mul(1000)
        .unwrap()
}

#[account]
pub struct BondingCurve {
    pub launch_token_surplus: u64,
    pub virtual_sol: u64,
    pub real_sol: u64,
    pub pool_sol_reserves: u64,
    pub pool_token_reserves: u64,
    pub current_leverage_index: u8,
    //leverage 120 as 1.2, start_token_reserves, end_token_reserves
    pub leverage: Vec<[u64; 3]>,
}

impl BondingCurve {
    pub const SIZE: usize = 8 + 8 + 8 + 8 + 8 + 8 + 1 + 4 + 4 * 24;

    pub fn init(&mut self) {
        self.virtual_sol = INIT_VIRTUAL_SOL_AMOUNT;
        self.launch_token_surplus = POOL_TOKEN_RESERVES_LIMIT;
        self.pool_sol_reserves = INIT_VIRTUAL_SOL_AMOUNT;
        self.pool_token_reserves = INIT_TOKEN_AMOUNT;
        self.real_sol = 0;
        self.leverage = vec![[100, INIT_TOKEN_AMOUNT, POOL_TOKEN_RESERVES_LIMIT + 1]];
        self.current_leverage_index = 0;
    }
    
    pub fn leverage(&mut self) -> Result<()> {
        require!(self.leverage.len() == 1, LeverageError::MintAlreadyLeverage);
        let remaim = self.pool_token_reserves - POOL_TOKEN_RESERVES_LIMIT;
        let x2 = remaim.mul(90).div(100);
        let x3 = remaim.mul(5).div(100);
        let start_3 = self.pool_token_reserves-x2;
        let start_4 = self.pool_token_reserves-x2-x3;
        self.leverage[0] = [100, INIT_TOKEN_AMOUNT, self.pool_token_reserves + 1];
        self.leverage.push([300, self.pool_token_reserves, start_3+1]);
        self.leverage.push([500, start_3, start_4+1]);
        self.leverage.push([700, start_4, POOL_TOKEN_RESERVES_LIMIT + 1]);
        self.current_leverage_index = 1;
        Ok(())
    }
}