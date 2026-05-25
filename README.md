# Percolator Meta

MetaDAO-governed Solana programs for bootstrapping Percolator markets, distributing the initial COIN supply, and accounting for insurance/backing capital without giving the DAO custody over depositor principal.

## Genesis Plan

1. MetaDAO initializes `CoinConfig` with a configurable bootstrap delay in slots. A zero delay is live immediately; the intended launch setting can be a six-month slot delay.
2. During bootstrap, users deposit base units into `genesis_vault`. One deposited base unit equals one genesis vote unit.
3. Genesis depositors take the first market's code and market risk in exchange for voting power over the fixed genesis COIN distribution.
4. Futarchy kickstarts the first Percolator market with the pooled base units as a 50/50 split: `floor(total / 2)` to insurance and the remainder to backing.
5. Before finalization, futarchy may recover bootstrap market insurance/backing principal and earnings only back into `genesis_vault`.
6. After the delay, genesis depositors vote on allocation items. Futarchy can mint COIN only for majority-approved items, and 100% of the fixed reward supply must be minted before finalization.
7. Finalization requires both a kicked bootstrap market and `minted_supply == reward_supply`.
8. After finalization, depositors can withdraw up to their original base-unit deposit. If the market lost capital, withdrawals are pro-rata against recovered funds and unpaid principal remains reserved for later recovery.

Any surplus in `genesis_vault` above outstanding genesis principal is drawable by futarchy after finalization.

## Post-Genesis Lifecycle

After `activate_live`, anyone may create additional Percolator markets through `init_percolator_market`. The caller funds the market account, but the COIN-specific `percolator_market_admin` PDA becomes Percolator admin.

Futarchy controls the market lifecycle through explicit meta-program instructions:

- Percolator market init, asset lifecycle, oracle setup, fee policy, resolve, and close-slab cleanup.
- Insurance and backing risk vault setup.
- Builder-code approvals by `(coin_mint, builder_program, code_hash)` plus a terms hash.

Raw Percolator `UpdateAuthority` is not exposed through the generic admin proxy. Custody-bearing authority changes must use the explicit setup paths.

Cranks and permissionless Percolator maintenance remain external.

## Capital Accounting

Post-genesis staking depositors are segregated from DAO custody:

```text
drawable = vault_balance - total_staked
```

`draw_insurance` can only move profits above depositor principal. `unstake` and `claim_stake_rewards` are permissionless and do not require governance.

External insurance/backing depositors use risk vaults:

- Deposits and principal withdrawals are tracked per depositor in this program.
- Percolator only supplies aggregate counters and accumulators.
- Backing earnings can be claimed by depositors minus a DAO fee routed to the main insurance vault.
- Backing/insurance lockups and delayed withdrawals are enforced by the meta program.

Genesis depositors are different: their principal is intentionally at risk during the bootstrap market.

## Reward Model

Staking rewards use a Synthetix-style accumulator:

```text
reward_rate = n_per_epoch / epoch_slots
user_rate = reward_rate * user_deposit / total_staked
```

COIN rewards are minted by the program PDA. They are not drawn from collateral vaults.

## Tested Surface

The LiteSVM suite covers the full current lifecycle:

- Configurable bootstrap delay and live activation.
- Genesis deposit, vote, 100% supply mint, finalize, withdrawal, surplus, recovery, and 50/50 kickstart.
- Permissionless market creation plus futarchy-controlled Percolator lifecycle/admin operations.
- Insurance/backing risk vault setup, sync, depositor withdrawal, and backing earnings fee routing.
- Builder approvals and executable-program validation.
- Post-genesis depositor capital protection, reward accounting, withdrawal guarantees, market isolation, and adversarial account substitution.

Current full-suite smoke target:

```bash
cargo build-sbf --manifest-path governance/Cargo.toml
cargo build-sbf --manifest-path program/Cargo.toml
RUST_MIN_STACK=8388608 cargo test --manifest-path program/Cargo.toml --test integration
```

