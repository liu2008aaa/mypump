import * as anchor from '@coral-xyz/anchor';
import { Pumpup } from "../target/types/pumpup";
import { Connection, PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import {getAccount, ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync } from "@solana/spl-token";
import * as fs from 'fs';
import BN from 'bn.js';

const DENOMINATOR_VALUE = new BN(100);
const MPL_TOKEN_METADATA_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

export function loadKeypair(filename: string): Keypair {
    const secretKeyString = fs.readFileSync(filename, 'utf8');
    const secretKeyArray = Uint8Array.from(JSON.parse(secretKeyString));
    return Keypair.fromSecretKey(secretKeyArray);
}

export function loadFeeAddress() {
    return loadKeypair("./keys/feeAddress.json");
}

export function loadMigrationAddress() {
    return loadKeypair("./keys/migrationAddress.json");
}

export function appendedMintPubkey(mint: PublicKey) {
    fs.appendFileSync('./keys/mint.txt', mint.toBase58() + "\n", 'utf8');
}

export function loadMints():PublicKey[] {
    try {
        const data = fs.readFileSync('./keys/mint.txt', 'utf8');
        const mints = data
            .split(/\r?\n/)
            .filter(line => line.trim() !== "")
            .map(line => new PublicKey(line.trim()));

        return mints;
    } catch (err) {
        console.error("Error reading file:", err);
        return [];
    }
}

export function getConfigAccount(program: anchor.Program<Pumpup>) {
    let configurationAccount = PublicKey.findProgramAddressSync(
        [Buffer.from('pumpup.config')],
        program.programId);
    return configurationAccount;
}
export function getMintAuthAccount(program: anchor.Program<Pumpup>) {
    let mintAuthAccount = PublicKey.findProgramAddressSync(
        [Buffer.from('pumpup.mint_authority')],
        program.programId);
    return mintAuthAccount;
}

export function getMetadataAccount(mint: PublicKey) {
    let metadataAccount = PublicKey.findProgramAddressSync(
        [Buffer.from('metadata'), MPL_TOKEN_METADATA_ID.toBuffer(), mint.toBuffer()],
        MPL_TOKEN_METADATA_ID);
    return metadataAccount;
}
export function getOriginalAccount(mint: PublicKey, program: anchor.Program<Pumpup>) {
    let originalAccount = PublicKey.findProgramAddressSync(
        [Buffer.from('pumpup.original.data'), mint.toBuffer()],
        program.programId
    );
    return originalAccount;
}

export function getPoolSolAccount(mint: PublicKey, program: anchor.Program<Pumpup>) {
    let poolSolAccount =  PublicKey.findProgramAddressSync(
        [Buffer.from("pumpup.pool"), mint.toBuffer()],
        program.programId,
    );
    return poolSolAccount;
}

export function getPoolTokenAccount(mint: PublicKey, program: anchor.Program<Pumpup>) {
    let poolSolAccount = getPoolSolAccount(mint, program);
    const poolTokenAccount = getAssociatedTokenAddressSync(
        mint,
        poolSolAccount[0],
        true,
        TOKEN_PROGRAM_ID,
    );
    return poolTokenAccount;
}

export function getTokenAccount(mint: PublicKey, userPubKey: PublicKey) {
    const tokenAccount = getAssociatedTokenAddressSync(
        mint,
        userPubKey,
        false,
        TOKEN_PROGRAM_ID,
    );
    return tokenAccount;
}

export function getAiTokenAccount(mint: PublicKey, program: anchor.Program<Pumpup>){
   let configAccount = getConfigAccount(program);
   let aiTokenAccount = getAssociatedTokenAddressSync(
        mint,
        configAccount[0],
        true,
        TOKEN_PROGRAM_ID,
    );
    return aiTokenAccount;
}

export async function getAiTokenBalance(mint: PublicKey, program: anchor.Program<Pumpup>){
   
    let configAccount = getConfigAccount(program);
    let aiTokenAccount = getAssociatedTokenAddressSync(
         mint,
         configAccount[0],
         true,
         TOKEN_PROGRAM_ID,
     );
     let tokenAccountInfo = await getAccount(program.provider.connection, aiTokenAccount);
     return tokenAccountInfo.amount.toString();
 }


export async function buildFeedDataInstruction(mint: PublicKey, program: anchor.Program<Pumpup>, input: number[]) {
    let configurationAccount = getConfigAccount(program);
    let originalPDA = getOriginalAccount(mint, program);
    let migrationAddress = loadMigrationAddress();
    
    let feedDataInstruction = await program.methods
        .feed(input)
        .accountsPartial({
            config: configurationAccount[0],
            original: originalPDA[0],
            mint: mint,
            migrationAddress: migrationAddress.publicKey,
        })
        .instruction();
    return feedDataInstruction;
}

export async function buildInferenceInstruction(mint: PublicKey, period:BN, program: anchor.Program<Pumpup>) {
    let configurationAccount = getConfigAccount(program);
    let poolSolAccount = getPoolSolAccount(mint, program);
    let migrationAddress = loadMigrationAddress();
    let originalAccount = getOriginalAccount(mint,program);
    
    let feedDataInstruction = await program.methods
        .inference(period)
        .accountsPartial({
            config: configurationAccount[0],
            mint: mint,
            original:originalAccount[0],
            poolSolAccount:poolSolAccount[0],
            migrationAddress: migrationAddress.publicKey,
        })
        .instruction();
    return feedDataInstruction;
}

export async function buildLeverageInstruction(mint: PublicKey, program: anchor.Program<Pumpup>) {
    let configurationAccount = getConfigAccount(program);
    let poolSolAccount = getPoolSolAccount(mint, program);
    let migrationAddress = loadMigrationAddress();
    
    let feedDataInstruction = await program.methods
        .leverage()
        .accountsPartial({
            config: configurationAccount[0],
            mint: mint,
            poolSolAccount:poolSolAccount[0],
            migrationAddress: migrationAddress.publicKey,
        })
        .instruction();
    return feedDataInstruction;
}

export async function buildWithdrawInstruction(mint: PublicKey, program: anchor.Program<Pumpup>) {
    let migrationAddress = loadMigrationAddress();
    let feeAddress = loadFeeAddress();
    let configurationAccount = getConfigAccount(program);
    let poolSolAccount = getPoolSolAccount(mint, program);
    let poolTokenAccount = getPoolTokenAccount(mint, program);
    let migrationTokenAccount = getTokenAccount(mint, migrationAddress.publicKey);
    let aiTokenAccount = getAiTokenAccount(mint, program);
    console.log('aiTokenAccount :{}', aiTokenAccount);

    const withdraw = await program.methods
        .withdraw()
        .accountsPartial({
            config: configurationAccount[0],
            mint: mint,
            poolSolAccount: poolSolAccount[0],
            poolTokenAccount: poolTokenAccount,
            toTokenAccount: migrationTokenAccount[0],
            aiTokenAccount: aiTokenAccount,
            pumpupFee: feeAddress.publicKey,
            toSolAccount: migrationAddress.publicKey,
            rent: SYSVAR_RENT_PUBKEY,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
        })
        .instruction();
    return withdraw;
}


export async function getPoolInfo(mint: PublicKey, program: anchor.Program<Pumpup>): Promise<PoolInfo> {
    const [poolSolAccount, bump] = getPoolSolAccount(mint, program);
    try {
        const bondingCurve = await program.account.bondingCurve.fetch(poolSolAccount);
        const k =  new BN(bondingCurve.poolSolReserves).mul(new BN(bondingCurve.poolTokenReserves));
        console.log(`launchTokenSurplus: ${bondingCurve.launchTokenSurplus}, realSol: ${bondingCurve.realSol}, virtualSol: ${bondingCurve.virtualSol}, poolSolAmount: ${bondingCurve.poolSolReserves}, poolTokenAmount: ${bondingCurve.poolTokenReserves}`);
        const leverage: [BN, BN, BN, BN, BN][] = bondingCurve.leverage.map((item: BN[]) => {
            if (item.length === 3) {
                const start_sol = k.div(item[1]);
                const end_sol = k.div(item[2]);
                return [item[0], item[1], item[2], start_sol, end_sol] as [BN, BN, BN, BN, BN]; // Make sure each inner array has 3 elements
            } else {
                throw new Error("Invalid leverage array length, each item must have exactly 3 elements.");
            }
        });
        leverage.forEach(le => {
            console.log(`leverage: ${le}`)
        });
        console.log(`current leverage index: ${bondingCurve.currentLeverageIndex}`);
        return {
            realSol: new BN(bondingCurve.realSol),
            virtualSol: new BN(bondingCurve.virtualSol),
            launchTokenSurplus: new BN(bondingCurve.launchTokenSurplus),
            poolSolAmount: new BN(bondingCurve.poolSolReserves),
            poolTokenAmount: new BN(bondingCurve.poolTokenReserves),
            currentLeverageIndex: bondingCurve.currentLeverageIndex,
            leverage
        };
    } catch (error) {
      console.error(error);
      throw new Error('Error fetching pool info data')
    }

}


export async function getFeeRate(program: anchor.Program<Pumpup>): Promise<BN> {
    let configPublicKey = getConfigAccount(program);
    try {
        // 获取当前区块的区块哈希
        const config_data = await program.account.pumpupConfiguration.fetch(configPublicKey[0]);
        return new BN(config_data.feeRate);
    } catch (error) {
      console.error(error);
      throw new Error('Error fetching fee reate');
    }
}


export function getBuyParamsWithSlippage(params: PayParams): BuyWithSlippage {

    // maxSolAmount = amount * (1 + slipage/100) * (1 + feeRate/100)
    let maxSolAmount: BN;
    let swapTokenAmount: BN;
    if (params.isSol) {
        swapTokenAmount = getSwapAmountWithLeverage(params.amount, params.isSol, params.leverage, params.currentLeverageIndex, params.poolSolAmount, params.poolTokenAmount);
        maxSolAmount = params.amount.mul(params.slippage.add(DENOMINATOR_VALUE)).div(DENOMINATOR_VALUE)
                        .mul(params.pumpupFeeRate.add(DENOMINATOR_VALUE)).div(DENOMINATOR_VALUE);
    } else {
        swapTokenAmount = params.amount; 
        let swapSolAmount = getSwapAmountWithLeverage(params.amount, params.isSol,params.leverage, params.currentLeverageIndex, params.poolSolAmount, params.poolTokenAmount);
        maxSolAmount = swapSolAmount.mul(params.slippage.add(DENOMINATOR_VALUE)).div(DENOMINATOR_VALUE)
        .mul(params.pumpupFeeRate.add(DENOMINATOR_VALUE)).div(DENOMINATOR_VALUE);
    }

    return {
        buyTokenAmount: swapTokenAmount,
        maxSolAmount: maxSolAmount,
    }

}


function getSwapAmountWithLeverage(amount: BN, isSol: boolean, leverage: [BN, BN, BN, BN, BN][], currentLeverageIndex: number, poolSolAmount: BN, poolTokenAmount: BN): BN{
    let zero = new BN(0);
    let userFromAmount = new BN(0);
    let userToAmount = new BN(0);
    let newPoolSolAmount = poolSolAmount;
    let newPoolTokenAmount = poolTokenAmount;
    let leverageIndex = currentLeverageIndex;

    let swapFromAmount = amount;
    while (userFromAmount.lt(amount) && swapFromAmount.gt(zero) && leverageIndex < leverage.length) {
        let leverageInfo = leverage[leverageIndex];
        let leverageRate = new BN(leverageInfo[0]);
        let swapToAmount = new BN(0);
        let tradeable: BN;
        if (isSol) {
            let leverageRangeEnd = new BN(leverageInfo[4]);
            tradeable = leverageRangeEnd.sub(newPoolSolAmount.sub(new BN(1)));
        } else {
            let leverageRangeEnd = new BN(leverageInfo[2]);
            tradeable = newPoolTokenAmount.sub(leverageRangeEnd.sub(new BN(1)));
        }

        let st = swapFromAmount.mul(leverageRate).div(DENOMINATOR_VALUE);
        if (tradeable.lte(st)) {
            console.log("leverageSupply: " + tradeable);
            st = tradeable;
            leverageIndex += 1;
        } 

        if (isSol) {
            swapToAmount = calculate_swap_amount_in(st, newPoolSolAmount, newPoolTokenAmount);
        } else {
            swapToAmount = calculate_swap_amount_out(st, newPoolSolAmount, newPoolTokenAmount);
        }
        console.log("leverage amount: " + st + ", swap to target amount: " + swapToAmount);
        let userSwapFromAmount = ceil_div(st.mul(DENOMINATOR_VALUE), leverageRate);
        let userSwapToAmount = ceil_div(swapToAmount.mul(DENOMINATOR_VALUE), leverageRate);
        console.log("user amount: " + userSwapFromAmount + ", swap to target amount: " + userSwapToAmount);

        userToAmount = userToAmount.add(userSwapToAmount);
        userFromAmount = userFromAmount.add(userSwapFromAmount);
        if (isSol) {
            newPoolSolAmount = newPoolSolAmount.add(st);
            newPoolTokenAmount = newPoolTokenAmount.sub(swapToAmount);
        } else {
            newPoolSolAmount = newPoolSolAmount.add(swapToAmount);
            newPoolTokenAmount = newPoolTokenAmount.sub(st);
        }
        swapFromAmount = amount.sub(userFromAmount);
    }

    return userToAmount;

}

function ceil_div(a: BN, b: BN): BN{
    return a.add(b.sub(new BN(1))).div(b);
}


export function getSellParamsWithSlippage(params: PayParams): SellWithSlippage {
     // maxSolAmount = amount * (1 - slipage/100)
    let swapTokenAmount = params.amount;
    let swap_sol_amount = sellTokenCalculateSol(swapTokenAmount, params.poolSolAmount, params.poolTokenAmount, params.currentLeverageIndex, params.leverage);
    let minSolAmount = swap_sol_amount.mul(DENOMINATOR_VALUE.sub(params.slippage)).div(DENOMINATOR_VALUE);

    return {
        sellTokenAmount: swapTokenAmount,
        minSolAmount: minSolAmount,
    }

}

function sellTokenCalculateSol(
    tokenAmount: BN,
    poolSolAmount: BN,
    poolTokenAmount: BN,
    currentLeverageIndex: number,
    leverage: [BN, BN, BN, BN, BN][],
): BN {
    let aiTokenAmount = new BN(0);
    let userTokenAmount = new BN(0);
    let aiSolAmount = new BN(0);
    let userSolAmount = new BN(0);
    let newPoolSolAmount = poolSolAmount;
    let newPoolTokenAmount = poolTokenAmount;
    let index = currentLeverageIndex;

    while (userTokenAmount.lt(tokenAmount) && index >= 0) {
        console.log(`user token amount ${userTokenAmount}`)
        const currentLeverage = leverage[index];
        const leverageRate = currentLeverage[0];
        const startToken = currentLeverage[1];
        console.log(`currentLeverage ${currentLeverage}`)
        let st = tokenAmount.sub(userTokenAmount);
        st = st.mul(leverageRate).div(DENOMINATOR_VALUE)
        console.log(`tokenAmount ${tokenAmount}, st ${st}`)
        let tradeableToken = startToken.sub(newPoolTokenAmount).add(new BN(1));

        if (tradeableToken.lte(st)) {
            st = tradeableToken;
            index -= 1;
            if(tradeableToken.eq(new BN(0))){
                continue;
            }
        }
        console.log(`st ${st}`)
        const result = calculate_swap_amount_in(st, newPoolTokenAmount,newPoolSolAmount);
        console.log(`calculate_swap_amount_in result ${result}`)
        const userToken = ceil_div(st.mul(DENOMINATOR_VALUE), leverageRate);
        const aiToken = st.sub(userToken);
        userTokenAmount = userTokenAmount.add(userToken);
        aiTokenAmount = aiTokenAmount.add(aiToken);
        
        const userSol = ceil_div(result.mul(DENOMINATOR_VALUE), leverageRate);
        const aiSol = result.sub(userSol);
        userSolAmount = userSolAmount.add(userSol);
        aiSolAmount = aiSolAmount.add(aiSol);

        console.log("level :" + leverageRate);
        console.log("user token: " + userToken);
        console.log("user Sol: " + userSol);

        newPoolSolAmount = newPoolSolAmount.sub(result);
        newPoolTokenAmount = newPoolTokenAmount.add(st);
    }
    return userSolAmount;
}

function sellSolCalculateToken(
    solAmount: BN,
    poolSolAmount: BN,
    poolTokenAmount: BN,
    currentLeverageIndex: number,
    leverage: [BN, BN, BN, BN, BN][],
) {
    let aiTokenAmount = new BN(0);
    let userTokenAmount = new BN(0);
    let aiSolAmount = new BN(0);
    let userSolAmount = new BN(0);
    let newPoolSolAmount = poolSolAmount;
    let newPoolTokenAmount = poolTokenAmount;
    let index = currentLeverageIndex;

    while (userSolAmount.lt(solAmount) && index >= 0) {
        const currentLeverage = leverage[index];
        const leverageRate = currentLeverage[0];
        const startSol = currentLeverage[3];
        let st = solAmount.sub(userSolAmount);
        st = st.mul(leverageRate).div(DENOMINATOR_VALUE)
        let tradeableSol = newPoolSolAmount.sub(startSol).add(new BN(1));

        if (tradeableSol.lte(st)) {
            st = tradeableSol;
            index -= 1;
            if (tradeableSol.eq(new BN(0))) {
                continue;
            }
        }
        const result = calculate_swap_amount_out(st, newPoolTokenAmount, newPoolSolAmount);

        const userSol = ceil_div(st.mul(DENOMINATOR_VALUE), leverageRate);
        const aiSol = st.sub(userSol);
        userSolAmount = userSolAmount.add(userSol);
        aiSolAmount = aiSolAmount.add(aiSol);

        const userToken = ceil_div(result.mul(DENOMINATOR_VALUE), leverageRate);
        const aiToken = result.sub(userToken);
        userTokenAmount = userTokenAmount.add(userToken);
        aiTokenAmount = aiTokenAmount.add(aiToken);

        newPoolSolAmount = newPoolSolAmount.sub(st);
        newPoolTokenAmount = newPoolTokenAmount.add(result);
    }
    return userTokenAmount;
}

function calculate_swap_amount_in(
    source_amount: BN,
    swap_source_amount: BN,
    swap_destination_amount: BN,
) : BN {
    // (x + delta_x) * (y - delta_y) = x * y
    // delta_y = (delta_x * y) / (x + delta_x)
    let numerator = source_amount.mul(swap_destination_amount);
    let denominator = swap_source_amount.add(source_amount);
    let destinsation_amount_swapped = numerator.div(denominator);
    return destinsation_amount_swapped
}

function calculate_swap_amount_out(
    destinsation_amount: BN,
    swap_source_amount: BN,
    swap_destination_amount: BN,
) : BN {
    // (x + delta_x) * (y - delta_y) = x * y
    // delta_x = (x * delta_y) / (y - delta_y)
    let numerator = swap_source_amount.mul(destinsation_amount);
    let denominator = swap_destination_amount
        .sub(destinsation_amount);
    let source_amount_swapped = ceil_div(numerator, denominator);
    return source_amount_swapped
}

interface PoolInfo {
    realSol: BN;
    virtualSol: BN;
    launchTokenSurplus: BN;
    poolSolAmount: BN;
    poolTokenAmount: BN;
    currentLeverageIndex:number;
    leverage: [BN, BN, BN, BN, BN][];
}


interface PayParams {
    amount: BN;
    isSol: boolean;
    slippage: BN;
    pumpupFeeRate: BN;
    poolSolAmount: BN;
    poolTokenAmount: BN;
    launchTokenSurplus: BN;
    currentLeverageIndex:number;
    leverage: [BN, BN, BN, BN, BN][];
}


interface BuyWithSlippage {
    buyTokenAmount: BN;
    maxSolAmount: BN;
}

interface SellWithSlippage {
    sellTokenAmount: BN;
    minSolAmount: BN;
}