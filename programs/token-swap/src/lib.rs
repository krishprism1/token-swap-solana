use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Token, TokenAccount, Mint, Transfer as SplTransfer };
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::associated_token;
use anchor_lang::solana_program::system_instruction;
use pyth_solana_receiver_sdk::price_update::{ PriceUpdateV2 };
use pyth_solana_receiver_sdk::price_update::get_feed_id_from_hex;
use anchor_lang::solana_program::program::invoke_signed;

declare_id!("BN7BxF5hiBK9v93ieKP5r8g1qbxwDaCTjudY8JAh8cUP");

const MIN_PURCHASE: u64 = 50;
const MAX_PURCHASE: u64 = 5_000_000;

#[derive(Accounts)]
pub struct BuySplWithSol<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [b"state"], bump)]
    pub state: Account<'info, State>,

    #[account(mut, seeds = [b"pda_sol"], bump)]
    pub pda_sol_account: SystemAccount<'info>,

    #[account(mut, seeds = [b"pda_spl_ata"], bump)]
    pub pda_spl_ata: Account<'info, TokenAccount>,

    #[account(mut, address = state.mint)]
    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub user_spl_ata: Account<'info, TokenAccount>,

    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,

    pub price_update: Account<'info, PriceUpdateV2>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuySplWithSpl<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [b"state"], bump)]
    pub state: Account<'info, State>,

    #[account(mut, seeds = [b"pda_spl_ata"], bump)]
    pub pda_spl_ata: Account<'info, TokenAccount>,

    #[account(mut, seeds = [b"pda_usdc_ata"], bump)]
    pub pda_usdc_ata: Account<'info, TokenAccount>,

    #[account(mut, seeds = [b"pda_usdt_ata"], bump)]
    pub pda_usdt_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_ata: Account<'info, TokenAccount>,

    pub user_mint: Account<'info, Mint>,

    #[account(mut, address = state.mint)]
    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub user_spl_ata: Account<'info, TokenAccount>,

    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,

    pub price_update: Account<'info, PriceUpdateV2>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct GetPrice<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub price_update: Account<'info, PriceUpdateV2>,
}

#[account]
pub struct State {
    pub admin: Pubkey,
    pub usdc_mint: Pubkey,
    pub usdt_mint: Pubkey,
    pub mint: Pubkey, // Project SPL token mint
}

#[derive(Accounts)]
pub struct InitializeState<'info> {
    #[account(init, payer = admin, space = 8 + 32 + 32 + 32 + 32, seeds = [b"state"], bump)]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializePdaSplAta<'info> {
    #[account(
        init,
        payer = admin,
        seeds = [b"pda_spl_ata"],
        bump,
        token::mint = mint,
        token::authority = state
    )]
    pub pda_spl_ata: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    #[account(seeds = [b"state"], bump)]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitializePdaUsdcAta<'info> {
    #[account(
        init,
        payer = admin,
        seeds = [b"pda_usdc_ata"],
        bump,
        token::mint = usdc_mint,
        token::authority = state
    )]
    pub pda_usdc_ata: Account<'info, TokenAccount>,
    pub usdc_mint: Account<'info, Mint>,
    #[account(seeds = [b"state"], bump)]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitializePdaUsdtAta<'info> {
    #[account(
        init,
        payer = admin,
        seeds = [b"pda_usdt_ata"],
        bump,
        token::mint = usdt_mint,
        token::authority = state
    )]
    pub pda_usdt_ata: Account<'info, TokenAccount>,
    pub usdt_mint: Account<'info, Mint>,
    #[account(seeds = [b"state"], bump)]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut, seeds = [b"state"], bump)]
    pub state: Account<'info, State>,

    #[account(mut, associated_token::mint = state.mint, associated_token::authority = admin)]
    pub admin_ata: Account<'info, TokenAccount>,

    #[account(mut, seeds = [b"pda_spl_ata"], bump)]
    pub pda_spl_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"state"], bump)]
    pub state: Account<'info, State>,

    #[account(mut, associated_token::mint = state.mint, associated_token::authority = admin)]
    pub admin_ata: Account<'info, TokenAccount>,
    #[account(mut, associated_token::mint = state.usdc_mint, associated_token::authority = admin)]
    pub admin_usdc_ata: Account<'info, TokenAccount>,

    #[account(mut, associated_token::mint = state.usdt_mint, associated_token::authority = admin)]
    pub admin_usdt_ata: Account<'info, TokenAccount>,

    #[account(mut, seeds = [b"pda_spl_ata"], bump)]
    pub pda_spl_ata: Account<'info, TokenAccount>,

    #[account(mut, seeds = [b"pda_usdc_ata"], bump)]
    pub pda_usdc_ata: Account<'info, TokenAccount>,

    #[account(mut, seeds = [b"pda_usdt_ata"], bump)]
    pub pda_usdt_ata: Account<'info, TokenAccount>,

    #[account(mut, seeds = [b"pda_sol"], bump)]
    pub pda_sol_account: SystemAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAdmin<'info> {
    #[account(mut, seeds = [b"state"], bump)]
    pub state: Account<'info, State>,

    #[account(mut)]
    pub current_admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[program]
