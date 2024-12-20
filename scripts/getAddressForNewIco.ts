import { toPublicKey } from "@metaplex-foundation/js";
import { getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import { Connection, Keypair } from "@solana/web3.js";
import { getNetworkConfig } from "./splHelper/helper";
import { networkName } from "./splHelper/consts";
import { bs58 } from "@project-serum/anchor/dist/cjs/utils/bytes";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";

(async () => {
  // Constants
  const icoMintAddress = toPublicKey("AvEt25pkz91AaJM1K2bGcCGvm1AzfELFkQgKQEFUQc7n"); // ICO
  const icoProgramAddress = toPublicKey("FZZPymCYLZHYb3krdyXSPLvm2YqNmJTZLrjenaCJNJGE");
  const userPubkey = toPublicKey("2vLR1s4cmXkYLutA8Xex7Mj1KmuxHw2ahL6GPXrJyEZN");
  const adminPubkey = toPublicKey("C5jtiLaDBDoRL1dkag8gVEQ7xR9GtJ36wdL57xyfHzkF");

  // Derive Program ATA (PDA) for ICO Mint
  const [programAtaPDA, programAtaPDABump] = findProgramAddressSync(
    [icoMintAddress.toBuffer()],
    icoProgramAddress
  );
  console.log(
    "programAtaPDA: ",
    programAtaPDA.toString(),
    programAtaPDABump
  );

  // Derive ICO PDA (for storing ICO data)
  const [icoDataPDA, icoDataPDABump] = findProgramAddressSync(
    [Buffer.from("ico_pda")],
    icoProgramAddress
  );
  console.log(
    "icoDataPDA: ",
    icoDataPDA.toString(),
    icoDataPDABump
  );

  // Network and Wallet setup
  const network = getNetworkConfig(networkName);
  const connection = new Connection(network.cluster, {
    commitment: "confirmed",
  });
  const secretKey: any = process.env.USER_WALLET;
  const userWallet = Keypair.fromSecretKey(bs58.decode(secretKey));
  console.log("wallet: ", userWallet.publicKey.toString());

  // Create Associated Token Accounts (ATAs)
  const icoAtaForAdmin = (await getOrCreateAssociatedTokenAccount(
    connection,
    userWallet,
    icoMintAddress,
    adminPubkey,
    false
  )).address.toString();

  const icoAtaForUser = (await getOrCreateAssociatedTokenAccount(
    connection,
    userWallet,
    icoMintAddress,
    userPubkey,
    false
  )).address.toString();

  console.log({icoAtaForAdmin, icoAtaForUser})

  
  // console.log(`\n\n
  // initiateAndCreateProgramATA --> (250, 1000000)
  // programAta: ${programAtaPDA.toString()}
  // icoDataPDA: ${icoDataPDA.toString()}
  // icoMint: ${icoMintAddress.toString()}
  // adminAta: ${icoAtaForAdmin.address.toString()}
  // admin: ${userWallet.publicKey.toString()}
  // `);

  // console.log(`\n\n
  // buyWithSol --> (${programAtaPDABump}, 50)
  // programAta: ${programAtaPDA.toString()}
  // icoDataPDA: ${icoDataPDA.toString()}
  // icoMint: ${icoMintAddress.toString()}
  // userAta: ${icoAtaForUser.address.toString()}
  // buyer: ${userPubkey.toString()}
  // admin: ${userWallet.publicKey.toString()}
  // tokenProgram: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
  // systemProgram: 11111111111111111111111111111111
  // `);

  // console.log(`\n\n
  // withdrawTokens --> (${programAtaPDABump}, 500)
  // programAta: ${programAtaPDA.toString()}
  // icoDataPDA: ${icoDataPDA.toString()}
  // adminAta: ${icoAtaForAdmin.address.toString()}
  // admin: ${userWallet.publicKey.toString()}
  // tokenProgram: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
  // `);

  // console.log(`\n\n
  // depositTokens --> (1000)
  // programAta: ${programAtaPDA.toString()}
  // icoDataPDA: ${icoDataPDA.toString()}
  // adminAta: ${icoAtaForAdmin.address.toString()}
  // admin: ${userWallet.publicKey.toString()}
  // tokenProgram: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
  // `);

  // console.log(`\n\n
  // updatePrice --> (200)
  // icoDataPDA: ${icoDataPDA.toString()}
  // admin: ${userWallet.publicKey.toString()}
  // `);
})();


/*
  initiateAndCreateProgramATA= (250, 1000000)
  admin= C5jtiLaDBDoRL1dkag8gVEQ7xR9GtJ36wdL57xyfHzkF
  icoMint= AvEt25pkz91AaJM1K2bGcCGvm1AzfELFkQgKQEFUQc7n
  adminAta= DE9hDfuK47kNbs5mraXrJskUq3YYdLh2Bn6fCkJ8tX2E
  programAta= Ch8ofNJZgvXUK8mVDirZMtmakTGsdSy8R84hwuTKYRWC
  icoPda= DTtmbGXevnr5NAqHUzwKQf41r8ysRGDikjBYGYprPM4e
*/


/*
buyWithSol= (4000000, 254)
buyer= C5jtiLaDBDoRL1dkag8gVEQ7xR9GtJ36wdL57xyfHzkF
admin= C5jtiLaDBDoRL1dkag8gVEQ7xR9GtJ36wdL57xyfHzkF

*/
