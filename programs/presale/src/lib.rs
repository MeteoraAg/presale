#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

#[macro_use]
pub mod macros;

mod errors;
use errors::*;

mod instructions;
pub use instructions::*;

mod const_pda;
pub use const_pda::*;

mod constants;
pub use constants::*;

mod state;
pub use state::*;

mod events;
use events::*;

mod math;
pub use math::*;

mod token2022;
pub use token2022::*;

mod presale_mode_handler;
pub use presale_mode_handler::*;

declare_id!("Ff7Lo7AsVxB4VtJH2Ajm7KLLVaVTGMV1W3ws2o5Eo2UT");

#[program]
pub mod presale {
    use super::*;

    pub fn initialize_fixed_price_presale_args(
        ctx: Context<InitializeFixedPricePresaleArgsCtx>,
        params: InitializeFixedPricePresaleExtraArgs,
    ) -> Result<()> {
        instructions::handle_initialize_fixed_price_presale_args(ctx, params)
    }

    pub fn close_fixed_price_presale_args(
        _ctx: Context<CloseFixedPricePresaleArgsCtx>,
    ) -> Result<()> {
        Ok(())
    }

    pub fn initialize_presale<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, InitializePresaleCtx<'info>>,
        params: InitializePresaleArgs,
    ) -> Result<()> {
        instructions::handle_initialize_token_and_create_presale_vault(ctx, &params)
    }

    pub fn initialize_presale_token2022<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, InitializePresaleToken2022Ctx<'info>>,
        params: InitializePresaleArgs,
    ) -> Result<()> {
        instructions::handle_initialize_presale_token2022(ctx, &params)
    }

    pub fn create_merkle_root_config(
        ctx: Context<CreateMerkleRootConfigCtx>,
        params: CreateMerkleRootConfigParams,
    ) -> Result<()> {
        instructions::handle_create_merkle_root_config(ctx, params)
    }

    pub fn create_permissionless_escrow(ctx: Context<CreatePermissionlessEscrowCtx>) -> Result<()> {
        instructions::handle_create_permissionless_escrow(ctx)
    }

    pub fn create_permissioned_escrow_with_creator(
        ctx: Context<CreatePermissionedEscrowWithCreatorCtx>,
    ) -> Result<()> {
        instructions::handle_create_permissioned_escrow_with_creator(ctx)
    }

    pub fn create_permissioned_escrow_with_merkle_proof(
        ctx: Context<CreatePermissionedEscrowWithMerkleProofCtx>,
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        instructions::handle_create_permissioned_escrow_with_merkle_proof(ctx, proof)
    }

    pub fn deposit(ctx: Context<DepositCtx>, max_amount: u64) -> Result<()> {
        instructions::handle_deposit(ctx, max_amount)
    }

    pub fn withdraw(ctx: Context<WithdrawCtx>, amount: u64) -> Result<()> {
        instructions::handle_withdraw(ctx, amount)
    }
}
