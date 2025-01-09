import * as anchor from '@coral-xyz/anchor';
import { Program } from "@coral-xyz/anchor";
import { Pumpup } from "../target/types/pumpup";
import { Keypair, PublicKey, SystemProgram, Transaction, SYSVAR_RENT_PUBKEY} from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID} from "@solana/spl-token";
import { appendedMintPubkey, getConfigAccount, getMintAuthAccount, getPoolSolAccount, getPoolTokenAccount, loadMints, getMetadataAccount, getAiTokenAccount } from './libs';
const MPL_TOKEN_METADATA_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

describe("pumpup create", () => {
  
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const pumpuProgram = anchor.workspace.Pumpup as Program<Pumpup>;
  const payer = provider.wallet as anchor.Wallet;
  const path = require('path');

  it('create', async () => {
      let mint = await createMint();
      console.log("created mint", mint);
  });

  it.skip('create and recode', async () => {
    for(let i = 0; i< 10; i++){
      let mint = await createMint();
    }
  });

  it.skip('load mints', async () => {
      let mints = await loadMints();
      console.log("mints", mints);
  });


  async function createMint(): Promise<anchor.web3.PublicKey>{
    let name =  "test coin";
    let symbol =  "testcoin";
    let uri =  "https://ipfs.io/ipfs/QmbiMaKCHCKrkuxxZeYWYfNVmzJaYyrV2kEqZrCXC1vjFk";

    //config account
    let config = getConfigAccount(pumpuProgram);
    let mint: Keypair;
    for (let i = 0; i < 10000; i++) {
      const keypair = Keypair.generate();
      if (keypair.publicKey.toBase58().endsWith("up")) {
        mint = keypair;
        break;
      }
    }

    
    let mintAuthAccount = getMintAuthAccount(pumpuProgram);
    const [poolSolAccount, bump] = getPoolSolAccount(mint.publicKey,pumpuProgram);
    const aiTokenAccount= getAiTokenAccount(mint.publicKey, pumpuProgram);
    const poolTokenAccount = getPoolTokenAccount(mint.publicKey, pumpuProgram);
    const metadataAccount = getMetadataAccount(mint.publicKey);
    console.log("metadataAccount: " + metadataAccount);

    console.log("poolSolAccount: " + poolSolAccount);

    console.log("poolTokenAccount: " + poolTokenAccount);

    console.log("aiTokenAccount: " + aiTokenAccount);

    // create
    try {
     let createIx =  await pumpuProgram.methods
            .create(name, symbol, uri)
            .accountsPartial({
              creator: payer.publicKey,
              mint: mint.publicKey,
              poolTokenAccount: poolTokenAccount,
              poolSolAccount: poolSolAccount,
              mintAuthority: mintAuthAccount[0],
              metadataAccount: metadataAccount[0],
              rent: SYSVAR_RENT_PUBKEY,
              tokenProgram: TOKEN_PROGRAM_ID,
              tokenMetadataProgram: MPL_TOKEN_METADATA_ID,
              associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
            })
            .instruction();

            let initAi =  await pumpuProgram.methods
            .initAiToken()
            .accountsPartial({
              config: config[0],
              creator: payer.publicKey,
              mint: mint.publicKey,
              rent: SYSVAR_RENT_PUBKEY,
              tokenProgram: TOKEN_PROGRAM_ID,
              associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
            })
            .instruction();
      const tx = new Transaction()
        .add(createIx).add(initAi)
      // send tx
      const signature = await provider.sendAndConfirm(tx, [payer.payer, mint]);
      console.log('Transaction signature:', signature);
      appendedMintPubkey(mint.publicKey);
      return mint.publicKey;
    } catch (error) {
      console.log('create mint error: ', error);
    }
  }

});


