# Summary

This PR fixes the findings classified as REAL BUG in `DESIGN_VERIFICATION.md:17-23`: F1, F2, F3, and O1.

- Gate governance adapter signing with a bootstrap controller derived from the current COIN mint authority. `init_authority` must be signed before the SPL Token mint-authority handoff, and later governed CPI routes require that stored controller signer (`governance/src/lib.rs:133-150`, `governance/src/lib.rs:171-226`, `governance/src/lib.rs:300-301`, `governance/src/lib.rs:369-370`).
- Bind `pull_insurance` and market reward initialization to real Percolator market state. The rewards program now validates Percolator program id, slab owner, slab magic, collateral mint, MRC slab binding, rewards stake vault, Percolator vault account, and Percolator vault authority PDA (`program/src/lib.rs:280-340`, `program/src/lib.rs:582-587`, `program/src/lib.rs:1202-1231`).
- Add LiteSVM regression coverage for F1, F2, F3, and O1 in `program/tests/critical_pocs.rs:490`, `program/tests/critical_pocs.rs:545`, `program/tests/critical_pocs.rs:579`, `program/tests/critical_pocs.rs:603`, `program/tests/critical_pocs.rs:633`, `program/tests/critical_pocs.rs:666`, and `program/tests/critical_pocs.rs:733`, with the malicious callee under `malicious-drain/src/lib.rs:19` and its SPL Token transfer at `malicious-drain/src/lib.rs:43-59`.
- Update the integration harness for the fixed Percolator program id, the current Percolator `InitMarket` wire layout, and the new adapter bootstrap ceremony (`program/tests/integration.rs:30`, `program/tests/integration.rs:150-190`, `program/tests/integration.rs:382`, `program/tests/integration.rs:480-523`).
- Clarify the authority bootstrap ceremony in the spec (`spec.md:13-14`, `spec.md:101`).

# Commits

- `b495f1b5e82b6fc92d4c9da19af1e4ab7fc5835a` - Gate adapter signing with bootstrap controller
- `c5994a4f4b21b257606f8748e1d47331910970d1` - Bind rewards CPI paths to Percolator market state
- Final branch artifact commit - adds regression tests, malicious PoC BPF, verification logs, issue drafts, and filing plan.

# Issues

Closes #<TBD-F1>
Closes #<TBD-F2>
Closes #<TBD-F3>
Closes #<TBD-O1>

F4 is not included because current master has no `mint_reward` route (`DESIGN_VERIFICATION.md:12`). O2/O3 are not included as standalone vulnerabilities because they did not meet the REAL BUG threshold after design verification (`DESIGN_VERIFICATION.md:14-15`).

# Verification

Full logs are in `verification/`.

```text
cargo build-sbf --manifest-path governance/Cargo.toml
cargo build-sbf --manifest-path program/Cargo.toml
cargo build-sbf --manifest-path malicious-drain/Cargo.toml
cargo test --test critical_pocs -- --nocapture
cargo check --workspace
cargo test --workspace --lib
git diff --check
```

`cargo test --test critical_pocs -- --nocapture` result from `verification/04-critical-pocs.log:832-850`:

