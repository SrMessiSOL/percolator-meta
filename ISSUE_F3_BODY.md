Title: Governance adapter permits arbitrary profit draws from reward vaults

## Summary

`governance_adapter::draw_insurance` can be driven by any signed payer on `338c31d`. The rewards program correctly caps draws to vault profit, but the adapter did not enforce that the draw was DAO-approved before signing the governance PDA CPI.

This contradicts the design intent that all governance-like actions come from an executed MetaDAO proposal (`spec.md:10`) and the README attack analysis that a vault draw requires governance PDA signature and therefore only DAO votes can trigger draws (`README.md:80`). The profit-only invariant exists (`README.md:16`), but it does not authorize arbitrary callers to redirect that profit.

## Vulnerable Path

On base `338c31d`, the adapter accepted any signed payer for `draw_insurance`, verified only the authority PDA path, forwarded arbitrary destination and amount, and signed the CPI into `rewards` (`DESIGN_VERIFICATION.md:11`). `rewards::draw_insurance` validated only the collateral mint on the destination account and the profit cap before transferring (`DESIGN_VERIFICATION.md:11`).

## Proof

Regression test: `program/tests/critical_pocs.rs:733`.

Before the fix, the PoC transferred `300000` of simulated vault profit to the attacker (`HEARTBEAT.md:5`). After the fix, the attacker path fails with `MissingRequiredSignature`, and both the stake vault and attacker account are unchanged:

```text
F3 fixed: draw_result=Err(FailedTransactionMetadata { err: InstructionError(0, MissingRequiredSignature), meta: TransactionMetadata { signature: 5TmxQpczXwazwjHJhpPhBrCVLMg8cnS7ZPLZavoYxjDR6ukmkrZf62xQsNjMp1GaXjcfQrj5ANHjpsMHYe3GnR8X, logs: ["Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx invoke [1]", "Program log: Governance adapter controller mismatch", "Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx consumed 6181 of 200000 compute units", "Program US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx failed: missing required signature for instruction"], compute_units_consumed: 6181, return_data: TransactionReturnData { program_id: US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx, data: [] } } }) stake_vault_before=1300000 stake_vault_after=1300000 attacker_before=0 attacker_after=0 drawn=0
test test_f3_governance_bypass_attacker_draws_vault_profit ... ok
```

Full output: `verification/04-critical-pocs.log:846-848`.

## Severity

Critical. The direct principal-protection cap held, but any caller could redirect all vault profit that governance was meant to control.

## Fix

Fixed in `b495f1b5e82b6fc92d4c9da19af1e4ab7fc5835a`.

The adapter authority account now stores the bootstrap controller, and `draw_insurance` requires that controller signer before signing into `rewards` (`governance/src/lib.rs:102-130`, `governance/src/lib.rs:347-390`).