pub mod token_swap {
    use super::*;

    pub fn initialize_state(
        ctx: Context<InitializeState>,
        usdc_mint: Pubkey,
        usdt_mint: Pubkey,
        mint: Pubkey
    ) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.admin = *ctx.accounts.admin.key;
        state.usdc_mint = usdc_mint;
        state.usdt_mint = usdt_mint;
        state.mint = mint;
        Ok(())
    }

    pub fn initialize_pda_spl_ata(ctx: Context<InitializePdaSplAta>) -> Result<()> {
        msg!("PDA SPL ATA initialized: {}", ctx.accounts.pda_spl_ata.key());
        Ok(())
    }

    pub fn initialize_pda_usdc_ata(ctx: Context<InitializePdaUsdcAta>) -> Result<()> {
        msg!("PDA USDC ATA initialized: {}", ctx.accounts.pda_usdc_ata.key());
        Ok(())
    }

    pub fn initialize_pda_usdt_ata(ctx: Context<InitializePdaUsdtAta>) -> Result<()> {
        msg!("PDA USDT ATA initialized: {}", ctx.accounts.pda_usdt_ata.key());
        Ok(())
    }

    pub fn update_admin(ctx: Context<UpdateAdmin>, new_admin: Pubkey) -> Result<()> {
        let state = &mut ctx.accounts.state;
        require_keys_eq!(state.admin, ctx.accounts.current_admin.key(), CustomError::Unauthorized);

        state.admin = new_admin;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), SplTransfer {
            from: ctx.accounts.admin_ata.to_account_info(), // From admin_spl_ata
            to: ctx.accounts.pda_spl_ata.to_account_info(), // To pda_spl_ata
            authority: ctx.accounts.admin.to_account_info(), // Project owner's signature
        });

        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let state = &ctx.accounts.state;
        require_keys_eq!(state.admin, ctx.accounts.admin.key(), CustomError::Unauthorized);

        let seeds = &[b"state".as_ref(), &[ctx.bumps.state]];
        let signer = &[&seeds[..]];

        let pda_sol_balance = ctx.accounts.pda_sol_account.lamports();
        if pda_sol_balance > 0 {
            let transfer_instruction = system_instruction::transfer(
                &ctx.accounts.pda_sol_account.key(), // PDA
                &ctx.accounts.admin.key(), // Admin
                pda_sol_balance
            );

            invoke_signed(
                &transfer_instruction,
                &[
                    ctx.accounts.pda_sol_account.to_account_info(),
                    ctx.accounts.admin.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[&[b"pda_sol", &[ctx.bumps.pda_sol_account]]] 
            )?;
        }

        let pda_spl_balance = ctx.accounts.pda_spl_ata.amount;
        if pda_spl_balance > 0 {
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                SplTransfer {
                    from: ctx.accounts.pda_spl_ata.to_account_info(),
                    to: ctx.accounts.admin_ata.to_account_info(),
                    authority: ctx.accounts.state.to_account_info(),
                },
                signer
            );

            token::transfer(cpi_ctx, pda_spl_balance)?;
        }

        // Transfer USDC
        let usdc_balance = ctx.accounts.pda_usdc_ata.amount;
        if usdc_balance > 0 {
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                SplTransfer {
                    from: ctx.accounts.pda_usdc_ata.to_account_info(),
                    to: ctx.accounts.admin_usdc_ata.to_account_info(),
                    authority: ctx.accounts.state.to_account_info(),
                },
                signer
            );
            token::transfer(cpi_ctx, usdc_balance)?;
        }

        // Transfer USDT
        let usdt_balance = ctx.accounts.pda_usdt_ata.amount;
        if usdt_balance > 0 {
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                SplTransfer {
                    from: ctx.accounts.pda_usdt_ata.to_account_info(),
                    to: ctx.accounts.admin_usdt_ata.to_account_info(),
                    authority: ctx.accounts.state.to_account_info(),
                },
                signer
            );
            token::transfer(cpi_ctx, usdt_balance)?;
        }

        Ok(())
    }

    pub fn buy_spl_with_sol(ctx: Context<BuySplWithSol>, lamports_to_pay: u64) -> Result<()> {
        let spl_precision = (10_u64).pow(ctx.accounts.mint.decimals as u32);

        let spl_price_in_usd = 0.02f64;
        let lamports_per_sol = 1_000_000_000u64;

        let price_update = &mut ctx.accounts.price_update;
        let maximum_age: u64 = 60;
        let feed_id: [u8; 32] = get_feed_id_from_hex(
            "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d"
        )?;
        let price = price_update.get_price_no_older_than(&Clock::get()?, maximum_age, &feed_id)?;
        let sol_price_in_usd: f64 = (price.price as f64) * (10f64).powi(price.exponent);

        let sol_amount = (lamports_to_pay as f64) / (lamports_per_sol as f64);
        let user_pay_in_usd = sol_amount * sol_price_in_usd;
        let spl_amount_float = (user_pay_in_usd / spl_price_in_usd) * (spl_precision as f64);
        let spl_amount: u64 = spl_amount_float.floor() as u64;

        if spl_amount < MIN_PURCHASE * spl_precision {
            return Err(CustomError::PurchaseAmountTooLow.into());
        }

        if spl_amount > MAX_PURCHASE * spl_precision {
            return Err(CustomError::PurchaseAmountTooHigh.into());
        }

        if ctx.accounts.pda_spl_ata.amount < spl_amount {
            return Err(CustomError::InsufficientSPLBalance.into());
        }

        let user_signer = &ctx.accounts.user;
        let system_program = &ctx.accounts.system_program;

        let transfer_instruction = system_instruction::transfer(
            user_signer.key,
            &ctx.accounts.pda_sol_account.key,
            lamports_to_pay
        );

        anchor_lang::solana_program::program::invoke(
            &transfer_instruction,
            &[
                user_signer.to_account_info(),
                ctx.accounts.pda_sol_account.to_account_info(),
                system_program.to_account_info(),
            ]
        )?;

        let seeds = &[b"state".as_ref(), &[ctx.bumps.state]];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SplTransfer {
                from: ctx.accounts.pda_spl_ata.to_account_info(),
                to: ctx.accounts.user_spl_ata.to_account_info(),
                authority: ctx.accounts.state.to_account_info(),
            },
            signer
        );
        token::transfer(cpi_ctx, spl_amount)?;
        Ok(())
    }

    pub fn buy_spl_with_spl(ctx: Context<BuySplWithSpl>, token_amount: u64) -> Result<()> {
        let spl_precision = (10_u64).pow(ctx.accounts.mint.decimals as u32);

        let spl_price_in_usd = 0.02_f64;
        let decimals = 1_000_000u64;
        let maximum_age: u64 = 90;
        const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        const USDT_MINT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";

        let user_mint_key = ctx.accounts.user_mint.key().to_string();

        let feed_ids = match user_mint_key.as_str() {
            USDT_MINT => Some("0x2b89b9dc8fdf9f34709a5b106b472f0f39bb6ca9ce04b0fd7f2e971688e2e53b"),
            USDC_MINT => Some("0xeaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a"),
            _ => None,
        };

        let price_update = &mut ctx.accounts.price_update;

        let feed_id: [u8; 32] = match feed_ids {
            Some(id) => get_feed_id_from_hex(id)?,
            None => {
                return Err(CustomError::InvalidMint.into());
            }
        };

        let price = price_update.get_price_no_older_than(&Clock::get()?, maximum_age, &feed_id)?;
        let usdc_price_in_usd: f64 = (price.price as f64) * (10f64).powi(price.exponent);

        let spl_amount_float =
            ((token_amount as f64) / (decimals as f64) / spl_price_in_usd) * (spl_precision as f64);

        let spl_amount: u64 = spl_amount_float.floor() as u64;

        if spl_amount < MIN_PURCHASE * spl_precision {
            return Err(CustomError::PurchaseAmountTooLow.into());
        }

        if spl_amount > MAX_PURCHASE * spl_precision {
            return Err(CustomError::PurchaseAmountTooHigh.into());
        }

        if ctx.accounts.pda_spl_ata.amount < spl_amount {
            return Err(CustomError::InsufficientSPLBalance.into());
        }

        let to_account_info = match user_mint_key.as_str() {
            USDC_MINT => ctx.accounts.pda_usdc_ata.to_account_info(),
            USDT_MINT => ctx.accounts.pda_usdt_ata.to_account_info(),
            _ => {
                return Err(CustomError::InvalidMint.into());
            }
        };

        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), SplTransfer {
            from: ctx.accounts.user_token_ata.to_account_info(),
            to: to_account_info,
            authority: ctx.accounts.user.to_account_info(),
        });
        token::transfer(cpi_ctx, token_amount)?;

        let seeds = &[b"state".as_ref(), &[ctx.bumps.state]];
        let signer = &[&seeds[..]];

        let cpi_ctx_spl_transfer = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SplTransfer {
                from: ctx.accounts.pda_spl_ata.to_account_info(),
                to: ctx.accounts.user_spl_ata.to_account_info(),
                authority: ctx.accounts.state.to_account_info(),
            },
            signer
        );
        token::transfer(cpi_ctx_spl_transfer, spl_amount)?;
        Ok(())
    }
}

#[error_code]
pub enum CustomError {
    #[msg("Not enough SPL tokens in project wallet.")]
    InsufficientSPLBalance,
    #[msg("The purchase amount is below the minimum limit.")]
    PurchaseAmountTooLow,
    #[msg("The purchase amount exceeds the maximum limit.")]
    PurchaseAmountTooHigh,
    #[msg("Invalid USDC/USDT mint address.")]
    InvalidMint,
    #[msg("Unauthorized Access")]
    Unauthorized,
}
