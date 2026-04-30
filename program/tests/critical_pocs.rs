//! Critical PoCs for issue #1.
//!
//! Build before running:
//!   cargo build-sbf --manifest-path governance/Cargo.toml
//!   cargo build-sbf --manifest-path program/Cargo.toml
//!   cargo build-sbf --manifest-path malicious-drain/Cargo.toml
//!
//! Run:
//!   cargo test --test critical_pocs -- --nocapture

use governance_adapter::{
    authority_address as governance_authority_address, id as governance_program_id,
};
use litesvm::LiteSVM;
use percolator_prog::{accounts as percolator_accounts, constants as percolator_constants};
use solana_sdk::{
    account::Account,
    clock::Clock,
    instruction::{AccountMeta, Instruction},
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    sysvar,
    transaction::Transaction,
};
use spl_token::state::{Account as TokenAccount, AccountState, Mint};
use std::path::PathBuf;

const FAKE_SLAB_LEN: usize = 4096;

fn rewards_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.push("target/deploy/rewards_program.so");
    assert!(path.exists(), "missing rewards BPF at {:?}", path);
    path
}

fn governance_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.push("target/deploy/governance_adapter.so");
    assert!(path.exists(), "missing governance BPF at {:?}", path);
    path
}

fn malicious_drain_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.push("target/deploy/malicious_drain.so");
    assert!(path.exists(), "missing malicious-drain BPF at {:?}", path);
    path
}

fn make_token_account_data(mint: &Pubkey, owner: &Pubkey, amount: u64) -> Vec<u8> {
    let mut data = vec![0u8; TokenAccount::LEN];
    let account = TokenAccount {
        mint: *mint,
        owner: *owner,
        amount,
        state: AccountState::Initialized,
        ..TokenAccount::default()
    };
    TokenAccount::pack(account, &mut data).unwrap();
    data
}

fn make_mint_data_with_authority(mint_authority: &Pubkey) -> Vec<u8> {
    let mut data = vec![0u8; Mint::LEN];
    let mint = Mint {
        mint_authority: solana_sdk::program_option::COption::Some(*mint_authority),
        supply: 0,
        decimals: 6,
        is_initialized: true,
        freeze_authority: solana_sdk::program_option::COption::None,
    };
    Mint::pack(mint, &mut data).unwrap();
    data
}

fn make_mint_data_no_authority() -> Vec<u8> {
    let mut data = vec![0u8; Mint::LEN];
    let mint = Mint {
        mint_authority: solana_sdk::program_option::COption::None,
        supply: 0,
        decimals: 6,
        is_initialized: true,
        freeze_authority: solana_sdk::program_option::COption::None,
    };
    Mint::pack(mint, &mut data).unwrap();
    data
}

fn encode_governance_init_authority() -> Vec<u8> {
    vec![0u8]
}

fn encode_governance_init_coin_config() -> Vec<u8> {
    vec![1u8]
}

fn encode_governance_init_market_rewards(n: u64, epoch_slots: u64) -> Vec<u8> {
    let mut data = vec![2u8];
    data.extend_from_slice(&n.to_le_bytes());
    data.extend_from_slice(&epoch_slots.to_le_bytes());
    data
}

fn encode_governance_draw_insurance(amount: u64) -> Vec<u8> {
    let mut data = vec![3u8];
    data.extend_from_slice(&amount.to_le_bytes());
    data
}

fn encode_governance_mint_reward(amount: u64) -> Vec<u8> {
    let mut data = vec![4u8];
    data.extend_from_slice(&amount.to_le_bytes());
    data
}

