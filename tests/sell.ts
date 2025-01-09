import * as anchor from '@coral-xyz/anchor';
import { Program } from "@coral-xyz/anchor";
import { Pumpup } from "../target/types/pumpup";
import { SystemProgram, Transaction} from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID} from "@solana/spl-token";
import { BN } from "bn.js";
import { getConfigAccount, getPoolSolAccount, getPoolTokenAccount, loadFeeAddress, getTokenAccount, loadMints, getPoolInfo, getFeeRate, getSellParamsWithSlippage, getAiTokenAccount } from './libs';


describe("pumpup sell", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const pumpuProgram = anchor.workspace.Pumpup as Program<Pumpup>;
  const payer = provider.wallet as anchor.Wallet;
  const feeAddress = loadFeeAddress();
  const mint_pubkey = loadMints().pop();

  it('sell', async () => {
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
    const aiTokenAccount = getAiTokenAccount(mint_pubkey, pumpuProgram);
    console.log("userTokenAccount: ", userTokenAccount)
    console.log("========= before sell =========")
    let poolInfo = await getPoolInfo(mint_pubkey, pumpuProgram);
    const feeRate = await getFeeRate(pumpuProgram);
    console.log("feeRate: ", feeRate.toString())
    const sellParams = getSellParamsWithSlippage({
      amount: new BN(30_500_000_000_000),
      isSol: false,
      slippage: new BN(2),
      pumpupFeeRate: feeRate,
      poolSolAmount: poolInfo.poolSolAmount,
      poolTokenAmount: poolInfo.poolTokenAmount,
      launchTokenSurplus: poolInfo.launchTokenSurplus,
      currentLeverageIndex: poolInfo.currentLeverageIndex,
      leverage:poolInfo.leverage,
    });
    console.log("sellTokenAmount: ", sellParams.sellTokenAmount.toString())
    console.log("minSolAmount: ", sellParams.minSolAmount.toString())
    // 调用 sell 方法
    try {

      const tx = new Transaction()
        .add(
          await pumpuProgram.methods
            .sell(sellParams.sellTokenAmount, sellParams.minSolAmount)
            .accountsPartial({
              config: config[0],
              user: payer.publicKey,
              mint: mint_pubkey,
              poolTokenAccount: poolTokenAccount,
              poolSolAccount: poolSolAccount,
              aiTokenAccount: aiTokenAccount,
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
      console.log("========= after sell =========")
      poolInfo = await getPoolInfo( mint_pubkey, pumpuProgram);
     
    } catch (error) {
      console.log('sell with token error: ', error);
    }

  });



});
