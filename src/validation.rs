use solana_program::{
    account_info::AccountInfo,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::MarketError;

pub fn validate_accounts<'a>(
    accounts: &'a [AccountInfo<'a>],
    expected_len: usize,
    signer_required: bool,
    writable_indices: &[usize],
) -> Result<(), ProgramError> {
    
    if accounts.len() < expected_len {
        msg!(
            "Account count mismatch: expected at least {}, got {}",
            expected_len,
            accounts.len()
        );
        return Err(MarketError::InvalidAccountCount.into());
    }

    
    if signer_required && !accounts[0].is_signer {
        msg!("Account[0] must be a signer");
        return Err(MarketError::InvalidSigner.into());
    }

    
    for &idx in writable_indices {
        if idx >= accounts.len() {
            msg!("Writable index {} out of bounds (len={})", idx, accounts.len());
            return Err(MarketError::InvalidAccountIndex.into());
        }
        if !accounts[idx].is_writable {
            msg!("Account[{}] must be writable", idx);
            return Err(MarketError::InvalidWritable.into());
        }
    }
    
    for &idx in writable_indices {
        if accounts[idx].executable {
            msg!("Account[{}] is executable — expected a data account", idx);
            return Err(MarketError::InvalidAccount.into());
        }
    }

    Ok(())
}

pub fn verify_program_owned(
    account: &AccountInfo,
    program_id: &Pubkey,
) -> Result<(), ProgramError> {
    if account.owner != program_id {
        msg!(
            "Account {} is not owned by program {}",
            account.key,
            program_id
        );
        return Err(MarketError::InvalidOwner.into());
    }
    Ok(())
}

pub fn verify_pda(
    account: &AccountInfo,
    seeds: &[&[u8]],
    program_id: &Pubkey,
) -> Result<u8, ProgramError> {
    let (expected_pda, bump) = Pubkey::find_program_address(seeds, program_id);
    if account.key != &expected_pda {
        msg!(
            "PDA mismatch: expected {}, got {}",
            expected_pda,
            account.key
        );
        return Err(MarketError::InvalidPDA.into());
    }
    Ok(bump)
}

pub fn verify_uninitialized(account: &AccountInfo) -> Result<(), ProgramError> {
    let data = account.try_borrow_data()?;
    if !data.is_empty() && data[0] != 0 {
        msg!("Account {} is already initialized", account.key);
        return Err(MarketError::InvalidAccount.into());
    }
    Ok(())
}

pub fn verify_initialized(account: &AccountInfo) -> Result<(), ProgramError> {
    let data = account.try_borrow_data()?;
    if data.is_empty() || data.iter().all(|&b| b == 0) {
        msg!("Account {} is uninitialized", account.key);
        return Err(MarketError::UninitializedAccount.into());
    }
    Ok(())
}