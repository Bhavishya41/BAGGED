use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Burn, Mint, TokenAccount, TokenInterface};

declare_id!("BcR28Y2BnLcmzhCPqst3ZeMo8SAaUo72iWwiRu6PiSNg");

#[program]
pub mod redeem {
    use super::*;

    // 1. Initialize a Perk (The "Coupon" Setup)
    pub fn initialize_perk(ctx: Context<InitializePerk>, perk_id: u64, price: u64, supply: u32) -> Result<()> {
        let perk = &mut ctx.accounts.perk;
        perk.creator = ctx.accounts.creator.key();
        perk.perk_id = perk_id;
        perk.price = price;
        perk.supply = supply;
        perk.redeemed_count = 0;
        
        msg!("Perk {} initialized. Price: {} tokens. Supply: {}", perk_id, price, supply);
        Ok(())
    }

    // 2. Redeem a Perk (The "Burn" Logic)
    pub fn redeem_perk(ctx: Context<RedeemPerk>) -> Result<()> {
        let perk = &mut ctx.accounts.perk;
        
        require!(perk.redeemed_count < perk.supply, ErrorCode::SoldOut);

        // THE FIX: We define the accounts first
        let cpi_accounts = Burn {
            mint: ctx.accounts.token_mint.to_account_info(),
            from: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        
        // THE FIX: In the new Anchor version, pass the .key() instead of to_account_info()
        // and wrap it in the token_interface helper
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.key(), cpi_accounts);
        
        token_interface::burn(cpi_ctx, perk.price)?;

        perk.redeemed_count += 1;
        
        emit!(RedemptionEvent {
            user: ctx.accounts.user.key(),
            perk_id: perk.perk_id,
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(perk_id: u64)]
pub struct InitializePerk<'info> {
    #[account(
        init, 
        payer = creator, 
        space = 8 + 32 + 8 + 8 + 4 + 4, 
        seeds = [b"perk", creator.key().as_ref(), &perk_id.to_le_bytes()], 
        bump
    )]
    pub perk: Account<'info, Perk>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RedeemPerk<'info> {
    #[account(mut)]
    pub perk: Account<'info, Perk>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub token_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[account]
pub struct Perk {
    pub creator: Pubkey,
    pub perk_id: u64,
    pub price: u64,
    pub supply: u32,
    pub redeemed_count: u32,
}

#[event]
pub struct RedemptionEvent {
    pub user: Pubkey,
    pub perk_id: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("This perk is sold out.")]
    SoldOut,
}