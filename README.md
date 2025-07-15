# Presale Smart Contract

This is an Anchor-based smart contract for conducting a token presale on the Solana blockchain. It enables users to contribute using any token and claim a specified amount of presale tokens after the presale ends. The presale is considered failed if the minimum cap is not reached. Additionally, lock and vesting schedules can be configured for claiming.

The program does not manage the deployment of liquidity for the raised capital. This means the presale creator must manually withdraw the raised funds and deploy the liquidity themselves.

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
