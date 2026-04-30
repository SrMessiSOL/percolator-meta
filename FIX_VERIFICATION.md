# FIX_VERIFICATION

Branch: `fix/critical-findings`

Base: `origin/master` at `338c31d950ef09ec1d80626bd737075989169e68` (`2026-04-21T19:48:21Z`), recorded in `HEADS.md:3-6`.

## Fix Commits

- F1/F3: `b495f1b5e82b6fc92d4c9da19af1e4ab7fc5835a` (`Gate adapter signing with bootstrap controller`)
- F2/O1: `c5994a4f4b21b257606f8748e1d47331910970d1` (`Bind rewards CPI paths to Percolator market state`)

## F1 - Adapter Arbitrary Market Init

Classification: REAL BUG in `DESIGN_VERIFICATION.md:9`.

Fix: `governance::init_authority` now requires the caller to be the current SPL Token mint authority and stores that caller as the controller; governed CPI routes load the authority account and require the stored controller signer before signing into `rewards` (`governance/src/lib.rs:133-150`, `governance/src/lib.rs:171-226`, `governance/src/lib.rs:245-246`, `governance/src/lib.rs:300-301`, `governance/src/lib.rs:369-370`).

Regression tests: `program/tests/critical_pocs.rs:490` and `program/tests/critical_pocs.rs:545`.

Verbatim proof from `verification/04-critical-pocs.log:834-838`:

```text
F1 first-mover fixed: attacker_init_authority_result=Err("FailedTransactionMetadata { err: InstructionError(0, MissingRequiredSignature), meta: TransactionMetadata { signature: 2WEM5YWWUq3vw6TSDAZQnMWDWHrhiADsvGMEqPg2Jon3LHPXTeBCYZPkdttagsi3i2rJbSk99dAQVoADACbPF99D, logs: [\"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]\", \"Program log: init_authority signer must be current COIN mint authority\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 3430 of 200000 compute units\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: missing required signature for instruction\"], compute_units_consumed: 3430, return_data: TransactionReturnData { program_id: US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx, data: [] } } }") authority_exists=false
test test_f1_first_mover_init_authority_requires_current_mint_authority ... ok
F1 fixed: attacker_init_result=Err("FailedTransactionMetadata { err: InstructionError(0, MissingRequiredSignature), meta: TransactionMetadata { signature: 3CJMUrj2poJ3WRVE6p8JBPdLAWWSakX1FALJJqJjxtTSes3BxKeoksGEmiLaRB1j5W6cfWmpUohaoqATBNB8mmgm, logs: [\"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]\", \"Program log: Governance adapter controller mismatch\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 6717 of 200000 compute units\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: missing required signature for instruction\"], compute_units_consumed: 6717, return_data: TransactionReturnData { program_id: US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx, data: [] } } }") mrc_exists=false attacker_coin_before=0 attacker_coin_after=0
F1 blocked params: n_per_epoch=50000 epoch_slots=10 slab=11111119T6fgHG3unjQB6vpWozhBdiXDbQovvFVeF collateral_mint=111111152P2r5yt6odmBLPsFCLBrFisJ3aS7LqLAT
test test_f1_governance_bypass_attacker_initializes_market_and_mints_coin ... ok
```

## F2 - `pull_insurance` Caller-Supplied Callee Drain

Classification: REAL BUG in `DESIGN_VERIFICATION.md:10`.

Fix: `pull_insurance` now pins the callee to `percolator_prog::id()`, checks the passed slab equals the MRC slab, loads the real Percolator market, validates slab owner/magic/collateral mint, validates the Percolator vault authority PDA, validates the Percolator vault token account, and validates the rewards stake vault before the signed CPI (`program/src/lib.rs:280-340`, `program/src/lib.rs:1202-1231`).

Regression test: `program/tests/critical_pocs.rs:666`.

Verbatim proof from `verification/04-critical-pocs.log:845-847`:

