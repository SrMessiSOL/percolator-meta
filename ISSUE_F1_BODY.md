Title: Governance adapter permits arbitrary market reward initialization

## Summary

`governance_adapter::init_market_rewards` can be driven by any signed payer on `338c31d`, letting a non-DAO caller initialize a reward market with caller-chosen `n_per_epoch`, `epoch_slots`, slab, collateral mint, and stake vault.

This contradicts the design intent that governance-like actions come from an executed MetaDAO proposal (`spec.md:10`) and that the adapter is only a signing shim whose trust boundary is established during the init ceremony (`spec.md:28`). `CoinConfig.authority` is supposed to be the only key that can register new markets for a COIN (`spec.md:216`), and the README describes this path as DAO-only (`README.md:83`).

## Vulnerable Path

On base `338c31d`, `init_authority` was the root gap: it accepted any signed payer and created the authority PDA without proving that payer controlled the DAO or the COIN mint authority (`DESIGN_VERIFICATION.md:9`). After that, the adapter accepted any signed payer for `init_market_rewards`, verified only the PDA path, forwarded caller-chosen reward params and accounts, and had the adapter PDA sign into `rewards` (`DESIGN_VERIFICATION.md:9`).

The rewards program then only checked that the signer matched `CoinConfig.authority`, so the adapter signature was enough (`DESIGN_VERIFICATION.md:9`).

## Proof

Regression tests: `program/tests/critical_pocs.rs:490` and `program/tests/critical_pocs.rs:545`.

Before the fix, this PoC initialized the MRC with attacker params and minted `50000` COIN to the attacker (`HEARTBEAT.md:5`). After the fix, first-mover adapter initialization and post-bootstrap market initialization both fail with `MissingRequiredSignature`, the authority/MRC accounts remain uninitialized, and the attacker's COIN account is unchanged:

```text
F1 first-mover fixed: attacker_init_authority_result=Err("FailedTransactionMetadata { err: InstructionError(0, MissingRequiredSignature), meta: TransactionMetadata { signature: 2WEM5YWWUq3vw6TSDAZQnMWDWHrhiADsvGMEqPg2Jon3LHPXTeBCYZPkdttagsi3i2rJbSk99dAQVoADACbPF99D, logs: [\"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]\", \"Program log: init_authority signer must be current COIN mint authority\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 3430 of 200000 compute units\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: missing required signature for instruction\"], compute_units_consumed: 3430, return_data: TransactionReturnData { program_id: US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx, data: [] } } }") authority_exists=false
F1 fixed: attacker_init_result=Err("FailedTransactionMetadata { err: InstructionError(0, MissingRequiredSignature), meta: TransactionMetadata { signature: 3CJMUrj2poJ3WRVE6p8JBPdLAWWSakX1FALJJqJjxtTSes3BxKeoksGEmiLaRB1j5W6cfWmpUohaoqATBNB8mmgm, logs: [\"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]\", \"Program log: Governance adapter controller mismatch\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 6717 of 200000 compute units\", \"Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: missing required signature for instruction\"], compute_units_consumed: 6717, return_data: TransactionReturnData { program_id: US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx, data: [] } } }") mrc_exists=false attacker_coin_before=0 attacker_coin_after=0
test test_f1_governance_bypass_attacker_initializes_market_and_mints_coin ... ok
```

Full output: `verification/04-critical-pocs.log:834-838`.

## Severity

Critical. A first mover or arbitrary caller could bind the adapter authority and register a rewards market with attacker-selected emissions, then farm real COIN.

## Fix

Fixed in `b495f1b5e82b6fc92d4c9da19af1e4ab7fc5835a`.

The adapter authority account now stores a controller. `init_authority` must be signed by the current SPL Token mint authority before the mint-authority handoff, and governed adapter routes require that stored controller signer before any PDA-signed CPI (`governance/src/lib.rs:133-150`, `governance/src/lib.rs:171-226`, `governance/src/lib.rs:300-301`). The spec now documents that bootstrap requirement explicitly (`spec.md:13-14`, `spec.md:101`).
