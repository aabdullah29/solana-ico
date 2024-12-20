use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

const ICO_MINT: &str = "AvEt25pkz91AaJM1K2bGcCGvm1AzfELFkQgKQEFUQc7n";
const PROGRAM_ATA_SEED: &[u8] = b"program_ata";
const ICO_PDA_SEED: &[u8] = b"ico_pda";

declare_id!("FGyoWBVesESYEJidurSeEBSgqG75No238xHaeJ8ZnH6Z");

#[program]
mod ico_program {
    use super::*;

    // Initiates the Program, creates the program token account and initializes data
    pub fn initiate_and_create_program_ata(
        ctx: Context<InitiateAndCreateProgramATA>,
        tokens_per_lamport: u64,
        tokens_deposit_for_ico: u64,
    ) -> Result<()> {
        // Check for invalid SOL per token value
        if tokens_per_lamport == 0 {
            return Err(ProgramError::InvalidArgument.into());
        }

        // transfer tokens from admin ata to program ata
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.admin_ata.to_account_info(),
                to: ctx.accounts.program_ata.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, tokens_deposit_for_ico)?;

        let ico_pda = &mut ctx.accounts.ico_pda;
        ico_pda.admin = ctx.accounts.admin.key();
        ico_pda.tokens_per_lamport = tokens_per_lamport;
        ico_pda.tokens_balance = tokens_deposit_for_ico;
        ico_pda.ata_bump = ctx.bumps.program_ata;

