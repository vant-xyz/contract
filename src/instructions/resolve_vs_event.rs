use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{
    constants::{MARKET_SEED, MAX_MARKET_ID_LEN, MAX_OUTCOME_DESCRIPTION_LEN},
    error::MarketError,
    state::VSStatus,
    utils::{read_string, read_u8},
    validation::{validate_accounts, verify_pda},
};

pub fn process_resolve_vs_event<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    data: &[u8],
) -> ProgramResult {
    validate_accounts(accounts, 2, false, &[0, 1])?;

    let accounts_iter = &mut accounts.iter();
    let vs_account = next_account_info(accounts_iter)?;
    let settler = next_account_info(accounts_iter)?;

    if !settler.is_signer {
        return Err(MarketError::InvalidSigner.into());
    }

    let mut offset = 0usize;
    let vs_id = read_string(data, &mut offset, MAX_MARKET_ID_LEN)?;
    let outcome = read_u8(data, &mut offset)?;
    let desc = read_string(data, &mut offset, MAX_OUTCOME_DESCRIPTION_LEN)?;

    verify_pda(vs_account, &[MARKET_SEED, vs_id.as_bytes()], program_id)?;

    let mut event = {
        let data = vs_account.try_borrow_data()?;
        crate::state::VSEvent::unpack(&data)?
    };

    if event.status == VSStatus::Resolved || event.status == VSStatus::Cancelled {
        return Err(MarketError::MarketAlreadyResolved.into());
    }

    if *settler.key != event.creator {
        return Err(MarketError::UnauthorizedSettler.into());
    }

    if outcome > 1 {
        return Err(MarketError::InvalidOutcome.into());
    }

    event.status = VSStatus::Resolved;
    event.outcome = Some(outcome);
    event.outcome_description = desc;

    let mut account_data = vs_account.try_borrow_mut_data()?;
    event.pack(&mut account_data)?;
    Ok(())
}
