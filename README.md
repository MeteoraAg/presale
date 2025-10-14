# Meteora Presale

An Anchor-based Solana smart contract designed for managing token presales. This program allows users to contribute using any SPL token and later claim their allocated presale tokens once the presale concludes.

## Note

The program does not manage the deployment of liquidity for the raised capital. This means the presale creator must manually withdraw the raised funds and deploy the liquidity themselves.

## Program ID

The same program ID applies to both mainnet and devnet:

```
presSVxnf9UU8jMxhgSMqaRwNiT36qeBdNeTRKjTdbj
```

## Key features

💱 SPL Token Support

- Supports contributions using any SPL token, including SPL Token 2022.

🧾 Multiple User Buckets

- Each presale can define multiple user registries (“buckets”), each with its own minimum and maximum deposit caps per buyer.
- Maximum up to 5 user registries

⚙️ Multiple Presale Modes

- Choose from Fixed Price, Prorata, or First-Come, First-Serve (FCFS) presale types.

🔐 Flexible Access Control

- Supports both permissioned (whitelisted) and permissionless presale configurations.

⏳ Comprehensive Locking & Vesting

- Offers full or partial locking and vesting schedules. With partial locking, a portion of tokens is released immediately, while the remaining tokens are locked and gradually vested over time.

## Presale configuration

| Name                    | Description                                                                                                                                  | Remarks                                                    |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| **presale_mint**        | The mint address of the presale token.                                                                                                       | The program will transfer presale tokens from the creator. |
| **presale_maximum_cap** | Maximum amount of funds that can be raised during the presale.                                                                               |                                                            |
| **presale_minimum_cap** | Minimum amount of funds required for the presale to succeed. If the amount raised is below this threshold, the presale is considered failed. |                                                            |
| **presale_start_time**  | Timestamp indicating when the presale starts.                                                                                                |                                                            |
| **presale_end_time**    | Timestamp indicating when the presale ends.                                                                                                  |                                                            |
| **unsold_token_action** | Defines how unsold tokens are handled after the presale — either burned or refunded to the creator.                                          |                                                            |
| **whitelist_mode**      | Defines access control: _permissionless_, _permissioned with authority_, or _permissioned with Merkle tree_.                                 |                                                            |
| **lock_duration**       | Duration for which purchased tokens remain locked.                                                                                           |                                                            |
| **vest_duration**       | Duration over which tokens are gradually vested and released.                                                                                |                                                            |

## Presale registry configuration

| Name                          | Description                                                       | Remarks |
| ----------------------------- | ----------------------------------------------------------------- | ------- |
| **presale_pool_supply**       | Total token supply allocated for the current registry (“bucket”). |         |
| **buyer_minimum_deposit_cap** | Minimum amount a buyer is allowed to deposit.                     |         |
| **buyer_maximum_deposit_cap** | Maximum amount a buyer is allowed to deposit.                     |         |
| **deposit_fee_bps**           | Deposit fee charged to buyers, expressed in basis points (bps).   |         |

## Presale Modes

### Fixed Price

- Tokens are sold at a fixed price.
- The presale ends early if the maximum cap is reached before the scheduled end time.

### FCFS (First Come, First Served)

- The token price is dynamically determined by the amount of capital raised, calculated as `quote_token_amount / presale_base_token_amount`.
- The presale ends early if the cap is reached before the scheduled end time.

### Prorata

- The token price is dynamically determined by the total capital raised, calculated as `quote_token_amount / presale_base_token_amount`.
- The presale can be oversubscribed.
- Any oversubscribed amount will be refunded to users once the presale ends.

## Instructions reference

| **Name**                                         | **Description**                                                                                                                                                      | **Remarks**                                                          |
| ------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| **initialize_fixed_price_presale_args**          | Creates a fixed-price presale args account to store configuration data required for initializing a fixed-price presale.                                              |                                                                      |
| **close_fixed_price_presale_args**               | Closes the fixed-price presale args account.                                                                                                                         |                                                                      |
| **initialize_presale**                           | Initializes a new presale.                                                                                                                                           |                                                                      |
| **create_merkle_root_config**                    | Creates a Merkle root configuration account.                                                                                                                         | Only used for **Merkle proof–based permissioned** presales.          |
| **create_permissionless_escrow**                 | Creates an escrow account for a buyer.                                                                                                                               | Only for **permissionless** mode.                                    |
| **create_permissioned_escrow_with_creator**      | Creates an escrow account authorized by the presale creator.                                                                                                         | Only for **permissioned with authority** mode.                       |
| **create_permissioned_escrow_with_merkle_proof** | Creates an escrow account verified via Merkle proof.                                                                                                                 | Only for **permissioned with Merkle proof** mode.                    |
| **create_operator**                              | Whitelists a wallet as an operator authorized to sign escrow creation transactions.                                                                                  | Only for **permissioned with authority** mode.                       |
| **revoke_operator**                              | Revokes a previously whitelisted operator.                                                                                                                           |                                                                      |
| **deposit**                                      | Deposits funds into the escrow account. In **fixed-price** mode, the deposit amount is automatically **rounded down** to the nearest purchasable unit.               |                                                                      |
| **withdraw**                                     | Withdraws deposited funds from the escrow account. In **fixed-price** mode, the withdrawal amount is automatically **rounded down** to the nearest purchasable unit. |                                                                      |
| **claim**                                        | Claims purchased presale tokens.                                                                                                                                     |                                                                      |
| **withdraw_remaining_quote**                     | Withdraws any unused or oversubscribed deposit amount.                                                                                                               | Only for **prorata** mode.                                           |
| **perform_unsold_base_token_action**             | Executes the configured action (**burn** or **refund**) for unsold base tokens after presale completion.                                                             |                                                                      |
| **close_escrow**                                 | Closes the escrow account.                                                                                                                                           |                                                                      |
| **creator_withdraw**                             | Allows the presale creator to withdraw the raised funds.                                                                                                             |                                                                      |
| **refresh_escrow**                               | Refreshes the escrow account to update the latest claimable token amount.                                                                                            |                                                                      |
| **create_permissioned_server_metadata**          | Creates a permissioned server metadata account to store the server URL used for retrieving Merkle proofs or partially signed escrow creation transactions.           | Only for **permissioned with authority** and **Merkle proof** modes. |
| **close_permissioned_server_metadata**           | Closes the permissioned server metadata account.                                                                                                                     |                                                                      |
| **creator_collect_fee**                          | Allows the presale creator to withdraw collected fees.                                                                                                               |                                                                      |

## Dependencies

| Name   | Version |
| ------ | ------- |
| Rust   | 1.87.0  |
| Anchor | 0.31.1  |
| Solana | 2.1.0   |

## SDK

Please refer to https://github.com/MeteoraAg/presale-sdk
