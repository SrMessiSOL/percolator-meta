# spec.md — MetaDAO Futarchy to Percolator Genesis and Market Factory

Look at `../percolator-prog` for the Percolator program source and API shape. This repo is pure Solana Rust and uses LiteSVM tests.

## Design constraints

1. No admin keys, multisigs, or off-chain publishers are trusted after bootstrap.
2. Governance-like actions are expected to be triggered from a MetaDAO proposal marked executed.
3. Futarchy must not be able to steal depositor principal. User principal is segregated in genesis ledgers or risk-vault ledgers.
4. The governance adapter is a signing shim for the configured COIN instance, not a policy engine.
5. Legacy per-market staking/reward-pool instructions are removed. Tags `0`, `1`, `2`, `4`, `5`, `6`, `7`, and `9` are disabled.

## Programs

| Program | Role |
|---------|------|
| `meta_dao` | Proposal lifecycle and execution bit |
| `governance_adapter` | Owns the governance authority PDA and CPIs into `rewards` |
| `percolator` | Market creation, insurance/backing accounting, cranks |
| `rewards` | COIN mint authority, genesis, risk-vault subledgering, Percolator market factory/admin wiring |
| SPL Token Program | COIN mint and base/collateral token accounts |

## COIN and bootstrap

`CoinConfig` is created once per COIN mint. It records:

- `authority`: the configured governance adapter PDA.
- `bootstrap_start_slot`.
- `bootstrap_delay_slots`.
- `live_slot`.
- `phase`: bootstrap or live.

`init_coin_config(bootstrap_delay_slots)` with zero delay starts live immediately. A nonzero delay requires governed `activate_live` after `bootstrap_start_slot + bootstrap_delay_slots`.

The COIN mint must have:

- `mint_authority = PDA(rewards, [b"coin_mint_authority", coin_mint])`
- `freeze_authority = None`

## Genesis

`init_genesis_bootstrap(reward_supply, deposit_window_slots?)` creates:

```text
genesis_cfg   = PDA(rewards, [b"genesis_cfg", coin_mint])
genesis_vault = PDA(rewards, [b"genesis_vault", coin_mint])
```

The optional deposit window must be nonzero and no longer than the remaining bootstrap delay. If omitted, it defaults to about one week at 400ms slots, capped by the remaining bootstrap delay.

During the deposit window, `genesis_deposit(amount)` transfers base units into `genesis_vault` and records one vote unit per base unit. Deposits close when the window ends, when the genesis market is kicked, or when genesis is finalized.

Genesis depositors take the first market's code and market risk in exchange for voting power over the fixed COIN distribution. `kickstart_genesis_market(domain, expiry_slot)` deploys the pooled base units into a PDA-admin Percolator market:

```text
insurance = floor(total_deposited / 2)
backing   = total_deposited - insurance
```

Before finalization, `recover_genesis_market(kind, domain, amount)` can recover bootstrap insurance/backing principal and earnings only to `genesis_vault`.

After live activation, anyone may create `GenesisDistribution` allocation items. Genesis depositors vote with their recorded weight. Futarchy may call `genesis_mint_reward` only for majority-approved items. Genesis finalization requires:

- the bootstrap market was kicked; and
- `minted_supply == reward_supply`.

After finalization, `genesis_withdraw` returns up to the user's original principal. If the market lost capital, withdrawals are pro-rata against recovered funds and unpaid principal remains reserved for later recovery. `draw_genesis_surplus` can draw only vault balance above outstanding genesis principal.

## Post-genesis markets

After the COIN instance is live, any user may call `init_percolator_market`. The caller funds the market account, but the program signs Percolator `InitMarket` with:

```text
market_admin = PDA(rewards, [b"percolator_market_admin", coin_mint])
```

Futarchy controls subsequent Percolator lifecycle/admin actions through `governance_adapter::percolator_admin`, which CPIs into `rewards::percolator_admin`. The rewards program verifies `CoinConfig.authority`, signs as `market_admin`, and forwards only explicit lifecycle/admin tags.

The generic proxy must not forward raw custody changes, direct funding, or withdrawal instructions. Cranks remain external.

## Risk vaults

Post-genesis insurance and backing depositors use risk vaults:

```text
risk_vault       = PDA(rewards, [b"risk_vault", market_slab, kind_domain])
risk_token_vault = PDA(rewards, [b"risk_token_vault", market_slab, kind_domain])
risk_position    = PDA(rewards, [b"risk_position", risk_vault, user])
risk_ledger      = PDA(rewards, [b"risk_ledger", market_slab, kind_domain])
```

The meta program handles per-user subaccounting. Percolator supplies aggregate counters and accumulators through the engine ledger.

Risk vaults track:

- total deposits, withdrawals, and shares;
- reward, loss, and recovery accumulators;
- per-user shares, pending withdrawals, rewards, and losses;
- lockup and delayed-withdrawal slots;
- optional backing DAO fee routed to the main insurance token vault.

Insurance vaults cannot charge a DAO fee. Backing vaults may route a futarchy-configured fee to the main insurance vault.

## Builder approvals

`approve_builder(code_hash, terms_hash, enabled)` creates or updates:

```text
builder_approval = PDA(rewards, [b"builder_approval", coin_mint, builder_program, code_hash])
```

The target must be an executable BPF-loader-owned program account. Approval records the code hash, terms hash, approval slot, and enabled flag. Approval does not give the builder custody over depositor principal.

## Audit checklist

- [ ] `init_coin_config` validates COIN mint authority and rejects freeze authority.
- [ ] `init_genesis_bootstrap` records a short deposit window and rejects late deposits before live activation.
- [ ] Genesis deposits mint one vote unit per base unit and close after the deposit window or market kickstart.
- [ ] Genesis reward minting executes only majority-approved allocation items and cannot exceed `reward_supply`.
- [ ] Genesis finalization requires a kicked market and full reward-supply distribution.
- [ ] Genesis recovery can return bootstrap funds only to `genesis_vault` and is disabled after finalization.
- [ ] Permissionless market creation sets Percolator admin to the COIN market-admin PDA.
- [ ] The futarchy Percolator admin proxy forwards only allowed lifecycle/admin tags.
- [ ] Risk-vault deposits and withdrawals are depositor-controlled and enforce lockups/delays.
- [ ] Backing earnings fee routing can only send fees to the main insurance vault.
- [ ] Builder approvals are keyed by `(coin_mint, builder_program, code_hash)` and require executable BPF-loader-owned code.
- [ ] Legacy staking/reward-pool tags are invalid in both `rewards` and `governance_adapter`.
