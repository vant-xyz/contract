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

pub fn process_join_vs_event<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    data: &[u8],
) -> ProgramResult {
    validate_accounts(accounts, 2, false, &[0])?;

    let accounts_iter = &mut accounts.iter();
    let vs_account = next_account_info(accounts_iter)?;
    let participant = next_account_info(accounts_iter)?;

    if !participant.is_signer {
        return Err(MarketError::InvalidSigner.into());
    }

    let mut offset = 0usize;
    let vs_id = read_string(data, &mut offset, MAX_MARKET_ID_LEN)?;
    verify_pda(vs_account, &[MARKET_SEED, vs_id.as_bytes()], program_id)?;

    let mut event = {
        let data = vs_account.try_borrow_data()?;
        crate::state::VSEvent::unpack(&data)?
    };

    if event.status != VSStatus::Open {
        return Err(MarketError::InvalidInstructionData.into());
    }

    let now = current_timestamp()?;
    if now > event.join_deadline_utc {
        return Err(MarketError::MarketNotResolvable.into());
    }

    if event.has_participant(participant.key) {
        return Err(MarketError::VSParticipantAlreadyJoined.into());
    }

    if event.participants.len() >= event.participant_count as usize {
        return Err(MarketError::VSParticipantLimitReached.into());
    }

    event.participants.push(*participant.key);
    if event.participants.len() == event.participant_count as usize {
        event.status = VSStatus::Active;
    }

    let mut account_data = vs_account.try_borrow_mut_data()?;
    event.pack(&mut account_data)?;
    Ok(())
}
