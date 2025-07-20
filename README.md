# Presale Smart Contract

This is an Anchor-based smart contract for conducting a token presale on the Solana blockchain. It enables users to contribute using any token and claim a specified amount of presale tokens after the presale ends. The presale is considered failed if the minimum cap is not reached. Additionally, lock and vesting schedules can be configured for claiming.

The program does not manage the deployment of liquidity for the raised capital. This means the presale creator must manually withdraw the raised funds and deploy the liquidity themselves.

## Presale configuration

| Name                      | Description                                                                                                                   | Remark                                                   |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- |
| presale_mint              | Mint of the presale                                                                                                           | The program will transfer the presale token from creator |
| presale_pool_supply       | token supply for sale                                                                                                         |                                                          |
| presale_maximum_cap       | Maximum presale fund to raise                                                                                                 |                                                          |
| presale_minimum_cap       | Minimum presale fund to raise. If the fund raised is below presale_minimum_cap threshold, the presale is considered as failed |
| buyer_minimum_deposit_cap | Minimum deposit amount from buyer                                                                                             |                                                          |
| buyer_maximum_deposit_cap | Maximum deposit amount from buyer                                                                                             |                                                          |
| presale_start_time        | When does the presale start                                                                                                   |                                                          |
| presale_end_time          | When does the presale end                                                                                                     |                                                          |
| whitelist_mode            | Permissionless, permissioned with authority, and permissioned with merkle tree                                                |                                                          |
| presale_mode              | Fixed price, FCFS, and prorata                                                                                                |                                                          |
| lock_duration             | Buyer token lock duration                                                                                                     |                                                          |
| vest_duration             | Buyer token vest duration                                                                                                     |

## Modes

### Fixed Price

- Tokens are sold at a fixed price.
- The presale ends early if the maximum cap is reached before the scheduled end time.
- The creator can choose what to do with any unsold tokens: either transfer them back or burn them.

### FCFS (First Come, First Served)

- The token price is dynamically determined by the amount of capital raised, calculated as `quote_token_amount / presale_base_token_amount`.
- The presale ends early if the cap is reached before the scheduled end time.
- There will be no unsold tokens.

### Prorata

- The token price is dynamically determined by the total capital raised, calculated as `quote_token_amount / presale_base_token_amount`.
- The presale can be oversubscribed.
- Any oversubscribed amount will be refunded to users once the presale ends.
- There will be no unsold tokens.
