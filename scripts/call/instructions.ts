import { Connection, Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { bs58 } from "@project-serum/anchor/dist/cjs/utils/bytes";
import { networkName } from "../splHelper/consts";
import {
  AnchorProvider,
  BN,
  Program,
  Wallet,
  web3 as anchorWeb3,
} from "@project-serum/anchor";
import { getNetworkConfig } from "../splHelper/helper";
import { getOrCreateAssociatedTokenAccount, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";

const idl = require("./idl.json");

const getProviderAndAddresses = () => {
  const icoMint = new PublicKey("AvEt25pkz91AaJM1K2bGcCGvm1AzfELFkQgKQEFUQc7n");
  const prodramId = new PublicKey("4bLbF6LwTuiPY5V63A7v4N8Uabcawt2HpjfobrjknLhm");

  // const secretKey = process.env.USER_WALLET;
  // const adminWallet = Keypair.fromSecretKey(bs58.decode(secretKey)); // from secretkey
  const ADMIN_KEYPAIR = process.env.ADMIN_KEYPAIR;
  const privateKeyArray = Uint8Array.from(JSON.parse(ADMIN_KEYPAIR));
  const adminWallet = Keypair.fromSecretKey(new Uint8Array(privateKeyArray)); // from keyfair
  console.log("wallet: ", adminWallet.publicKey.toString());

  const network = getNetworkConfig(networkName);
  const connection = new Connection(network.cluster, {
    commitment: "confirmed",
  });

  const opts: any = { preflightCommitment: "processed" };
  const provider = new AnchorProvider(
    connection,
    new Wallet(adminWallet),
    opts.preflightCommitment
  );
  const program = new Program(idl, prodramId, provider);

  const [programATA, _programATABump] = findProgramAddressSync(
    [Buffer.from("program_ata"), icoMint.toBuffer()],
    prodramId
  );

  const [icoPDA, _icoPDABump] = findProgramAddressSync(
    [Buffer.from("ico_pda")],
    prodramId
  );

  console.log({
    adminWallet: adminWallet.publicKey.toString(),
    icoMint: icoMint.toString(),
    programATA: programATA.toString(),
    icoPDA: icoPDA.toString(),
    // adminAta: adminAta.address.toString(),
  });

  return {
    adminWallet,
    connection,
    provider,
    program,
    prodramId,
    icoMint,
    programATA,
    icoPDA,
  };
};

async function initiateAndCreateProgramATA(tokensPerLamport, tokensDepositForICO) {
  const { adminWallet, connection, program, icoMint, programATA, icoPDA } =
    getProviderAndAddresses();

  const adminAta = await getOrCreateAssociatedTokenAccount(
    connection,
    adminWallet,
    icoMint,
    adminWallet.publicKey
  );

  const sigTx = await program.methods
    .initiateAndCreateProgramAta(new BN(tokensPerLamport), new BN(tokensDepositForICO))
    .accounts({
      admin: adminWallet.publicKey,
      icoMint,
      adminAta: adminAta.address, // Replace with admin's ATA
      programAta: programATA,
      icoPda: icoPDA,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: anchorWeb3.SystemProgram.programId,
      rent: anchorWeb3.SYSVAR_RENT_PUBKEY,
    })
    .signers([adminWallet])
    .rpc();

  console.log("Program initialized and program ATA created.");
  console.log(`sigTx: `, sigTx);
}

async function buyWithSol(amount, buyerPublicKey = undefined) {
  const { adminWallet, connection, program, icoMint, programATA, icoPDA } =
    getProviderAndAddresses();

  buyerPublicKey = buyerPublicKey ?? adminWallet.publicKey;

  const buyerAta = await getOrCreateAssociatedTokenAccount(
    connection,
    adminWallet,
    icoMint,
    buyerPublicKey
  );

  const sigTx = await program.methods
    .buyWithSol(new BN(amount))
    .accounts({
      admin: adminWallet.publicKey,
      buyer: buyerPublicKey,
      buyerAta: buyerAta.address,
      programAta: programATA,
      icoPda: icoPDA,
      systemProgram: anchorWeb3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([adminWallet])
    .rpc();

  console.log(`${amount} lamports used to buy tokens.`);
  console.log(`sigTx: `, sigTx);
}

async function withdrawTokens(amount) {
  const { adminWallet, connection, program, icoMint, programATA, icoPDA } =
    getProviderAndAddresses();

  const adminAta = await getOrCreateAssociatedTokenAccount(
    connection,
    adminWallet,
    icoMint,
    adminWallet.publicKey
  );

  const sigTx = await program.methods
    .withdrawTokens(new BN(amount))
    .accounts({
      admin: adminWallet.publicKey,
      adminAta: adminAta.address, // Replace with admin's ATA
      programAta: programATA,
      icoPda: icoPDA,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([adminWallet])
    .rpc();

  console.log(`${amount} tokens withdrawn from program ATA.`);
  console.log(`sigTx: `, sigTx);
}

async function depositTokens(amount) {
  const { adminWallet, connection, program, icoMint, programATA, icoPDA } =
    getProviderAndAddresses();

  const adminAta = await getOrCreateAssociatedTokenAccount(
    connection,
    adminWallet,
    icoMint,
    adminWallet.publicKey
  );

  const sigTx = await program.methods
    .depositTokens(new BN(amount))
    .accounts({
      admin: adminWallet.publicKey,
      adminAta: adminAta.address, // Replace with admin's ATA
      programAta: programATA,
      icoPda: icoPDA,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([adminWallet])
    .rpc();

  console.log(`${amount} tokens deposited to program ATA.`);
  console.log(`sigTx: `, sigTx);
}

async function updatePrice(tokensPerLamport) {
  const { adminWallet, program, icoPDA } = getProviderAndAddresses();

  const sigTx = await program.methods
    .updatePrice(new BN(tokensPerLamport)) // Pass only the argument here
    .accounts({
      admin: adminWallet.publicKey,
      icoPda: icoPDA,
    })
    .signers([adminWallet])
    .rpc();

  console.log(`Token price updated to ${tokensPerLamport} tokens per lamport.`);
  console.log(`sigTx: `, sigTx);
}

async function getIcoPdaData() {
  const { program, icoPDA } = getProviderAndAddresses();
  const data = await program.account.icoDataPda.fetch(icoPDA);
  const _programData = {};
  Object.entries(data).some(([key, value]) => {
    _programData[key] = value?.toString();
  });

  console.log(_programData);
}

// 100000000000
// 10000000000
// for initial price 0.0000001 sol token amount will be 10000
// for initial price 0.0001 lamport token amount will be 10
// 0.0005
(() => {
  // const decomals = 6;
  // initiateAndCreateProgramATA(100000, 10000 * 10 ** decomals);
  // buyWithSol(0.0005 * LAMPORTS_PER_SOL)
  // withdrawTokens(1000 * 10 ** decomals);
  // depositTokens(1005 * 10 ** decomals);
  // updatePrice(2);
  // getIcoPdaData();
})();

// const newPriceInSol = 0.0005;
// const decimals = 6;
// const tokensPerSol =
//   (LAMPORTS_PER_SOL / (newPriceInSol * LAMPORTS_PER_SOL)) * 10 ** decimals;
// console.log("tokensPerLamport: ", tokensPerSol / LAMPORTS_PER_SOL);
