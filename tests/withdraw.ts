import * as anchor from '@coral-xyz/anchor';
import { Program } from "@coral-xyz/anchor";
import { Pumpup } from "../target/types/pumpup";
import { Connection, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import { buildWithdrawInstruction, loadMigrationAddress, loadMints } from './libs';

describe("pumpup withdraw", () => {
  const connection = new Connection("https://api.devnet.solana.com");
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const pumpuProgram = anchor.workspace.Pumpup as Program<Pumpup>;
  const payer = provider.wallet as anchor.Wallet;

  it('withdraw', async () => {

    let migrationAddress = loadMigrationAddress()
    const migrationWallet = new anchor.Wallet(migrationAddress);
    provider.wallet = migrationWallet;
    try{
        console.log("migrationWallet", migrationWallet.publicKey);
        const mint_pubkey = loadMints().pop();

        let withdraw =  await buildWithdrawInstruction(mint_pubkey, pumpuProgram);

        const tx = new Transaction().add(withdraw);
        // send tx
        const signature = await provider.sendAndConfirm(tx, [migrationAddress]);
        console.log('Transaction signature:', signature);
      } catch (error) {
        console.log('withdraw mint error: ', error);
      }
  });


  it.skip('migration to user', async () => {
    let migrationAddress = loadMigrationAddress()
    let transferInstraction = SystemProgram.transfer({
        fromPubkey: migrationAddress.publicKey,
        toPubkey: payer.publicKey,
        lamports: 1000
      });
      const tx = new Transaction().add(transferInstraction);
      // send tx
      const signature = await provider.sendAndConfirm(tx, [migrationAddress]);
      console.log('Transaction signature:', signature);
  });

});
