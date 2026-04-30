Title: `init_market_rewards` accepts non-Percolator slabs

## Summary

`rewards::init_market_rewards` on `338c31d` accepted byte-shaped fake slabs. It checked that the slab did not have an all-zero magic value and that the admin bytes were burned, but did not require the slab account to be owned by the Percolator program, did not require the exact Percolator magic, and did not bind the slab config collateral mint to the rewards collateral mint.

This contradicts the spec's market isolation model: each market has its own collateral mint and isolated vault (`spec.md:51`), proposal payloads commit to collateral mint and market config (`spec.md:76-83`), and `MarketRewardsCfg.market_slab` is a Percolator slab account (`spec.md:228`).

## Vulnerable Path

On base `338c31d`, rewards checked only the slab header magic nonzero condition and the burned admin bytes before creating the MRC (`DESIGN_VERIFICATION.md:13`). It did not check `market_slab.owner == percolator_prog::id()`, exact magic, or `header.config.collateral_mint == collateral_mint.key` (`DESIGN_VERIFICATION.md:13`).

## Proof

Regression tests: `program/tests/critical_pocs.rs:579`, `program/tests/critical_pocs.rs:603`, and `program/tests/critical_pocs.rs:633`.

After the fix, authorized adapter calls with a non-Percolator-owned slab, bad Percolator magic, and a collateral-mint mismatch all fail before MRC creation:

```text
O1 fixed: authorized_init_result=Err("FailedTransactionMetadata { err: InstructionError(0, IllegalOwner), meta: TransactionMetadata { signature: 3pXsrQHa9TCNyP15vHj1eNjAnSKEN7J4NSQicqPRnMi4MbheRcdkZT3qA4p69m4WHfJkPenHqGEqWRHDxfqh5bzv, logs: [\"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]\", \"Program 111111131h1vYVSYuKP6AhS86fbRdMw9XHiZAvAaj invoke [2]\", \"Program log: Market slab must be owned by Percolator\", \"Program 111111131h1vYVSYuKP6AhS86fbRdMw9XHiZAvAaj consumed 11108 of 191443 compute units\", \"Program 111111131h1vYVSYuKP6AhS86fbRdMw9XHiZAvAaj failed: Provided owner is not allowed\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 19665 of 200000 compute units\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: Provided owner is not allowed\"], compute_units_consumed: 19665, return_data: TransactionReturnData { program_id: 111111131h1vYVSYuKP6AhS86fbRdMw9XHiZAvAaj, data: [] } } }") slab_owner_is_percolator=false mrc_exists=false
O1 magic fixed: authorized_init_result=Err("FailedTransactionMetadata { err: InstructionError(0, InvalidAccountData), meta: TransactionMetadata { signature: 5ktF8aPWk6NipqRi6PdqUoBToLhV5ntfUNKg8wmxhJuaJ4dXmLdjFfe5LaJi1oTwmFqQ81RzaSBgSsrWrMW8Zk5A, logs: [\"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]\", \"Program 11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3 invoke [2]\", \"Program log: Percolator slab magic mismatch\", \"Program 11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3 consumed 11170 of 191443 compute units\", \"Program 11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3 failed: invalid account data for instruction\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 19727 of 200000 compute units\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: invalid account data for instruction\"], compute_units_consumed: 19727, return_data: TransactionReturnData { program_id: 11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3, data: [] } } }") slab_magic_is_exact=false mrc_exists=false
O1 collateral fixed: authorized_init_result=Err("FailedTransactionMetadata { err: InstructionError(0, InvalidAccountData), meta: TransactionMetadata { signature: P883zbqjM6syGVcbWcLRwrXSCVp7qZsETnhdAyNbgcArcHV7tak3xGHaCcuiNMpGpjBsqbfRNbJntJfzvEkk33A, logs: [\"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]\", \"Program 11111113pNDtm61yGF8j2ycAwLEPsuWQXobye5qDR invoke [2]\", \"Program log: Percolator slab collateral mint mismatch\", \"Program 11111113pNDtm61yGF8j2ycAwLEPsuWQXobye5qDR consumed 12700 of 186943 compute units\", \"Program 11111113pNDtm61yGF8j2ycAwLEPsuWQXobye5qDR failed: invalid account data for instruction\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 25757 of 200000 compute units\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: invalid account data for instruction\"], compute_units_consumed: 25757, return_data: TransactionReturnData { program_id: 11111113pNDtm61yGF8j2ycAwLEPsuWQXobye5qDR, data: [] } } }") slab_collateral_matches=false mrc_exists=false
test test_o1_init_market_rewards_rejects_non_percolator_slab ... ok
```

Full output: `verification/04-critical-pocs.log:839-844`.

## Severity

High. This weakens market isolation and lets a governed init bind rewards to a fake slab instead of a real Percolator market. The impact compounds with adapter authorization bugs, but the slab-binding issue is independent and should be fixed even for authorized callers.

## Fix

Fixed in `c5994a4f4b21b257606f8748e1d47331910970d1`.

The new `load_percolator_market` helper requires Percolator ownership, sufficient slab data, exact Percolator magic, and matching collateral mint (`program/src/lib.rs:288-315`). `init_market_rewards` now calls it before MRC creation and before writing reward config (`program/src/lib.rs:582-587`).