```text
running 8 tests
test test_f4_governance_bypass_attacker_mints_reward_coin ... ignored, F4 was proven only on the local mint_reward spike; current master has no mint_reward route
F1 first-mover fixed: attacker_init_authority_result=Err("FailedTransactionMetadata { err: InstructionError(0, MissingRequiredSignature), meta: TransactionMetadata { signature: 2WEM5YWWUq3vw6TSDAZQnMWDWHrhiADsvGMEqPg2Jon3LHPXTeBCYZPkdttagsi3i2rJbSk99dAQVoADACbPF99D, logs: [\"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]\", \"Program log: init_authority signer must be current COIN mint authority\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 3430 of 200000 compute units\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: missing required signature for instruction\"], compute_units_consumed: 3430, return_data: TransactionReturnData { program_id: US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx, data: [] } } }") authority_exists=false
F1 fixed: attacker_init_result=Err("FailedTransactionMetadata { err: InstructionError(0, MissingRequiredSignature), meta: TransactionMetadata { signature: 3CJMUrj2poJ3WRVE6p8JBPdLAWWSakX1FALJJqJjxtTSes3BxKeoksGEmiLaRB1j5W6cfWmpUohaoqATBNB8mmgm, logs: [\"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]\", \"Program log: Governance adapter controller mismatch\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 6717 of 200000 compute units\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: missing required signature for instruction\"], compute_units_consumed: 6717, return_data: TransactionReturnData { program_id: US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx, data: [] } } }") mrc_exists=false attacker_coin_before=0 attacker_coin_after=0
O1 fixed: authorized_init_result=Err("FailedTransactionMetadata { err: InstructionError(0, IllegalOwner), meta: TransactionMetadata { signature: 3pXsrQHa9TCNyP15vHj1eNjAnSKEN7J4NSQicqPRnMi4MbheRcdkZT3qA4p69m4WHfJkPenHqGEqWRHDxfqh5bzv, logs: [\"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]\", \"Program 111111131h1vYVSYuKP6AhS86fbRdMw9XHiZAvAaj invoke [2]\", \"Program log: Market slab must be owned by Percolator\", \"Program 111111131h1vYVSYuKP6AhS86fbRdMw9XHiZAvAaj consumed 11108 of 191443 compute units\", \"Program 111111131h1vYVSYuKP6AhS86fbRdMw9XHiZAvAaj failed: Provided owner is not allowed\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 19665 of 200000 compute units\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: Provided owner is not allowed\"], compute_units_consumed: 19665, return_data: TransactionReturnData { program_id: 111111131h1vYVSYuKP6AhS86fbRdMw9XHiZAvAaj, data: [] } } }") slab_owner_is_percolator=false mrc_exists=false
O1 magic fixed: authorized_init_result=Err("FailedTransactionMetadata { err: InstructionError(0, InvalidAccountData), meta: TransactionMetadata { signature: 5ktF8aPWk6NipqRi6PdqUoBToLhV5ntfUNKg8wmxhJuaJ4dXmLdjFfe5LaJi1oTwmFqQ81RzaSBgSsrWrMW8Zk5A, logs: [\"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]\", \"Program 11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3 invoke [2]\", \"Program log: Percolator slab magic mismatch\", \"Program 11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3 consumed 11170 of 191443 compute units\", \"Program 11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3 failed: invalid account data for instruction\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 19727 of 200000 compute units\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: invalid account data for instruction\"], compute_units_consumed: 19727, return_data: TransactionReturnData { program_id: 11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3, data: [] } } }") slab_magic_is_exact=false mrc_exists=false
O1 collateral fixed: authorized_init_result=Err("FailedTransactionMetadata { err: InstructionError(0, InvalidAccountData), meta: TransactionMetadata { signature: P883zbqjM6syGVcbWcLRwrXSCVp7qZsETnhdAyNbgcArcHV7tak3xGHaCcuiNMpGpjBsqbfRNbJntJfzvEkk33A, logs: [\"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]\", \"Program 11111113pNDtm61yGF8j2ycAwLEPsuWQXobye5qDR invoke [2]\", \"Program log: Percolator slab collateral mint mismatch\", \"Program 11111113pNDtm61yGF8j2ycAwLEPsuWQXobye5qDR consumed 12700 of 186943 compute units\", \"Program 11111113pNDtm61yGF8j2ycAwLEPsuWQXobye5qDR failed: invalid account data for instruction\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 25757 of 200000 compute units\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: invalid account data for instruction\"], compute_units_consumed: 25757, return_data: TransactionReturnData { program_id: 11111113pNDtm61yGF8j2ycAwLEPsuWQXobye5qDR, data: [] } } }") slab_collateral_matches=false mrc_exists=false
F2 fixed: pull_result=Err(FailedTransactionMetadata { err: InstructionError(0, IncorrectProgramId), meta: TransactionMetadata { signature: 5SWGorX6tb7fBarWwDGxswrGvcvffDgbv8KjsPHMZ6BQgFetxRvQ5Je1DMKyZdYaG3oPo2WLg5dbu4KgRNyTg1mW, logs: ["Program 1111111QLbz7JHiBTspS962RLKV8GndWFwiEaqKM invoke [1]", "Program log: Unexpected Percolator program id", "Program 1111111QLbz7JHiBTspS962RLKV8GndWFwiEaqKM consumed 2786 of 200000 compute units", "Program 1111111QLbz7JHiBTspS962RLKV8GndWFwiEaqKM failed: incorrect program id for instruction"], compute_units_consumed: 2786, return_data: TransactionReturnData { program_id: 1111111QLbz7JHiBTspS962RLKV8GndWFwiEaqKM, data: [] } } }) stake_vault_before=1000000 stake_vault_after=1000000 attacker_before=0 attacker_after=0 drained=0
F3 fixed: draw_result=Err(FailedTransactionMetadata { err: InstructionError(0, MissingRequiredSignature), meta: TransactionMetadata { signature: 5TmxQpczXwazwjHJhpPhBrCVLMg8cnS7ZPLZavoYxjDR6ukmkrZf62xQsNjMp1GaXjcfQrj5ANHjpsMHYe3GnR8X, logs: ["Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]", "Program log: Governance adapter controller mismatch", "Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 6181 of 200000 compute units", "Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: missing required signature for instruction"], compute_units_consumed: 6181, return_data: TransactionReturnData { program_id: US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx, data: [] } } }) stake_vault_before=1300000 stake_vault_after=1300000 attacker_before=0 attacker_after=0 drawn=0
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.09s
```
