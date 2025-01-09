import * as anchor from '@coral-xyz/anchor';
import { Program } from "@coral-xyz/anchor";
import { Pumpup } from "../target/types/pumpup";
import { Connection, PublicKey, Keypair, Transaction, Signer, ComputeBudgetProgram } from "@solana/web3.js";
import { buildFeedDataInstruction, buildInferenceInstruction, getConfigAccount, getOriginalAccount, getPoolInfo, loadMigrationAddress, loadMints } from './libs';
import { BN } from 'bn.js';

describe("ai leverage", () => {
  const connection = new Connection("https://api.devnet.solana.com");
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const pumpuProgram = anchor.workspace.Pumpup as Program<Pumpup>;
  const payer = provider.wallet as anchor.Wallet;

  it.skip('get transaction cpi event', async () => {
    let tx = await connection.getTransaction('55gPYsarBrLgEXGpyrHkCv5tmpuBQMFSQ6WsaLbiM1KJqjG3Z65PWdReTBmr6rRYC4Aq44BbsfewFHEaziWAtEtw',
      {commitment:'confirmed', maxSupportedTransactionVersion:0 })
    console.log("tx:{}", tx.meta.postTokenBalances);
    const ixData = anchor.utils.bytes.bs58.decode(tx.meta.innerInstructions[0].instructions[0].data);
    console.log('ixData:{}', tx.meta.innerInstructions[0].instructions[0])
    const eventData = anchor.utils.bytes.base64.encode(ixData.slice(8));
    const event = pumpuProgram.coder.events.decode(eventData);
    console.log("event:{}", event);
  });

  it('feed_data', async () => {

    let migrationAddress = loadMigrationAddress();
    const migrationWallet = new anchor.Wallet(migrationAddress);
    console.log(`migrationAddress:${migrationAddress.publicKey}`)
    provider.wallet = migrationWallet;
    let mint_pubkey = loadMints().pop();
    console.log(`mint:${mint_pubkey}`)
    let originalAccount =  getOriginalAccount(mint_pubkey, pumpuProgram);
    console.log(`originalAccount:${originalAccount[0]}`)
    const tx = new Transaction();
    try {
      let input1 = Array(33).fill(12.0);
      input1[2] = 4.2;
      const feedData1 = await buildFeedDataInstruction(mint_pubkey, pumpuProgram, input1);
      tx.add(ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }));
      tx.add(feedData1);

      const signature = await provider.sendAndConfirm(tx, [migrationAddress]);
      console.log('Transaction signature:', signature);
    } catch (error) {
      console.log('feed data error: ', error);
    }
    let original_data = await pumpuProgram.account.aiOriginalDataAccount.fetch(originalAccount[0]);
    console.log(`origianl data, period: ${original_data.period}, periodDone: ${original_data.periodDone}, regulated: ${original_data.regulated}, datas:${original_data.datas}`)

  });

  it('inference', async () => {
    let migrationAddress = loadMigrationAddress();
    const migrationWallet = new anchor.Wallet(migrationAddress);
    provider.wallet = migrationWallet;

    let mint_pubkey = loadMints().pop();

    const tx = new Transaction();

    console.log("========= before inference =========")
    let originalAccount = getOriginalAccount(mint_pubkey, pumpuProgram);
    let original_data = await pumpuProgram.account.aiOriginalDataAccount.fetch(originalAccount[0]);
    console.log(`origianl data, period: ${original_data.period}, periodDone: ${original_data.periodDone},regulated: ${original_data.regulated},`)
    await getPoolInfo(mint_pubkey, pumpuProgram);
    

    try {
      let period = original_data.period;
      const inferenceIx = await buildInferenceInstruction(mint_pubkey, period, pumpuProgram);
      tx.add(ComputeBudgetProgram.setComputeUnitLimit({ units: 12_000_000 }));
      tx.add(inferenceIx);

      const signature = await provider.sendAndConfirm(tx, [migrationAddress]);
      console.log('Transaction signature:', signature);
    } catch (error) {
      console.log('inference error: ', error);
    }


    console.log("========= after inference =========")
    original_data = await pumpuProgram.account.aiOriginalDataAccount.fetch(originalAccount[0]);
    console.log(`origianl data, period: ${original_data.period}, periodDone: ${original_data.periodDone}, regulated: ${original_data.regulated},`)
    await getPoolInfo(mint_pubkey, pumpuProgram);
  });

  it.skip('test original account realloc', async () => {

    let migrationAddress = loadMigrationAddress();
    const migrationWallet = new anchor.Wallet(migrationAddress);
    provider.wallet = migrationWallet;

    console.log("migration address: ", migrationWallet.publicKey);

    let mint_pubkey = loadMints().pop();
    //config account
    let configurationAccount = getConfigAccount(pumpuProgram);
    console.log("configurationAccount address: ", configurationAccount);

    let originalPDA, bump = getOriginalAccount(mint_pubkey, pumpuProgram);

    const tx = new Transaction();
    for (let i = 0; i < 3; i++) {
      const keypair = Keypair.generate();
      let input1 = Array(33).fill(0.0);
      const feedData1 = await buildFeedDataInstruction(keypair.publicKey, pumpuProgram, input1);
      tx.add(feedData1);
    }
    const signature = await provider.sendAndConfirm(tx, [migrationAddress]);
    console.log('Transaction signature:', signature);


    const original_data = await pumpuProgram.account.aiOriginalDataAccount.fetch(originalPDA[0]);
    console.log("original_data {}, length:{}", original_data, original_data.datas.length);
  });

});
