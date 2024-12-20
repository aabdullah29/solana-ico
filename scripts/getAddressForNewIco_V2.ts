import { toPublicKey } from "@metaplex-foundation/js";
import { getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import { Connection, Keypair } from "@solana/web3.js";
import { getNetworkConfig } from "./splHelper/helper";
import { networkName } from "./splHelper/consts";
import { bs58 } from "@project-serum/anchor/dist/cjs/utils/bytes";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";

(async () => {
  // Constants
  const icoMintAddress = toPublicKey("AvEt25pkz91AaJM1K2bGcCGvm1AzfELFkQgKQEFUQc7n"); // ICO Mint
  const icoProgramAddress = toPublicKey("FGyoWBVesESYEJidurSeEBSgqG75No238xHaeJ8ZnH6Z");
  const userPubkey = toPublicKey("2vLR1s4cmXkYLutA8Xex7Mj1KmuxHw2ahL6GPXrJyEZN");
  const adminPubkey = toPublicKey("C5jtiLaDBDoRL1dkag8gVEQ7xR9GtJ36wdL57xyfHzkF");

  // Derive Program ATA (PDA) for ICO Mint
  const [programAtaPDA, programAtaPDABump] = findProgramAddressSync(
    [Buffer.from("program_ata"), icoMintAddress.toBuffer()],
    icoProgramAddress
  );
  console.log("Program ATA PDA: ", programAtaPDA.toString(), "Bump: ", programAtaPDABump);

  // Derive ICO PDA (for storing ICO data)
  const [icoDataPDA, icoDataPDABump] = findProgramAddressSync(
    [Buffer.from("ico_pda")],
    icoProgramAddress
  );
  console.log("ICO Data PDA: ", icoDataPDA.toString(), "Bump: ", icoDataPDABump);

  // Network and Wallet setup
  const network = getNetworkConfig(networkName);
  const connection = new Connection(network.cluster, {
    commitment: "confirmed",
  });
  const secretKey: any = process.env.USER_WALLET;
  const userWallet = Keypair.fromSecretKey(bs58.decode(secretKey));
  console.log("Wallet: ", userWallet.publicKey.toString());

  // Create Associated Token Accounts (ATAs)
  const icoAtaForAdmin = await getOrCreateAssociatedTokenAccount(
    connection,
    userWallet,
    icoMintAddress,
    adminPubkey
  );
  console.log("Admin ATA: ", icoAtaForAdmin.address.toString());

  const icoAtaForUser = await getOrCreateAssociatedTokenAccount(
    connection,
    userWallet,
    icoMintAddress,
    userPubkey
  );
  console.log("User ATA: ", icoAtaForUser.address.toString());

  // Further program interaction logic (e.g., initiate ICO, buy tokens, etc.) can be added here.
})();

/*
  initiateAndCreateProgramATA= (10000, 10000000000)
  admin= C5jtiLaDBDoRL1dkag8gVEQ7xR9GtJ36wdL57xyfHzkF
  icoMint= AvEt25pkz91AaJM1K2bGcCGvm1AzfELFkQgKQEFUQc7n
  adminAta= DE9hDfuK47kNbs5mraXrJskUq3YYdLh2Bn6fCkJ8tX2E
  programAta= 64zW4k7SCfM7BKZDyvdtd71Bd5vfiL5mxPqo2pabJUNt
  icoPda= 3mg3onodFTyLLPkwp7VR2CfN9ZPnhx4NKKt9QANSMSZB
*/

/*
buyWithSol= (4000000, 254)
buyerAta= H3GrPCkhH6EiJ76fQrMg33QrVxdwXj8a4g6afepGsZCB
adminATA= DE9hDfuK47kNbs5mraXrJskUq3YYdLh2Bn6fCkJ8tX2E

*/