```text
F2 fixed: pull_result=Err(FailedTransactionMetadata { err: InstructionError(0, IncorrectProgramId), meta: TransactionMetadata { signature: 5SWGorX6tb7fBarWwDGxswrGvcvffDgbv8KjsPHMZ6BQgFetxRvQ5Je1DMKyZdYaG3oPo2WLg5dbu4KgRNyTg1mW, logs: ["Program 1111111QLbz7JHiBTspS962RLKV8GndWFwiEaqKM invoke [1]", "Program log: Unexpected Percolator program id", "Program 1111111QLbz7JHiBTspS962RLKV8GndWFwiEaqKM consumed 2786 of 200000 compute units", "Program 1111111QLbz7JHiBTspS962RLKV8GndWFwiEaqKM failed: incorrect program id for instruction"], compute_units_consumed: 2786, return_data: TransactionReturnData { program_id: 1111111QLbz7JHiBTspS962RLKV8GndWFwiEaqKM, data: [] } } }) stake_vault_before=1000000 stake_vault_after=1000000 attacker_before=0 attacker_after=0 drained=0
test test_f2_pull_insurance_caller_supplied_program_drains_stake_vault ... ok
```

## F3 - Adapter Arbitrary Profit Draw

Classification: REAL BUG in `DESIGN_VERIFICATION.md:11`.

Fix: same controller gate as F1; `draw_insurance` now requires the stored adapter controller signer before the adapter signs the governance PDA CPI into `rewards` (`governance/src/lib.rs:347-390`).

Regression test: `program/tests/critical_pocs.rs:733`.

Verbatim proof from `verification/04-critical-pocs.log:846-848`:

```text
F3 fixed: draw_result=Err(FailedTransactionMetadata { err: InstructionError(0, MissingRequiredSignature), meta: TransactionMetadata { signature: 5TmxQpczXwazwjHJhpPhBrCVLMg8cnS7ZPLZavoYxjDR6ukmkrZf62xQsNjMp1GaXjcfQrj5ANHjpsMHYe3GnR8X, logs: ["Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]", "Program log: Governance adapter controller mismatch", "Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 6181 of 200000 compute units", "Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: missing required signature for instruction"], compute_units_consumed: 6181, return_data: TransactionReturnData { program_id: US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx, data: [] } } }) stake_vault_before=1300000 stake_vault_after=1300000 attacker_before=0 attacker_after=0 drawn=0
test test_f3_governance_bypass_attacker_draws_vault_profit ... ok
```

## O1 - `init_market_rewards` Weak Slab Validation

Classification: REAL BUG in `DESIGN_VERIFICATION.md:13`.

Fix: `init_market_rewards` now calls `load_percolator_market`, which requires the slab owner to be `percolator_prog::id()`, requires the exact Percolator magic, requires enough account data for header plus config, and requires the slab config collateral mint to equal the passed collateral mint before MRC creation (`program/src/lib.rs:288-315`, `program/src/lib.rs:582-587`).

Regression tests: `program/tests/critical_pocs.rs:579`, `program/tests/critical_pocs.rs:603`, and `program/tests/critical_pocs.rs:633`.

Verbatim proof from `verification/04-critical-pocs.log:839-844`:

