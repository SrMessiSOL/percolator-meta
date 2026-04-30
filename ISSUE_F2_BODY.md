Title: `pull_insurance` signs rewards vault authority into caller-supplied program

## Summary

`rewards::pull_insurance` accepts a caller-supplied `percolator_program`, then signs the MRC PDA into that callee while passing the rewards stake vault as writable. A malicious callee can use the inherited MRC signer as the SPL Token authority and drain depositor collateral from the stake vault.

This contradicts the explicit design constraints that user funds are never at risk from futarchy (`spec.md:11`), collateral cannot leave the staking vault except through `unstake` to the staker (`spec.md:256`), and the MRC PDA authority should only transfer stake-vault collateral through `unstake` (`spec.md:285`).

## Vulnerable Path

On base `338c31d`, `pull_insurance` was permissionless, accepted `percolator_program` from the transaction accounts, built a CPI to `program_id: *percolator_program.key`, and used `invoke_signed` with the MRC PDA signer while passing the writable stake vault and SPL Token program (`DESIGN_VERIFICATION.md:10`).

The same path also left slab binding incomplete because the MRC account helper accepted `_market_slab` but did not use it (`DESIGN_VERIFICATION.md:10`).

## Proof

Regression test: `program/tests/critical_pocs.rs:666`.

Malicious BPF program: `malicious-drain/src/lib.rs:19`; the inherited signer is used for the SPL Token transfer at `malicious-drain/src/lib.rs:43-59`.

Before the fix, the PoC drained `700000` collateral from the stake vault into the attacker token account (`HEARTBEAT.md:5`). After the fix, the malicious callee is rejected with `IncorrectProgramId`, and both balances are unchanged:

```text
F2 fixed: pull_result=Err(FailedTransactionMetadata { err: InstructionError(0, IncorrectProgramId), meta: TransactionMetadata { signature: 5SWGorX6tb7fBarWwDGxswrGvcvffDgbv8KjsPHMZ6BQgFetxRvQ5Je1DMKyZdYaG3oPo2WLg5dbu4KgRNyTg1mW, logs: ["Program 1111111QLbz7JHiBTspS962RLKV8GndWFwiEaqKM invoke [1]", "Program log: Unexpected Percolator program id", "Program 1111111QLbz7JHiBTspS962RLKV8GndWFwiEaqKM consumed 2786 of 200000 compute units", "Program 1111111QLbz7JHiBTspS962RLKV8GndWFwiEaqKM failed: incorrect program id for instruction"], compute_units_consumed: 2786, return_data: TransactionReturnData { program_id: 1111111QLbz7JHiBTspS962RLKV8GndWFwiEaqKM, data: [] } } }) stake_vault_before=1000000 stake_vault_after=1000000 attacker_before=0 attacker_after=0 drained=0
test test_f2_pull_insurance_caller_supplied_program_drains_stake_vault ... ok
```

Full output: `verification/04-critical-pocs.log:845-847`.

## Severity

Critical. Any caller could provide a malicious callee and drain depositor principal from a rewards stake vault.

## Fix

Fixed in `c5994a4f4b21b257606f8748e1d47331910970d1`.

The fix pins the callee to `percolator_prog::id()` (`program/src/lib.rs:280-285`), validates the MRC slab binding (`program/src/lib.rs:1205-1218`), loads the Percolator market and validates slab owner/magic/collateral mint (`program/src/lib.rs:288-315`), validates the rewards stake vault (`program/src/lib.rs:1220-1224`), and validates the Percolator vault account and vault authority PDA before signing (`program/src/lib.rs:318-340`, `program/src/lib.rs:1225-1231`).