        msg!(
            "Program initiated with {} lamports per token and deposit {} tokens",
            tokens_per_lamport,
            tokens_deposit_for_ico
        );
        Ok(())
    }

    // Allows anyone to buy tokens with SOL and send SOL as lamports to the admin's account
    pub fn buy_with_sol(ctx: Context<BuyWithSol>, lamports: u64) -> Result<()> {
        if lamports == 0 {
            return Err(ProgramError::InvalidArgument.into());
        }

        // Check if buyer has enough lamports to cover the purchase
        if **ctx.accounts.buyer.try_borrow_mut_lamports()? < lamports {
            return Err(ProgramError::InsufficientFunds.into());
        }

        // Calculate total cost and check for overflow
        let ico_pda = &mut ctx.accounts.ico_pda;
        let tokens_amount = lamports
            .checked_mul(ico_pda.tokens_per_lamport)
            .ok_or(IcoCustomError::MathOverflow)?;

        // Check if enough tokens are available for purchase
        if ico_pda.tokens_balance < tokens_amount {
            return Err(ProgramError::InsufficientFunds.into());
        }

        // Deduct/Transfer SOL from buyer and send to admin
        let ix = solana_program::system_instruction::transfer(
            &ctx.accounts.buyer.key(),
            &ctx.accounts.admin.key(),
            lamports,
        );
        solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.admin.to_account_info(),
            ],
        )?;

        // Get the seeds and bump for the Program ATA signer
        let ico_mint_pubkey = ICO_MINT.parse::<Pubkey>().unwrap();
        let seeds = &[PROGRAM_ATA_SEED, ico_mint_pubkey.as_ref(), &[ico_pda.ata_bump]];
        let signer = [&seeds[..]];
        // Transfer tokens to the buyer's associated token account
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.program_ata.to_account_info(),
                to: ctx.accounts.buyer_ata.to_account_info(),
                authority: ctx.accounts.program_ata.to_account_info(),
            },
            &signer,
        );
        token::transfer(cpi_context, tokens_amount)?;

        // Update data for tokens sold and funds received
        ico_pda.lamports_received = ico_pda
            .lamports_received
            .checked_add(lamports)
            .ok_or(IcoCustomError::MathOverflow)?;
        ico_pda.total_sold = ico_pda
            .total_sold
            .checked_add(tokens_amount)
            .ok_or(IcoCustomError::MathOverflow)?;
        ico_pda.tokens_balance = ico_pda
            .tokens_balance
            .checked_sub(tokens_amount)
            .ok_or(IcoCustomError::MathOverflow)?;

        msg!(
            "User bought {} tokens for {} lamports",
            tokens_amount,
            lamports,
        );
        Ok(())
    }

    // Admin can withdraw tokens from the Program ATA account
    pub fn withdraw_tokens(ctx: Context<WithdrawTokens>, amount: u64) -> Result<()> {
        if amount == 0 {
            return Err(ProgramError::InvalidArgument.into());
        }

        // Ensure enough tokens are available for withdrawal
        let ico_pda = &ctx.accounts.ico_pda;
        if ico_pda.tokens_balance < amount {
            return Err(ProgramError::InsufficientFunds.into());
        }

        // Get the seeds and bump for the Program ATA signer
        let ico_mint_pubkey = ICO_MINT.parse::<Pubkey>().unwrap();
        let seeds = &[PROGRAM_ATA_SEED,ico_mint_pubkey.as_ref(), &[ico_pda.ata_bump]];
        let signer = [&seeds[..]];
        // Transfer tokens to the admin's associated token account
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.program_ata.to_account_info(),
                to: ctx.accounts.admin_ata.to_account_info(),
                authority: ctx.accounts.program_ata.to_account_info(),
            },
            &signer,
        );
        token::transfer(cpi_context, amount)?;

        // Update data for the withdrawn amount
        let ico_pda = &mut ctx.accounts.ico_pda;
        ico_pda.tokens_balance = ico_pda
            .tokens_balance
            .checked_sub(amount)
            .ok_or(IcoCustomError::MathOverflow)?;

        msg!("Admin withdrew {} tokens from Program ATA", amount);
        Ok(())
    }

    // Admin can deposit tokens in Program ATA account
    pub fn deposit_tokens(ctx: Context<DepositTokens>, amount: u64) -> Result<()> {
        if amount == 0 {
            return Err(ProgramError::InvalidArgument.into());
        }

        // transfer tokens from admin ata to program ata
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.admin_ata.to_account_info(),
                to: ctx.accounts.program_ata.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount)?;

        // Update data for the withdrawn amount
        let ico_pda = &mut ctx.accounts.ico_pda;
        ico_pda.tokens_balance = ico_pda
            .tokens_balance
            .checked_add(amount)
            .ok_or(IcoCustomError::MathOverflow)?;

        msg!("Admin deposit {} tokens in Program ATA", amount);
        Ok(())
    }

    // Admin update token price
    pub fn update_price(ctx: Context<UpdatePrice>, tokens_per_lamport: u64) -> Result<()> {
        if tokens_per_lamport == 0 {
            return Err(ProgramError::InvalidArgument.into());
        }

        // Update data for the withdrawn amount
        let ico_pda = &mut ctx.accounts.ico_pda;
        ico_pda.tokens_per_lamport = tokens_per_lamport;

        msg!(
            "Admin update token price {} token/lmports",
            tokens_per_lamport
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitiateAndCreateProgramATA<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        address = ICO_MINT.parse::<Pubkey>().unwrap(),
        )]
    pub ico_mint: Account<'info, Mint>,

    #[account(mut)]
    pub admin_ata: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        seeds = [PROGRAM_ATA_SEED, ICO_MINT.parse::<Pubkey>().unwrap().as_ref()],
        bump,
        token::mint = ico_mint,
        token::authority = program_ata,
    )]
    pub program_ata: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        seeds = [ICO_PDA_SEED],
        bump,
        space = 8 + std::mem::size_of::<IcoDataPda>(),
    )]
    pub ico_pda: Account<'info, IcoDataPda>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BuyWithSol<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    pub admin: AccountInfo<'info>,

    #[account(mut)]
    pub buyer_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [PROGRAM_ATA_SEED, ICO_MINT.parse::<Pubkey>().unwrap().as_ref()],
        bump= ico_pda.ata_bump,
        )]
    pub program_ata: Account<'info, TokenAccount>,

    #[account(mut, has_one = admin, seeds=[ICO_PDA_SEED], bump)]
    pub ico_pda: Account<'info, IcoDataPda>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawTokens<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub admin_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [PROGRAM_ATA_SEED, ICO_MINT.parse::<Pubkey>().unwrap().as_ref()],
        bump= ico_pda.ata_bump,
        )]
    pub program_ata: Account<'info, TokenAccount>,

    #[account(mut, has_one = admin, seeds=[ICO_PDA_SEED], bump)]
    pub ico_pda: Account<'info, IcoDataPda>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositTokens<'info> {
    pub admin: Signer<'info>,

    #[account(mut)]
    pub admin_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [PROGRAM_ATA_SEED, ICO_MINT.parse::<Pubkey>().unwrap().as_ref()],
        bump= ico_pda.ata_bump,
        )]
    pub program_ata: Account<'info, TokenAccount>,

    #[account(mut, has_one = admin, seeds=[ICO_PDA_SEED], bump)]
    pub ico_pda: Account<'info, IcoDataPda>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut, has_one = admin, seeds=[ICO_PDA_SEED], bump)]
    pub ico_pda: Account<'info, IcoDataPda>,

    pub system_program: Program<'info, System>,
}

// Data structure to hold ICO-related state
#[account]
pub struct IcoDataPda {
    pub admin: Pubkey,
    pub tokens_per_lamport: u64,
    pub tokens_balance: u64,
    pub total_sold: u64,
    pub lamports_received: u64,
    pub ata_bump: u8,
}

// Custom error enum for the ICO program
#[error_code]
pub enum IcoCustomError {
    #[msg("Mathematical overflow during operations.")]
    MathOverflow,

}
