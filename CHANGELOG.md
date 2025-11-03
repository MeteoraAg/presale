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

# presale [0.2.0] [PR #31](https://github.com/MeteoraAg/presale/pull/31)

### Fixed

- Calculation of vesting_start_time, vesting_end_time, lock_start_time, and lock_end_time.

### Added

- Add option to allow fixed price presale to disable withdraw
- Add option to allow fixed price, and fcfs presale to not end presale earlier if cap reached

### Removed

- Break: `initialize_fixed_price_presale_args` endpoint
- Break: `close_fixed_price_presale_args` endpoint

### Changed

- Break: `initialize_presale` has been removed and separated into 3 different endpoint, which are `initialize_prorata_presale`, `initialize_fixed_price_presale` and `initialize_fcfs_presale`
- Break: Newly added `immediate_release_timestamp` need to be `>= presale_end_time`
- Break: 