fn try_init_governance_authority(
    svm: &mut LiteSVM,
    governance_id: Pubkey,
    signer: &Keypair,
    authority_pda: Pubkey,
    rewards_id: Pubkey,
    coin_mint: Pubkey,
) -> Result<(), String> {
    let ix = Instruction {
        program_id: governance_id,
        accounts: vec![
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new(authority_pda, false),
            AccountMeta::new_readonly(rewards_id, false),
            AccountMeta::new_readonly(coin_mint, false),
            AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
        ],
        data: encode_governance_init_authority(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&signer.pubkey()),
        &[signer],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx)
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

fn encode_stake(amount: u64) -> Vec<u8> {
    let mut data = vec![1u8];
    data.extend_from_slice(&amount.to_le_bytes());
    data
}

fn encode_claim_stake_rewards() -> Vec<u8> {
    vec![4u8]
}

fn encode_pull_insurance(amount: u64) -> Vec<u8> {
    let mut data = vec![7u8];
    data.extend_from_slice(&amount.to_le_bytes());
    data
}

struct PocEnv {
    svm: LiteSVM,
    rewards_id: Pubkey,
    governance_id: Pubkey,
    payer: Keypair,
    authority_pda: Pubkey,
    slab: Pubkey,
    collateral_mint: Pubkey,
    coin_mint: Pubkey,
    mint_authority_pda: Pubkey,
}

impl PocEnv {
    fn new() -> Self {
        let mut svm = LiteSVM::new();

        let rewards_id = Pubkey::new_unique();
        let rewards_bytes = std::fs::read(rewards_path()).expect("read rewards BPF");
        svm.add_program(rewards_id, &rewards_bytes);

        let governance_id = governance_program_id();
        let governance_bytes = std::fs::read(governance_path()).expect("read governance BPF");
        svm.add_program(governance_id, &governance_bytes);

        let payer = Keypair::new();
        svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();
        svm.set_sysvar(&Clock {
            slot: 100,
            unix_timestamp: 100,
            ..Clock::default()
        });

        let collateral_mint = Pubkey::new_unique();
        svm.set_account(
            collateral_mint,
            Account {
                lamports: 1_000_000,
                data: make_mint_data_no_authority(),
                owner: spl_token::ID,
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();

        let slab = Pubkey::new_unique();
        let mut slab_data = vec![0u8; FAKE_SLAB_LEN];
        slab_data[..8].copy_from_slice(&percolator_constants::MAGIC.to_le_bytes());
        let cfg_off = percolator_constants::HEADER_LEN;
        slab_data[cfg_off..cfg_off + 32].copy_from_slice(collateral_mint.as_ref());
        let fake_percolator_vault = Pubkey::new_unique();
        slab_data[cfg_off + 32..cfg_off + 64].copy_from_slice(fake_percolator_vault.as_ref());
        let (_vault_auth, vault_bump) =
            percolator_accounts::derive_vault_authority(&percolator_prog::id(), &slab);
        slab_data[cfg_off + 106] = vault_bump;
        svm.set_account(
            slab,
            Account {
                lamports: 1_000_000_000,
                data: slab_data,
                owner: percolator_prog::id(),
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();

        let coin_mint = Pubkey::new_unique();
        let (mint_authority_pda, _) = Pubkey::find_program_address(
            &[b"coin_mint_authority", coin_mint.as_ref()],
            &rewards_id,
        );
        svm.set_account(
            coin_mint,
            Account {
                lamports: 1_000_000,
                data: make_mint_data_with_authority(&payer.pubkey()),
                owner: spl_token::ID,
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();

        let (authority_pda, _) = governance_authority_address(&rewards_id, &coin_mint);
        try_init_governance_authority(
            &mut svm,
            governance_id,
            &payer,
            authority_pda,
            rewards_id,
            coin_mint,
        )
        .expect("init authority failed");

        let mut coin_account = svm.get_account(&coin_mint).expect("missing coin mint");
        coin_account.data = make_mint_data_with_authority(&mint_authority_pda);
        svm.set_account(coin_mint, coin_account).unwrap();

        let (coin_cfg, _) =
            Pubkey::find_program_address(&[b"coin_cfg", coin_mint.as_ref()], &rewards_id);
        let ix = Instruction {
            program_id: governance_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(authority_pda, false),
                AccountMeta::new_readonly(rewards_id, false),
                AccountMeta::new_readonly(coin_mint, false),
                AccountMeta::new(coin_cfg, false),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
            ],
            data: encode_governance_init_coin_config(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).expect("init coin config failed");

        Self {
            svm,
            rewards_id,
            governance_id,
            payer,
            authority_pda,
            slab,
            collateral_mint,
            coin_mint,
            mint_authority_pda,
        }
    }

    fn airdrop(&mut self, key: &Pubkey) {
        self.svm.airdrop(key, 10_000_000_000).unwrap();
    }

    fn create_token_account(&mut self, mint: &Pubkey, owner: &Pubkey, amount: u64) -> Pubkey {
        let token_account = Pubkey::new_unique();
        self.svm
            .set_account(
                token_account,
                Account {
                    lamports: 1_000_000,
                    data: make_token_account_data(mint, owner, amount),
                    owner: spl_token::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();
        token_account
    }

    fn set_clock(&mut self, slot: u64) {
        self.svm.set_sysvar(&Clock {
            slot,
            unix_timestamp: slot as i64,
            ..Clock::default()
        });
        self.svm.expire_blockhash();
    }

    fn try_init_market_rewards_as(
        &mut self,
        signer: &Keypair,
        n_per_epoch: u64,
        epoch_slots: u64,
    ) -> Result<(), String> {
        let (mrc, _) =
            Pubkey::find_program_address(&[b"mrc", self.slab.as_ref()], &self.rewards_id);
        let (coin_cfg, _) =
            Pubkey::find_program_address(&[b"coin_cfg", self.coin_mint.as_ref()], &self.rewards_id);
        let (stake_vault, _) =
            Pubkey::find_program_address(&[b"stake_vault", self.slab.as_ref()], &self.rewards_id);

        let ix = Instruction {
            program_id: self.governance_id,
            accounts: vec![
                AccountMeta::new(signer.pubkey(), true),
                AccountMeta::new(self.authority_pda, false),
                AccountMeta::new_readonly(self.rewards_id, false),
                AccountMeta::new_readonly(self.slab, false),
                AccountMeta::new(mrc, false),
                AccountMeta::new_readonly(self.coin_mint, false),
                AccountMeta::new_readonly(coin_cfg, false),
                AccountMeta::new_readonly(self.collateral_mint, false),
                AccountMeta::new(stake_vault, false),
                AccountMeta::new_readonly(spl_token::ID, false),
                AccountMeta::new_readonly(sysvar::rent::ID, false),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
            ],
            data: encode_governance_init_market_rewards(n_per_epoch, epoch_slots),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&signer.pubkey()),
            &[signer],
            self.svm.latest_blockhash(),
        );
        self.svm
            .send_transaction(tx)
            .map(|_| ())
            .map_err(|e| format!("{:?}", e))
    }

    fn init_market_rewards_as(&mut self, signer: &Keypair, n_per_epoch: u64, epoch_slots: u64) {
        self.try_init_market_rewards_as(signer, n_per_epoch, epoch_slots)
            .expect("init_market_rewards via adapter failed");
    }

    fn stake(&mut self, user: &Keypair, amount: u64) -> Pubkey {
        let collateral_mint = self.collateral_mint;
        let user_ata = self.create_token_account(&collateral_mint, &user.pubkey(), amount);
        let (mrc, _) =
            Pubkey::find_program_address(&[b"mrc", self.slab.as_ref()], &self.rewards_id);
        let (stake_vault, _) =
            Pubkey::find_program_address(&[b"stake_vault", self.slab.as_ref()], &self.rewards_id);
        let (stake_position, _) = Pubkey::find_program_address(
            &[b"sp", self.slab.as_ref(), user.pubkey().as_ref()],
            &self.rewards_id,
        );

        let ix = Instruction {
            program_id: self.rewards_id,
            accounts: vec![
                AccountMeta::new(user.pubkey(), true),
                AccountMeta::new(mrc, false),
                AccountMeta::new_readonly(self.slab, false),
                AccountMeta::new(user_ata, false),
                AccountMeta::new(stake_vault, false),
                AccountMeta::new(stake_position, false),
                AccountMeta::new_readonly(spl_token::ID, false),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
                AccountMeta::new_readonly(sysvar::clock::ID, false),
            ],
            data: encode_stake(amount),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[user],
            self.svm.latest_blockhash(),
        );
        self.svm.send_transaction(tx).expect("stake failed");
        user_ata
    }

    fn claim_rewards(&mut self, user: &Keypair) -> Pubkey {
        let coin_mint = self.coin_mint;
        let user_coin_ata = self.create_token_account(&coin_mint, &user.pubkey(), 0);
        let (mrc, _) =
            Pubkey::find_program_address(&[b"mrc", self.slab.as_ref()], &self.rewards_id);
        let (stake_position, _) = Pubkey::find_program_address(
            &[b"sp", self.slab.as_ref(), user.pubkey().as_ref()],
            &self.rewards_id,
        );

        let ix = Instruction {
            program_id: self.rewards_id,
            accounts: vec![
                AccountMeta::new(user.pubkey(), true),
                AccountMeta::new(mrc, false),
                AccountMeta::new_readonly(self.slab, false),
                AccountMeta::new(stake_position, false),
                AccountMeta::new(self.coin_mint, false),
                AccountMeta::new(user_coin_ata, false),
                AccountMeta::new_readonly(self.mint_authority_pda, false),
                AccountMeta::new_readonly(spl_token::ID, false),
                AccountMeta::new_readonly(sysvar::clock::ID, false),
            ],
            data: encode_claim_stake_rewards(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[user],
            self.svm.latest_blockhash(),
        );
        self.svm.send_transaction(tx).expect("claim failed");
        user_coin_ata
    }

    fn read_token_balance(&self, token_account: &Pubkey) -> u64 {
        let account = self
            .svm
            .get_account(token_account)
            .expect("missing token account");
        TokenAccount::unpack(&account.data).unwrap().amount
    }

    fn set_token_balance(&mut self, token_account: &Pubkey, amount: u64) {
        let mut account = self
            .svm
            .get_account(token_account)
            .expect("missing token account");
        let mut token = TokenAccount::unpack(&account.data).unwrap();
        token.amount = amount;
        TokenAccount::pack(token, &mut account.data).unwrap();
        self.svm.set_account(*token_account, account).unwrap();
    }

    fn read_mrc_params(&self) -> (u64, u64, Pubkey) {
        let (mrc, _) =
            Pubkey::find_program_address(&[b"mrc", self.slab.as_ref()], &self.rewards_id);
        let account = self.svm.get_account(&mrc).expect("missing mrc");
        let collateral = Pubkey::new_from_array(account.data[72..104].try_into().unwrap());
        let n_per_epoch = u64::from_le_bytes(account.data[104..112].try_into().unwrap());
        let epoch_slots = u64::from_le_bytes(account.data[112..120].try_into().unwrap());
        (n_per_epoch, epoch_slots, collateral)
    }

    fn stake_vault(&self) -> Pubkey {
        Pubkey::find_program_address(&[b"stake_vault", self.slab.as_ref()], &self.rewards_id).0
    }

    fn coin_config(&self) -> Pubkey {
        Pubkey::find_program_address(&[b"coin_cfg", self.coin_mint.as_ref()], &self.rewards_id).0
    }
}

#[test]
fn test_f1_first_mover_init_authority_requires_current_mint_authority() {
    let mut svm = LiteSVM::new();

    let rewards_id = Pubkey::new_unique();
    let rewards_bytes = std::fs::read(rewards_path()).expect("read rewards BPF");
    svm.add_program(rewards_id, &rewards_bytes);

    let governance_id = governance_program_id();
    let governance_bytes = std::fs::read(governance_path()).expect("read governance BPF");
    svm.add_program(governance_id, &governance_bytes);

    let dao = Keypair::new();
    let attacker = Keypair::new();
    svm.airdrop(&dao.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&attacker.pubkey(), 10_000_000_000).unwrap();

    let coin_mint = Pubkey::new_unique();
    svm.set_account(
        coin_mint,
        Account {
            lamports: 1_000_000,
            data: make_mint_data_with_authority(&dao.pubkey()),
            owner: spl_token::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    let (authority_pda, _) = governance_authority_address(&rewards_id, &coin_mint);
    let result = try_init_governance_authority(
        &mut svm,
        governance_id,
        &attacker,
        authority_pda,
        rewards_id,
        coin_mint,
    );

    eprintln!(
        "F1 first-mover fixed: attacker_init_authority_result={:?} authority_exists={}",
        result,
        svm.get_account(&authority_pda).is_some()
    );
    assert!(
        result.is_err(),
        "non-mint-authority attacker must not create adapter authority"
    );
    assert!(
        svm.get_account(&authority_pda).is_none(),
        "authority PDA must remain uninitialized"
    );
}

#[test]
fn test_f1_governance_bypass_attacker_initializes_market_and_mints_coin() {
    let mut env = PocEnv::new();
    let attacker = Keypair::new();
    env.airdrop(&attacker.pubkey());

    let attacker_n_per_epoch = 50_000u64;
    let attacker_epoch_slots = 10u64;
    let attacker_coin = env.coin_mint;
    let attacker_coin_ata = env.create_token_account(&attacker_coin, &attacker.pubkey(), 0);
    let before_coin = env.read_token_balance(&attacker_coin_ata);
    let result =
        env.try_init_market_rewards_as(&attacker, attacker_n_per_epoch, attacker_epoch_slots);
    let (mrc, _) = Pubkey::find_program_address(&[b"mrc", env.slab.as_ref()], &env.rewards_id);

    eprintln!(
        "F1 fixed: attacker_init_result={:?} mrc_exists={} attacker_coin_before={} attacker_coin_after={}",
        result,
        env.svm.get_account(&mrc).is_some(),
        before_coin,
        env.read_token_balance(&attacker_coin_ata)
    );
    assert!(result.is_err(), "attacker init_market_rewards must fail");
    assert!(
        env.svm.get_account(&mrc).is_none(),
        "MRC must remain uninitialized"
    );
    assert_eq!(env.read_token_balance(&attacker_coin_ata), before_coin);
    eprintln!(
        "F1 blocked params: n_per_epoch={} epoch_slots={} slab={} collateral_mint={}",
        attacker_n_per_epoch, attacker_epoch_slots, env.slab, env.collateral_mint
    );
}

#[test]
fn test_o1_init_market_rewards_rejects_non_percolator_slab() {
    let mut env = PocEnv::new();
    let bootstrap = Keypair::from_bytes(&env.payer.to_bytes()).unwrap();
    let mut slab_account = env.svm.get_account(&env.slab).expect("missing slab");
    slab_account.owner = Pubkey::new_unique();
    env.svm.set_account(env.slab, slab_account).unwrap();

    let result = env.try_init_market_rewards_as(&bootstrap, 1_000, 100);
    let (mrc, _) = Pubkey::find_program_address(&[b"mrc", env.slab.as_ref()], &env.rewards_id);

    eprintln!(
        "O1 fixed: authorized_init_result={:?} slab_owner_is_percolator={} mrc_exists={}",
        result,
        env.svm.get_account(&env.slab).unwrap().owner == percolator_prog::id(),
        env.svm.get_account(&mrc).is_some()
    );
    assert!(result.is_err(), "non-Percolator slab must be rejected");
    assert!(
        env.svm.get_account(&mrc).is_none(),
        "MRC must remain uninitialized"
    );
}

#[test]
fn test_o1_init_market_rewards_rejects_bad_percolator_magic() {
    let mut env = PocEnv::new();
    let bootstrap = Keypair::from_bytes(&env.payer.to_bytes()).unwrap();
    let mut slab_account = env.svm.get_account(&env.slab).expect("missing slab");
    slab_account.data[..8].copy_from_slice(&(percolator_constants::MAGIC + 1).to_le_bytes());
    env.svm.set_account(env.slab, slab_account).unwrap();

    let result = env.try_init_market_rewards_as(&bootstrap, 1_000, 100);
    let (mrc, _) = Pubkey::find_program_address(&[b"mrc", env.slab.as_ref()], &env.rewards_id);

    eprintln!(
        "O1 magic fixed: authorized_init_result={:?} slab_magic_is_exact={} mrc_exists={}",
        result,
        {
            let account = env.svm.get_account(&env.slab).unwrap();
            u64::from_le_bytes(account.data[..8].try_into().unwrap()) == percolator_constants::MAGIC
        },
        env.svm.get_account(&mrc).is_some()
    );
    assert!(
        result.is_err(),
        "bad Percolator slab magic must be rejected"
    );
    assert!(
        env.svm.get_account(&mrc).is_none(),
        "MRC must remain uninitialized"
    );
}

#[test]
fn test_o1_init_market_rewards_rejects_collateral_mint_mismatch() {
    let mut env = PocEnv::new();
    let bootstrap = Keypair::from_bytes(&env.payer.to_bytes()).unwrap();
    let wrong_mint = Pubkey::new_unique();
    let mut slab_account = env.svm.get_account(&env.slab).expect("missing slab");
    let cfg_off = percolator_constants::HEADER_LEN;
    slab_account.data[cfg_off..cfg_off + 32].copy_from_slice(wrong_mint.as_ref());
    env.svm.set_account(env.slab, slab_account).unwrap();

    let result = env.try_init_market_rewards_as(&bootstrap, 1_000, 100);
    let (mrc, _) = Pubkey::find_program_address(&[b"mrc", env.slab.as_ref()], &env.rewards_id);

    eprintln!(
        "O1 collateral fixed: authorized_init_result={:?} slab_collateral_matches={} mrc_exists={}",
        result,
        {
            let account = env.svm.get_account(&env.slab).unwrap();
            let stored: [u8; 32] = account.data[cfg_off..cfg_off + 32].try_into().unwrap();
            stored == env.collateral_mint.to_bytes()
        },
        env.svm.get_account(&mrc).is_some()
    );
    assert!(
        result.is_err(),
        "Percolator slab collateral mismatch must be rejected"
    );
    assert!(
        env.svm.get_account(&mrc).is_none(),
        "MRC must remain uninitialized"
    );
}

#[test]
fn test_f2_pull_insurance_caller_supplied_program_drains_stake_vault() {
    let mut env = PocEnv::new();
    let depositor = Keypair::new();
    env.airdrop(&depositor.pubkey());
    let bootstrap = Keypair::from_bytes(&env.payer.to_bytes()).unwrap();
    env.init_market_rewards_as(&bootstrap, 0, 100);
    env.stake(&depositor, 1_000_000);

    let attacker = Keypair::new();
    env.airdrop(&attacker.pubkey());
    let collateral_mint = env.collateral_mint;
    let attacker_ata = env.create_token_account(&collateral_mint, &attacker.pubkey(), 0);

    let malicious_id = Pubkey::new_unique();
    let malicious_bytes = std::fs::read(malicious_drain_path()).expect("read malicious BPF");
    env.svm.add_program(malicious_id, &malicious_bytes);

    let (mrc, _) = Pubkey::find_program_address(&[b"mrc", env.slab.as_ref()], &env.rewards_id);
    let stake_vault = env.stake_vault();
    let fake_vault_pda = Pubkey::new_unique();
    let before_vault = env.read_token_balance(&stake_vault);
    let before_attacker = env.read_token_balance(&attacker_ata);
    let drain_amount = 700_000u64;

    let ix = Instruction {
        program_id: env.rewards_id,
        accounts: vec![
            AccountMeta::new(attacker.pubkey(), true),
            AccountMeta::new(mrc, false),
            AccountMeta::new(env.slab, false),
            AccountMeta::new(stake_vault, false),
            AccountMeta::new(attacker_ata, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(fake_vault_pda, false),
            AccountMeta::new_readonly(sysvar::clock::ID, false),
            AccountMeta::new_readonly(malicious_id, false),
        ],
        data: encode_pull_insurance(drain_amount),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&attacker.pubkey()),
        &[&attacker],
        env.svm.latest_blockhash(),
    );
    let result = env.svm.send_transaction(tx);

    let after_vault = env.read_token_balance(&stake_vault);
    let after_attacker = env.read_token_balance(&attacker_ata);
    eprintln!(
        "F2 fixed: pull_result={:?} stake_vault_before={} stake_vault_after={} attacker_before={} attacker_after={} drained={}",
        result,
        before_vault,
        after_vault,
        before_attacker,
        after_attacker,
        after_attacker - before_attacker
    );
    assert!(result.is_err(), "malicious pull_insurance must fail");
    assert_eq!(after_vault, before_vault, "stake vault must be unchanged");
    assert_eq!(
        after_attacker, before_attacker,
        "attacker account must be unchanged"
    );
}

#[test]
fn test_f3_governance_bypass_attacker_draws_vault_profit() {
    let mut env = PocEnv::new();
    let bootstrap = Keypair::from_bytes(&env.payer.to_bytes()).unwrap();
    env.init_market_rewards_as(&bootstrap, 0, 100);

    let depositor = Keypair::new();
    env.airdrop(&depositor.pubkey());
    env.stake(&depositor, 1_000_000);

    let stake_vault = env.stake_vault();
    env.set_token_balance(&stake_vault, 1_300_000);

    let attacker = Keypair::new();
    env.airdrop(&attacker.pubkey());
    let collateral_mint = env.collateral_mint;
    let attacker_dest = env.create_token_account(&collateral_mint, &attacker.pubkey(), 0);
    let (mrc, _) = Pubkey::find_program_address(&[b"mrc", env.slab.as_ref()], &env.rewards_id);

    let draw_amount = 300_000u64;
    let before_vault = env.read_token_balance(&stake_vault);
    let before_attacker = env.read_token_balance(&attacker_dest);
    let ix = Instruction {
        program_id: env.governance_id,
        accounts: vec![
            AccountMeta::new(attacker.pubkey(), true),
            AccountMeta::new(env.authority_pda, false),
            AccountMeta::new_readonly(env.rewards_id, false),
            AccountMeta::new_readonly(mrc, false),
            AccountMeta::new_readonly(env.slab, false),
            AccountMeta::new(stake_vault, false),
            AccountMeta::new(attacker_dest, false),
            AccountMeta::new_readonly(env.coin_mint, false),
            AccountMeta::new_readonly(env.coin_config(), false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: encode_governance_draw_insurance(draw_amount),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&attacker.pubkey()),
        &[&attacker],
        env.svm.latest_blockhash(),
    );
    let result = env.svm.send_transaction(tx);

    let after_vault = env.read_token_balance(&stake_vault);
    let after_attacker = env.read_token_balance(&attacker_dest);
    eprintln!(
        "F3 fixed: draw_result={:?} stake_vault_before={} stake_vault_after={} attacker_before={} attacker_after={} drawn={}",
        result,
        before_vault,
        after_vault,
        before_attacker,
        after_attacker,
        after_attacker - before_attacker
    );
    assert!(
        result.is_err(),
        "attacker draw_insurance via adapter must fail"
    );
    assert_eq!(after_vault, before_vault, "stake vault must be unchanged");
    assert_eq!(
        after_attacker, before_attacker,
        "attacker account must be unchanged"
    );
}

#[test]
#[ignore = "F4 was proven only on the local mint_reward spike; current master has no mint_reward route"]
fn test_f4_governance_bypass_attacker_mints_reward_coin() {
    let mut env = PocEnv::new();
    let attacker = Keypair::new();
    env.airdrop(&attacker.pubkey());
    let coin_mint = env.coin_mint;
    let attacker_dest = env.create_token_account(&coin_mint, &attacker.pubkey(), 0);

    let mint_amount = 123_456u64;
    let before_attacker = env.read_token_balance(&attacker_dest);
    let ix = Instruction {
        program_id: env.governance_id,
        accounts: vec![
            AccountMeta::new(attacker.pubkey(), true),
            AccountMeta::new(env.authority_pda, false),
            AccountMeta::new_readonly(env.rewards_id, false),
            AccountMeta::new(env.coin_mint, false),
            AccountMeta::new_readonly(env.coin_config(), false),
            AccountMeta::new(attacker_dest, false),
            AccountMeta::new_readonly(env.mint_authority_pda, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: encode_governance_mint_reward(mint_amount),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&attacker.pubkey()),
        &[&attacker],
        env.svm.latest_blockhash(),
    );
    env.svm
        .send_transaction(tx)
        .expect("attacker mint_reward via adapter failed");

    let after_attacker = env.read_token_balance(&attacker_dest);
    eprintln!(
        "F4 observed: attacker_before={} attacker_after={} minted={}",
        before_attacker,
        after_attacker,
        after_attacker - before_attacker
    );
    assert_eq!(after_attacker - before_attacker, mint_amount);
}
