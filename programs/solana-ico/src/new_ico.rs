use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("FZZPymCYLZHYb3krdyXSPLvm2YqNmJTZLrjenaCJNJGE");

#[program]
pub mod ico {
    pub const ICO_MINT_ADDRESS: &str = "FBKhAghAqzttng8UAAf7VuX7msiNAtVxgEsY4PrfZxP4";
    use super::*;

    /* 
    ===========================================================
        initiate_and_create_ico_ata function use InitiateAndCreateIcoATA struct
    ===========================================================
*/
    pub fn initiate_and_create_ico_ata(
        ctx: Context<InitiateAndCreateIcoATA>,
        ico_tokens_amount: u64,
        sol_per_token: u64,
    ) -> Result<()> {
        // get ico_data from InitiateAndCreateIcoATA
        let ico_data = &mut ctx.accounts.ico_data;
        if ico_data.initiate {
            msg!("AccountAlreadyInitialized from program.initiate_and_create_ico_ata.",);
            return Err(IcoCustomError::AccountAlreadyInitialized.into());
        }

        msg!("create program ATA for hold ICO tokens.");
        // transfer ICO admin to program ata
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.ico_ata_for_admin.to_account_info(),
                to: ctx.accounts.ico_ata_for_ico_program.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, ico_tokens_amount)?;
        msg!("send {} ICO token to program ATA.", ico_tokens_amount);