```text
O1 fixed: authorized_init_result=Err("FailedTransactionMetadata { err: InstructionError(0, IllegalOwner), meta: TransactionMetadata { signature: 3pXsrQHa9TCNyP15vHj1eNjAnSKEN7J4NSQicqPRnMi4MbheRcdkZT3qA4p69m4WHfJkPenHqGEqWRHDxfqh5bzv, logs: [\"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]\", \"Program 111111131h1vYVSYuKP6AhS86fbRdMw9XHiZAvAaj invoke [2]\", \"Program log: Market slab must be owned by Percolator\", \"Program 111111131h1vYVSYuKP6AhS86fbRdMw9XHiZAvAaj consumed 11108 of 191443 compute units\", \"Program 111111131h1vYVSYuKP6AhS86fbRdMw9XHiZAvAaj failed: Provided owner is not allowed\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 19665 of 200000 compute units\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: Provided owner is not allowed\"], compute_units_consumed: 19665, return_data: TransactionReturnData { program_id: 111111131h1vYVSYuKP6AhS86fbRdMw9XHiZAvAaj, data: [] } } }") slab_owner_is_percolator=false mrc_exists=false
O1 magic fixed: authorized_init_result=Err("FailedTransactionMetadata { err: InstructionError(0, InvalidAccountData), meta: TransactionMetadata { signature: 5ktF8aPWk6NipqRi6PdqUoBToLhV5ntfUNKg8wmxhJuaJ4dXmLdjFfe5LaJi1oTwmFqQ81RzaSBgSsrWrMW8Zk5A, logs: [\"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]\", \"Program 11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3 invoke [2]\", \"Program log: Percolator slab magic mismatch\", \"Program 11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3 consumed 11170 of 191443 compute units\", \"Program 11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3 failed: invalid account data for instruction\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 19727 of 200000 compute units\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: invalid account data for instruction\"], compute_units_consumed: 19727, return_data: TransactionReturnData { program_id: 11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3, data: [] } } }") slab_magic_is_exact=false mrc_exists=false
test test_o1_init_market_rewards_rejects_non_percolator_slab ... ok
test test_o1_init_market_rewards_rejects_bad_percolator_magic ... ok
O1 collateral fixed: authorized_init_result=Err("FailedTransactionMetadata { err: InstructionError(0, InvalidAccountData), meta: TransactionMetadata { signature: P883zbqjM6syGVcbWcLRwrXSCVp7qZsETnhdAyNbgcArcHV7tak3xGHaCcuiNMpGpjBsqbfRNbJntJfzvEkk33A, logs: [\"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]\", \"Program 11111113pNDtm61yGF8j2ycAwLEPsuWQXobye5qDR invoke [2]\", \"Program log: Percolator slab collateral mint mismatch\", \"Program 11111113pNDtm61yGF8j2ycAwLEPsuWQXobye5qDR consumed 12700 of 186943 compute units\", \"Program 11111113pNDtm61yGF8j2ycAwLEPsuWQXobye5qDR failed: invalid account data for instruction\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 25757 of 200000 compute units\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: invalid account data for instruction\"], compute_units_consumed: 25757, return_data: TransactionReturnData { program_id: 11111113pNDtm61yGF8j2ycAwLEPsuWQXobye5qDR, data: [] } } }") slab_collateral_matches=false mrc_exists=false
test test_o1_init_market_rewards_rejects_collateral_mint_mismatch ... ok
```

## Command Verification

Full command logs are under `verification/`.

```text
cargo build-sbf --manifest-path governance/Cargo.toml
```

Result: passed; see `verification/01-governance-build-sbf.log`.

```text
cargo build-sbf --manifest-path program/Cargo.toml
```

Result: passed; see `verification/02-program-build-sbf.log`. The log includes existing upstream warning output from `percolator-prog`, then `Finished release profile`.

```text
cargo build-sbf --manifest-path malicious-drain/Cargo.toml
```

Result: passed; see `verification/03-malicious-build-sbf.log`.

```text
cargo test --test critical_pocs -- --nocapture
```

Result: passed; `verification/04-critical-pocs.log:832-850` ends with:

```text
running 8 tests
test test_f4_governance_bypass_attacker_mints_reward_coin ... ignored, F4 was proven only on the local mint_reward spike; current master has no mint_reward route
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

```text
cargo check --workspace
```

Result: passed; see `verification/05-cargo-check-workspace.log`.

```text
cargo test --workspace --lib
```

Result: passed; `verification/06-cargo-test-workspace-lib.log:864-877` shows all workspace lib tests completed successfully.

```text
git diff --check
```

Result: passed with no output; see `verification/07-git-diff-check.log`.
