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

## presale [0.0.4] [PR #8](https://github.com/MeteoraAg/presale-sdk/pull/8)

### Fixed

- Presale progress end timing

### Added

- Allow integrator to disable compute unit instruction prepend by using `setComputeUnitOptimization`

## presale [0.0.3] [PR #6](https://github.com/MeteoraAg/presale-sdk/pull/7)

### Fixed

- `uiPriceToQPrice` conversion
- Minor bug fixes: avoiding zero division, null registryIndex

## presale [0.0.2] [PR #6](https://github.com/MeteoraAg/presale-sdk/pull/6)

### Fixed

- Export `Presale` class