        // save ico_data in ico_data PDA
        ico_data.sol_per_token = sol_per_token;
        ico_data.admin = *ctx.accounts.admin.key;
        ico_data.ico_tokens_balance = ico_tokens_amount;
        ico_data.initiate = true;
        msg!("save ico_data in program PDA.");
        Ok(())
    }

    /* 
    ===========================================================
        deposit_ico_tokens_in_ata function use DepositIcoTokensInATA struct
    ===========================================================
*/
    pub fn deposit_ico_tokens_in_ata(
        ctx: Context<DepositIcoTokensInATA>,
        ico_tokens_amount: u64,
    ) -> ProgramResult {
        if ctx.accounts.ico_data.admin != *ctx.accounts.admin.key {
            msg!("incorrect authority/admin.");
            return Err(ProgramError::IllegalOwner);
        }
        // transfer ICO tokens from admin ata to program ata
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.ico_ata_for_admin.to_account_info(),
                to: ctx.accounts.ico_ata_for_ico_program.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, ico_tokens_amount)?;

        let ico_data = &mut ctx.accounts.ico_data;
        ico_data.ico_tokens_balance += ico_tokens_amount;
        msg!(
            "deposit {} more ICO tokens in program ATA.",
            ico_tokens_amount
        );
        Ok(())
    }

    /* 
    ===========================================================
        withdraw_ico_tokens_from_ata function use WithdrawIcoTokensFromATA struct
    ===========================================================
*/
    pub fn withdraw_ico_tokens_from_ata(ctx: Context<WithdrawIcoTokensFromATA>) -> ProgramResult {
        if ctx.accounts.ico_data.admin != *ctx.accounts.admin.key {
            msg!("incorrect authority/admin.");
            return Err(ProgramError::IllegalOwner);
        }
        // transfer ICO tokens from program ata to admin ata
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.ico_ata_for_ico_program.to_account_info(),
                to: ctx.accounts.ico_ata_for_admin.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            },
        );

        // update pda data
        let ico_data = &mut ctx.accounts.ico_data;
        token::transfer(cpi_ctx, ico_data.ico_tokens_balance)?;
        ico_data.ico_tokens_balance = 0;
        msg!("withdraw ICO from program ATA.");
        Ok(())
    }

    /* 
    ===========================================================
        buy_with_sol function use BuyWithSol struct
    ===========================================================
*/
    pub fn buy_with_sol(
        ctx: Context<BuyWithSol>,
        _ico_ata_for_ico_program_bump: u8,
        ico_tokens_amount: u64,
    ) -> Result<()> {
        // calculate token amount as sol/tokens
        let ico_data = &mut ctx.accounts.ico_data;
        if ico_data.ico_tokens_balance < ico_tokens_amount {
            msg!("program ata don't have enough tokens.");
            return Err(IcoCustomError::InsufficientFundsInIco.into());
        }

        let sol_amount_in_lamport = ico_tokens_amount * ico_data.sol_per_token;

        // transfer sol from user to admin
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.admin.key(),
            sol_amount_in_lamport,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.admin.to_account_info(),
            ],
        )?;
        msg!(
            "transfer {} sol (lamports) to admin.",
            sol_amount_in_lamport
        );

        // transfer ICO from program to user ATA
        let ico_mint_address = ctx.accounts.ico_mint.key();
        let seeds = &[ico_mint_address.as_ref(), &[_ico_ata_for_ico_program_bump]];
        let signer = [&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.ico_ata_for_ico_program.to_account_info(),
                to: ctx.accounts.ico_ata_for_user.to_account_info(),
                authority: ctx.accounts.ico_ata_for_ico_program.to_account_info(),
            },
            &signer,
        );

        token::transfer(cpi_ctx, ico_tokens_amount)?;
        // update ico data
        ico_data.total_sol_recived += sol_amount_in_lamport;
        ico_data.total_ico_tokens_sold += ico_tokens_amount;
        ico_data.ico_tokens_balance -= ico_tokens_amount;
        msg!("transfer {} ico tokens to buyer/user.", ico_tokens_amount);
        Ok(())
    }

    /* 
    ===========================================================
        update_ico_data function use UpdateIcoData struct
    ===========================================================
*/
    pub fn update_ico_token_price(
        ctx: Context<UpdateIcoTokenPrice>,
        sol_per_token: u64,
    ) -> ProgramResult {
        if ctx.accounts.ico_data.admin != *ctx.accounts.admin.key {
            msg!("incorrect authority/admin.");
            return Err(ProgramError::IllegalOwner);
        }
        let ico_data = &mut ctx.accounts.ico_data;
        ico_data.sol_per_token = sol_per_token;
        msg!("update SOL/ICO_Token: {} ", sol_per_token);
        Ok(())
    }

    /* 
    -----------------------------------------------------------
        InitiateAndCreateIcoATA struct for initiate_and_create_ico_ata function
    -----------------------------------------------------------
*/
    #[derive(Accounts)]
    pub struct InitiateAndCreateIcoATA<'info> {
        // 1. PDA (pubkey) for ico ATA for our program.
        // seeds: [ico_mint + current program id] => "HashMap[seeds+bump] = pda"
        // token::mint: Token Program wants to know what kind of token this ATA is for
        // token::authority: It's a PDA so the authority is itself!
        #[account(
        init,
        payer = admin,
        seeds = [ ICO_MINT_ADDRESS.parse::<Pubkey>().unwrap().as_ref() ],
        bump,
        token::mint = ico_mint,
        token::authority = ico_ata_for_ico_program,
        )]
        pub ico_ata_for_ico_program: Account<'info, TokenAccount>,

        #[account(init, payer=admin, space=128, seeds=[b"ico_data"], bump)]
        pub ico_data: Account<'info, IcoData>,

        #[account(
        address = ICO_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
        )]
        pub ico_mint: Account<'info, Mint>,

        #[account(mut)]
        pub ico_ata_for_admin: Account<'info, TokenAccount>,

        #[account(mut)]
        pub admin: Signer<'info>,

        pub system_program: Program<'info, System>,
        pub token_program: Program<'info, Token>,
        pub rent: Sysvar<'info, Rent>,
    }

    /* 
    -----------------------------------------------------------
        DepositIcoInATA struct for deposit_ico_tokens_in_ata function
    -----------------------------------------------------------
*/
    #[derive(Accounts)]
    pub struct DepositIcoTokensInATA<'info> {
        #[account(mut)]
        pub ico_ata_for_ico_program: Account<'info, TokenAccount>,

        #[account(mut)]
        pub ico_data: Account<'info, IcoData>,

        #[account(
        address = ICO_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
        )]
        pub ico_mint: Account<'info, Mint>,

        #[account(mut)]
        pub ico_ata_for_admin: Account<'info, TokenAccount>,

        #[account(mut)]
        pub admin: Signer<'info>,
        pub token_program: Program<'info, Token>,
    }

    /* 
    -----------------------------------------------------------
        WithdrawIcoTokensFromATA struct for withdraw_ico_tokens_from_ata function
    -----------------------------------------------------------
*/
    #[derive(Accounts)]
    pub struct WithdrawIcoTokensFromATA<'info> {
        #[account(mut)]
        pub ico_ata_for_ico_program: Account<'info, TokenAccount>,

        #[account(mut)]
        pub ico_data: Account<'info, IcoData>,

        #[account(
        address = ICO_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
        )]
        pub ico_mint: Account<'info, Mint>,

        #[account(mut)]
        pub ico_ata_for_admin: Account<'info, TokenAccount>,

        #[account(mut)]
        pub admin: Signer<'info>,
        pub token_program: Program<'info, Token>,
    }

    /* 
    -----------------------------------------------------------
        BuyWithSol struct for buy_with_sol function
    -----------------------------------------------------------
*/
    #[derive(Accounts)]
    #[instruction(_ico_ata_for_ico_program_bump: u8)]
    pub struct BuyWithSol<'info> {
        #[account(
        mut,
        seeds = [ ico_mint.key().as_ref() ],
        bump = _ico_ata_for_ico_program_bump,
        )]
        pub ico_ata_for_ico_program: Account<'info, TokenAccount>,

        #[account(mut)]
        pub ico_data: Account<'info, IcoData>,

        #[account(
        address = ICO_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
        )]
        pub ico_mint: Account<'info, Mint>,

        #[account(mut)]
        pub ico_ata_for_user: Account<'info, TokenAccount>,

        #[account(mut)]
        pub user: Signer<'info>,

        /// CHECK:
        #[account(mut)]
        pub admin: AccountInfo<'info>,

        pub token_program: Program<'info, Token>,
        pub system_program: Program<'info, System>,
    }

    /* 
    -----------------------------------------------------------
        UpdateIcoTokenPrice struct for update_ico_token_price function
    -----------------------------------------------------------
*/
    #[derive(Accounts)]
    pub struct UpdateIcoTokenPrice<'info> {
        #[account(mut)]
        pub ico_data: Account<'info, IcoData>,
        #[account(mut)]
        pub admin: Signer<'info>,
        pub system_program: Program<'info, System>,
    }

    /* 
    -----------------------------------------------------------
        IcoData struct for PDA Account
    -----------------------------------------------------------
*/
    #[account]
    pub struct IcoData {
        // sol calculte in lamport
        pub sol_per_token: u64,
        pub total_sol_recived: u64,
        pub total_ico_tokens_sold: u64,
        pub ico_tokens_balance: u64,
        pub admin: Pubkey,
        pub initiate: bool,
    }

    /* 
    -----------------------------------------------------------
        IcoCustomError enum
    -----------------------------------------------------------
*/
}

#[error_code]
pub enum IcoCustomError {
    #[msg("The account has already been initialized.")]
    AccountAlreadyInitialized,
    #[msg("Insufficient funds in ICO.")]
    InsufficientFundsInIco,
}

