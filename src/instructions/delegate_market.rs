use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::invoke_signed,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction, system_program,
    sysvar::Sysvar,
};

use crate::{
    constants::{ER_VALIDATOR, MARKET_SEED, MAX_MARKET_ID_LEN},
    error::MarketError,
    utils::read_string,
    validation::verify_pda,
};

const DELEGATION_PROGRAM_ID: Pubkey =
    solana_program::pubkey!("DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh");

#[derive(BorshSerialize)]
struct DelegateArgs {
    commit_frequency_ms: u32,
    seeds: Vec<Vec<u8>>,
    validator: Option<Pubkey>,
}

fn delegation_record_pda(market_pda: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"delegation", market_pda.as_ref()],
        &DELEGATION_PROGRAM_ID,
    )
}

fn delegation_metadata_pda(market_pda: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"delegation-metadata", market_pda.as_ref()],
        &DELEGATION_PROGRAM_ID,
    )
}

fn delegate_buffer_pda(market_pda: &Pubkey, owner_program: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"buffer", market_pda.as_ref()], owner_program)
}

pub fn process_delegate_market<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let market_account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let buffer_account = next_account_info(accounts_iter)?;
    let delegation_record = next_account_info(accounts_iter)?;
    let delegation_metadata = next_account_info(accounts_iter)?;
    let owner_program_account = next_account_info(accounts_iter)?;
    let delegation_program_account = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;

    if !payer.is_signer {
        msg!("Payer must be a signer");
        return Err(MarketError::InvalidSigner.into());
    }
    if system_program_account.key != &system_program::id() {
        msg!("Invalid system_program account");
        return Err(MarketError::InvalidAccount.into());
    }
    if delegation_program_account.key != &DELEGATION_PROGRAM_ID {
        msg!("Invalid delegation_program account");
        return Err(MarketError::InvalidAccount.into());
    }

    let mut offset = 0usize;
    let market_id = read_string(data, &mut offset, MAX_MARKET_ID_LEN)?;
    let market_id_bytes = market_id.as_bytes();

    let bump = verify_pda(market_account, &[MARKET_SEED, market_id_bytes], program_id)?;

    let data_len = market_account.data_len();

    let (expected_buffer, buffer_bump) = delegate_buffer_pda(market_account.key, program_id);
    let (expected_record, _) = delegation_record_pda(market_account.key);
    let (expected_metadata, _) = delegation_metadata_pda(market_account.key);

    if buffer_account.key != &expected_buffer {
        msg!("Invalid buffer PDA");
        return Err(MarketError::InvalidAccount.into());
    }
    if delegation_record.key != &expected_record {
        msg!("Invalid delegation_record PDA");
        return Err(MarketError::InvalidAccount.into());
    }
    if delegation_metadata.key != &expected_metadata {
        msg!("Invalid delegation_metadata PDA");
        return Err(MarketError::InvalidAccount.into());
    }

    let market_seeds: &[&[u8]] = &[MARKET_SEED, market_id_bytes, &[bump]];
    let buffer_seeds: &[&[u8]] = &[b"buffer", market_account.key.as_ref(), &[buffer_bump]];

    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(data_len);

    invoke_signed(
        &system_instruction::create_account(
            payer.key,
            buffer_account.key,
            lamports,
            data_len as u64,
            program_id,
        ),
        &[
            payer.clone(),
            buffer_account.clone(),
            system_program_account.clone(),
        ],
        &[buffer_seeds],
    )?;

    {
        let market_data = market_account.try_borrow_data()?;
        let mut buf_data = buffer_account.try_borrow_mut_data()?;
        buf_data.copy_from_slice(&market_data);
    }

    {
        let mut market_data = market_account.try_borrow_mut_data()?;
        for byte in market_data.iter_mut() {
            *byte = 0;
        }
    }

    market_account.assign(&system_program::id());

    invoke_signed(
        &system_instruction::assign(market_account.key, &DELEGATION_PROGRAM_ID),
        &[market_account.clone(), system_program_account.clone()],
        &[market_seeds],
    )?;

    let args = DelegateArgs {
        commit_frequency_ms: 1_000,
        seeds: vec![MARKET_SEED.to_vec(), market_id_bytes.to_vec()],
        validator: Some(ER_VALIDATOR),
    };
    let mut ix_data = vec![0u8; 8];
    args.serialize(&mut ix_data)
        .map_err(|_| MarketError::InvalidInstructionData)?;

    let delegate_ix = Instruction {
        program_id: DELEGATION_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*payer.key, true),
            AccountMeta::new(*market_account.key, true),
            AccountMeta::new_readonly(*owner_program_account.key, false),
            AccountMeta::new(*buffer_account.key, false),
            AccountMeta::new(*delegation_record.key, false),
            AccountMeta::new(*delegation_metadata.key, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: ix_data,
    };

    invoke_signed(
        &delegate_ix,
        &[
            payer.clone(),
            market_account.clone(),
            owner_program_account.clone(),
            buffer_account.clone(),
            delegation_record.clone(),
            delegation_metadata.clone(),
            system_program_account.clone(),
        ],
        &[market_seeds],
    )?;

    let buffer_lamports = buffer_account.lamports();
    let payer_lamports = payer.lamports();
    **payer.try_borrow_mut_lamports()? = payer_lamports
        .checked_add(buffer_lamports)
        .ok_or(MarketError::ArithmeticOverflow)?;
    **buffer_account.try_borrow_mut_lamports()? = 0;
    {
        let mut buf_data = buffer_account.try_borrow_mut_data()?;
        for byte in buf_data.iter_mut() {
            *byte = 0;
        }
    }
    buffer_account.assign(&system_program::id());

    msg!("DelegateMarket complete. MarketID={}", market_id);
    Ok(())
}
