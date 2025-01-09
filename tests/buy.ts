import * as anchor from '@coral-xyz/anchor';
import { Program } from "@coral-xyz/anchor";
import { Pumpup } from "../target/types/pumpup";
import { ComputeBudgetProgram, SystemProgram, Transaction} from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID} from "@solana/spl-token";
import { BN } from "bn.js";
import { getConfigAccount, getPoolSolAccount, getPoolTokenAccount, loadFeeAddress, getTokenAccount, loadMints, getAiTokenBalance, getPoolInfo, getFeeRate, getBuyParamsWithSlippage} from './libs';

describe("pumpup buy", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const pumpuProgram = anchor.workspace.Pumpup as Program<Pumpup>;
  const payer = provider.wallet as anchor.Wallet;
  const feeAddress = loadFeeAddress();
  console.log("feeAddress", feeAddress.publicKey);
  const mint_pubkey = loadMints().pop();
  console.log("mint_pubkey", mint_pubkey);

  it('buy token', async () => {

    //config account
    let config = getConfigAccount(pumpuProgram);
    console.log("configAccount: ", config);
    const [poolSolAccount, bump] = getPoolSolAccount(mint_pubkey, pumpuProgram);
    console.log("poolSolAccount: ", poolSolAccount);
    console.log("pool bump: ", bump);
    const poolTokenAccount = getPoolTokenAccount(mint_pubkey, pumpuProgram);
    console.log("poolTokenAccount: ", poolTokenAccount)
    const userTokenAccount = getTokenAccount(mint_pubkey, payer.publicKey);
    console.log("userTokenAccount: ", userTokenAccount)
    console.log("========= before buy =========")
    let poolInfo = await getPoolInfo(mint_pubkey, pumpuProgram);
    const feeRate = await getFeeRate(pumpuProgram);
    console.log("feeRate: ", feeRate.toString())
    const buyParams = getBuyParamsWithSlippage({
      // amount: new BN(13_000_000_000_000),
      amount: new BN(30_500_000_000_000),
      isSol: false,
      slippage: new BN(2),
      leverage: poolInfo.leverage,
      pumpupFeeRate: feeRate,
      poolSolAmount: poolInfo.poolSolAmount,
      poolTokenAmount: poolInfo.poolTokenAmount,
      launchTokenSurplus: poolInfo.launchTokenSurplus,
      currentLeverageIndex: poolInfo.currentLeverageIndex,
    });
    console.log("buyTokenAmount: ", buyParams.buyTokenAmount.toString())
    console.log("maxSolAmount: ", buyParams.maxSolAmount.toString())

    // 调用 buy 方法
    try {

      const tx = new Transaction()
       .add(
          await pumpuProgram.methods
            .buy(buyParams.buyTokenAmount, buyParams.maxSolAmount)
            .accounts({
              config: config[0],
              user: payer.publicKey,
              mint: mint_pubkey,
              poolTokenAccount: poolTokenAccount,
              poolSolAccount: poolSolAccount,
              pumpupFee: feeAddress.publicKey,
              userTokenAccount: userTokenAccount,
              tokenProgram: TOKEN_PROGRAM_ID,
              associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
            })
            .instruction()
        )
  
      // send tx
      const signature = await provider.sendAndConfirm(tx, [payer.payer]);
      console.log('Transaction signature:', signature);
      console.log("========= after buy =========")
      poolInfo = await getPoolInfo(mint_pubkey, pumpuProgram);
      let aiTokenBalance = await getAiTokenBalance(mint_pubkey, pumpuProgram);
      console.log("aiTokenBalance: " + aiTokenBalance);
     
    } catch (error) {
      console.log('buy with token error: ', error);
    }
   
  })
});
