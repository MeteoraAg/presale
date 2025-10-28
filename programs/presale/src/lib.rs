#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[macro_use]
pub mod macros;

pub mod errors;
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

declare_id!("presSVxnf9UU8jMxhgSMqaRwNiT36qeBdNeTRKjTdbj");

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    // Required fields
    name: "Presale",
    project_url: "https://meteora.ag/",
    contacts: "email:feedback@raccoons.dev",
    policy: "https://github.com/MeteoraAg/presale",
    // Optional Fields
    preferred_languages: "en",
    source_code: "https://github.com/MeteoraAg/presale"
}

#[program]
pub mod presale {
    use super::*;

    pub fn initialize_prorata_presale<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, InitializePresaleCtx<'info>>,
        params: InitializeProrataPresaleArgs,
        remaining_account_info: RemainingAccountsInfo,
    ) -> Result<()> {
        instructions::handle_initialize_prorata_presale(ctx, params, remaining_account_info)
    }

    pub fn initialize_fixed_price_presale<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, InitializePresaleCtx<'info>>,
        params: InitializeFixedPricePresaleArgs,
        remaining_account_info: RemainingAccountsInfo,
    ) -> Result<()> {
        instructions::handle_initialize_fixed_price_presale(ctx, params, remaining_account_info)
    }

    pub fn initialize_fcfs_presale<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, InitializePresaleCtx<'info>>,
        params: InitializeFcfsPresaleArgs,
        remaining_account_info: RemainingAccountsInfo,
    ) -> Result<()> {
        instructions::handle_initialize_fcfs_presale(ctx, params, remaining_account_info)
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
        params: CreatePermissionedEscrowWithCreatorParams,
    ) -> Result<()> {
        instructions::handle_create_permissioned_escrow_with_creator(ctx, params)
    }

    pub fn create_permissioned_escrow_with_merkle_proof(
        ctx: Context<CreatePermissionedEscrowWithMerkleProofCtx>,
        params: CreatePermissionedEscrowWithMerkleProofParams,
    ) -> Result<()> {
        instructions::handle_create_permissioned_escrow_with_merkle_proof(ctx, params)
    }

    pub fn create_operator(ctx: Context<CreateOperatorCtx>) -> Result<()> {
        instructions::handle_create_operator(ctx)
    }

    pub fn revoke_operator(_ctx: Context<RevokeOperatorCtx>) -> Result<()> {
        Ok(())
    }

    pub fn deposit<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DepositCtx<'info>>,
        max_amount: u64,
        remaining_account_info: RemainingAccountsInfo,
    ) -> Result<()> {
        instructions::handle_deposit(ctx, max_amount, remaining_account_info)
    }

    pub fn withdraw<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, WithdrawCtx<'info>>,
        amount: u64,
        remaining_account_info: RemainingAccountsInfo,
    ) -> Result<()> {
        instructions::handle_withdraw(ctx, amount, remaining_account_info)
    }

    pub fn claim<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, ClaimCtx<'info>>,
        remaining_accounts_info: RemainingAccountsInfo,
    ) -> Result<()> {
        instructions::handle_claim(ctx, remaining_accounts_info)
    }

    pub fn withdraw_remaining_quote<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, WithdrawRemainingQuoteCtx<'info>>,
        remaining_accounts_info: RemainingAccountsInfo,
    ) -> Result<()> {
        instructions::handle_withdraw_remaining_quote(ctx, remaining_accounts_info)
    }

    pub fn perform_unsold_base_token_action<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, PerformUnsoldBaseTokenActionCtx<'info>>,
        remaining_accounts_info: RemainingAccountsInfo,
    ) -> Result<()> {
        instructions::handle_perform_unsold_base_token_action(ctx, remaining_accounts_info)
    }

    pub fn close_escrow(ctx: Context<CloseEscrowCtx>) -> Result<()> {
        instructions::handle_close_escrow(ctx)
    }

    pub fn creator_withdraw<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CreatorWithdrawCtx<'info>>,
        remaining_accounts_info: RemainingAccountsInfo,
    ) -> Result<()> {
        instructions::handle_creator_withdraw(ctx, remaining_accounts_info)
    }

    pub fn refresh_escrow(ctx: Context<RefreshEscrowCtx>) -> Result<()> {
        instructions::handle_refresh_escrow(ctx)
    }

    pub fn create_permissioned_server_metadata(
        ctx: Context<CreatePermissionedServerMetadataCtx>,
        server_url: String,
    ) -> Result<()> {
        instructions::handle_create_permissioned_server_metadata(ctx, server_url)
    }

    pub fn close_permissioned_server_metadata(
        ctx: Context<ClosePermissionedServerMetadataCtx>,
    ) -> Result<()> {
        instructions::handle_close_permissioned_server_metadata(ctx)
    }

    pub fn creator_collect_fee<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CreatorCollectFeeCtx<'info>>,
        remaining_accounts_info: RemainingAccountsInfo,
    ) -> Result<()> {
        instructions::handle_creator_collect_fee(ctx, remaining_accounts_info)
    }
}
