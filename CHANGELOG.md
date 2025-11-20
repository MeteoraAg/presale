# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

# presale [0.1.1] [PR #31](https://github.com/MeteoraAg/presale/pull/31)

### Fixed

- Calculation of vesting_start_time, vesting_end_time, lock_start_time, and lock_end_time.

### Added

- Add option to allow fixed price presale to disable withdraw
- Add option to allow fixed price, and fcfs presale to not end presale earlier if cap reached
- Close of `MerkleRoofConfig` account.

### Changed

- Reserve presale_mode_raw_data: [u128; 3] which used for storing different presale mode data. For example: price for fixed mode
- Removed lock_start_time and lock_end_time field since they can be represented by presale_end_time and vesting_start_time and not used anywhere.
- Add security_txt!
- When it's permissioned mode, dynamic price presale buyer_minimum_cap must be 1, and buyer_maximum_cap must be presale_maximum_cap. For fixed price presale, buyer_minimum_cap must be minimum possibly quote token, while buyer_maximum_cap must be presale_maximum_cap. This reduces user mistake on creating whitelist wallet cap that couldn't create any escrow account.
