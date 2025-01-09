use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Duplicate config initialize")]
    DuplicateConfigInitialize,

    #[msg("Duplicate tokens are not allowed")]
    DuplicateTokenNotAllowed,

    #[msg("invalid metadata account")]
    InvalidMetadataAccount,

    #[msg("Invalid PDA account")]
    InvalidPda,

    #[msg("Invalid amount: the amount needs to be greater than 0 and less than balance.")]
    InvalidAmount,

    #[msg("Invalid fee rate: the fee rate must range from 0 to 100")]
    InvalidFeeRate,

    #[msg("Invalid AI gas fee: the AI gas fee needs to be greater than 0")]
    InvalidAIGasFee,

    #[msg("Unexpected results of swap token")]
    UnexpectedCalculations,
    
    #[msg("Too little token to buy to complete the launch")]
    TooLittleTokenToBuyLaunch,

    #[msg("slippage: Too much SOL required to buy the given amount of tokens")]
    TooMuchSolSpendToBuyToken,

    #[msg("slippage: Too little SOL received to sell the given amount of tokens")]
    TooLittleSolReceiveToSellToken,

    /// Given pool token amount results in zero trading tokens
    #[msg("Given pool token amount results in zero trading tokens")]
    ZeroTradingTokens,

    #[msg("The bonding curve has completed and liquidity migrated to raydium.")]
    BondingCurveCompleted,

    #[msg("The bonding curve has not completed, or the withdrawal has been completed.")]
    BondingCurveNotCompletedOrWithdrawCompleted,


}

#[error_code]
pub enum LeverageError {
    #[msg("The original data status is invalied.")]
    OriginalDataStatusInvalied,
    
    #[msg("AI leverage adjustments have been completed.")]
    MintAlreadyLeverage,
}