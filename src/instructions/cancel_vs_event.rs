use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{
    constants::{MARKET_SEED, MAX_MARKET_ID_LEN},
    error::MarketError,
    state::VSStatus,
    utils::{current_timestamp, read_string},
    validation::{validate_accounts, verify_pda},
};

pub fn process_cancel_vs_event<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    data: &[u8],
) -> ProgramResult {
    validate_accounts(accounts, 2, false, &[0, 1])?;

    let accounts_iter = &mut accounts.iter();
    let vs_account = next_account_info(accounts_iter)?;
    let requester = next_account_info(accounts_iter)?;

    if !requester.is_signer {
        return Err(MarketError::InvalidSigner.into());
    }

    let mut offset = 0usize;
    let vs_id = read_string(data, &mut offset, MAX_MARKET_ID_LEN)?;
    verify_pda(vs_account, &[MARKET_SEED, vs_id.as_bytes()], program_id)?;

    let mut event = {
        let data = vs_account.try_borrow_data()?;
        crate::state::VSEvent::unpack(&data)?
    };

    if event.status == VSStatus::Resolved || event.status == VSStatus::Cancelled {
        return Err(MarketError::MarketAlreadyResolved.into());
    }

    if *requester.key != event.creator {
        return Err(MarketError::UnauthorizedSettler.into());
    }

    let now = current_timestamp()?;
    if now < event.resolve_deadline_utc {
        return Err(MarketError::MarketNotResolvable.into());
    }

    event.status = VSStatus::Cancelled;
    let mut account_data = vs_account.try_borrow_mut_data()?;
    event.pack(&mut account_data)?;
    Ok(())
}
