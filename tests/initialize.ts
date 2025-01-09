import * as anchor from '@coral-xyz/anchor';
import { Program } from "@coral-xyz/anchor";
import { Pumpup } from "../target/types/pumpup";
import { Keypair, SystemProgram, Transaction, SYSVAR_RENT_PUBKEY} from "@solana/web3.js";
import { BN } from "bn.js";
import { getConfigAccount, getMintAuthAccount, loadFeeAddress, loadKeypair, loadMigrationAddress } from './libs';

describe("pumpup initialize", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const pumpuProgram = anchor.workspace.Pumpup as Program<Pumpup>;
  const payer = provider.wallet as anchor.Wallet;
  const path = require('path');
  const fs = require('fs');

  it.skip('initialize wallet', async () => {
    // //init feeAddress    
    let feeAddress;
    const feeAddressPath = `./keys/feeAddress.json`;
    if (fs.existsSync(feeAddressPath)) {
      feeAddress = loadKeypair(feeAddressPath);
    } else {
      let feeAddress = Keypair.generate();
      await fs.mkdirSync(path.dirname(feeAddressPath), { recursive: true });
      await fs.writeFileSync(feeAddressPath, JSON.stringify(Array.from(feeAddress.secretKey)));
    }
    //init migrationAddress
    let migrationAddress;
    const migrationAddressPath = `./keys/migrationAddress.json`;
    if (fs.existsSync(migrationAddressPath)) {
      migrationAddress =loadKeypair(migrationAddressPath);
    } else {[]
      let migrationAddress = Keypair.generate();
      await fs.mkdirSync(path.dirname(migrationAddressPath), { recursive: true });
      await fs.writeFileSync(migrationAddressPath, JSON.stringify(Array.from(migrationAddress.secretKey)));
    }
})

it('initialize account', async () => {

    const feeRate = new BN(1);
    let feeAddress = loadFeeAddress();
    let migrationAddress = loadMigrationAddress();

    console.log("feeAddress: ", feeAddress.publicKey);
    console.log("migrationAddress: ", migrationAddress.publicKey);

    let configurationAccount = await getConfigAccount(pumpuProgram);

    console.log("configurationAccount address: ", configurationAccount);

    let mintAuthAccount = await getMintAuthAccount(pumpuProgram);

    console.log("mint_authority address: ", mintAuthAccount);

    //initialize
    const initializeInstruction = await pumpuProgram.methods
        .initialize(feeRate)
        .accounts({
          config: configurationAccount,
          feeAddress: feeAddress.publicKey,
          migrationAddress: migrationAddress.publicKey,
          mintAuthority: mintAuthAccount,
          authorityAddress: payer.publicKey,
          rent: SYSVAR_RENT_PUBKEY,
          systemProgram: SystemProgram.programId,
        })
        .instruction();

        const tx = new Transaction().add(initializeInstruction);
        // send
        const signature = await provider.sendAndConfirm(tx, [payer.payer]);
        console.log('Transaction signature:', signature);
})

});