The integration test also requires a built Percolator BPF binary at `../percolator-prog/target/deploy/percolator_prog.so`.

## Instructions

| Tag | Instruction | Purpose |
|-----|-------------|---------|
| 0 | `init_market_rewards` | Create per-market reward config and stake vault |
| 1 | `stake` | Deposit collateral and begin earning COIN |
| 2 | `unstake` | Withdraw collateral and claim pending COIN |
| 3 | `init_coin_config` | One-time COIN governance/mint setup |
| 4 | `claim_stake_rewards` | Claim pending COIN without withdrawing |
| 5 | `draw_insurance` | Governance draw of profits above `total_staked` |
| 6 | `register_insurance_operator` | Legacy burned-admin operator registration |
| 7 | `pull_insurance` | Sweep Percolator insurance into the stake vault |
| 8 | `mint_reward` | Governance-gated discretionary COIN mint |
| 9 | `set_market_rewards` | Update reward emission settings |
| 10 | `transfer_mint_authority` | Transfer or burn COIN mint authority |
| 11 | `activate_live` | Move from bootstrap to live after delay |
| 12 | `init_risk_vault` | Set up insurance/backing depositor accounting |
| 13 | `register_risk_vault_authority` | Register risk-vault PDA with Percolator |
| 14 | `risk_deposit` | External insurance/backing principal deposit |
| 15 | `risk_request_withdraw` | Request delayed principal withdrawal |
| 16 | `risk_withdraw` | Withdraw matured principal |
| 17 | `sync_risk_vault` | Sync Percolator aggregate counters |
| 18 | `risk_claim_rewards` | Claim backing earnings minus DAO fee |
| 19 | `init_percolator_market` | Permissionless Percolator `InitMarket` via PDA admin |
| 20 | `percolator_admin` | Futarchy-gated Percolator lifecycle/admin CPI |
| 21 | `init_genesis_bootstrap` | Create genesis config and base-token vault |
| 22 | `genesis_deposit` | Bootstrap base-unit deposit, 1 unit = 1 vote |
| 23 | `genesis_withdraw` | Post-finalization principal withdrawal |
| 24 | `genesis_mint_reward` | Mint approved genesis allocation |
| 25 | `finalize_genesis` | Complete genesis after kickstart and full mint |
| 26 | `draw_genesis_surplus` | Draw surplus above outstanding principal |
| 27 | `kickstart_genesis_market` | Deploy genesis principal 50/50 to first market |
| 28 | `recover_genesis_market` | Recover bootstrap market funds to `genesis_vault` |
| 29 | `init_genesis_distribution` | Create a genesis allocation item |
| 30 | `vote_genesis_distribution` | Vote on a genesis allocation item |
| 31 | `approve_builder` | Governed builder-code and terms registry |

## Key PDAs

| Account | Seeds |
|---------|-------|
| `CoinConfig` | `[b"coin_cfg", coin_mint]` |
| `percolator_market_admin` | `[b"percolator_market_admin", coin_mint]` |
| `GenesisConfig` | `[b"genesis_cfg", coin_mint]` |
| `GenesisVault` | `[b"genesis_vault", coin_mint]` |
| `GenesisPosition` | `[b"genesis_position", genesis_cfg, user]` |
| `GenesisDistribution` | `[b"genesis_distribution", genesis_cfg, proposal_id]` |
| `GenesisDistributionVote` | `[b"genesis_distribution_vote", distribution, voter]` |
| `BuilderApproval` | `[b"builder_approval", coin_mint, builder_program, code_hash]` |
| `MarketRewardsCfg` | `[b"mrc", market_slab]` |
| `StakePosition` | `[b"sp", market_slab, user]` |
| `DepositVault` | `[b"stake_vault", market_slab]` |
| `CoinMintAuthority` | `[b"coin_mint_authority", coin_mint]` |
