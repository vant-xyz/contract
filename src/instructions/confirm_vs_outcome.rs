use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{
    constants::{MARKET_SEED, MAX_MARKET_ID_LEN},
    error::MarketError,
    state::{VSMode, VSStatus},
    utils::{current_timestamp, read_string, read_u8},
    validation::{validate_accounts, verify_pda},
};

pub fn process_confirm_vs_outcome<'a>(
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
    let outcome = read_u8(data, &mut offset)?;
    if outcome > 1 {
        return Err(MarketError::InvalidOutcome.into());
    }

    verify_pda(vs_account, &[MARKET_SEED, vs_id.as_bytes()], program_id)?;

    let mut event = {
        let data = vs_account.try_borrow_data()?;
        crate::state::VSEvent::unpack(&data)?
    };

    if event.status != VSStatus::Active {
        return Err(MarketError::MarketNotStarted.into());
    }
    if !event.has_participant(participant.key) {
        return Err(MarketError::UnauthorizedSettler.into());
    }

    let now = current_timestamp()?;
    if now > event.resolve_deadline_utc {
        return Err(MarketError::MarketNotResolvable.into());
    }

    if event.votes_yes.iter().any(|p| p == participant.key)
        || event.votes_no.iter().any(|p| p == participant.key)
    {
        return Err(MarketError::VSDuplicateVote.into());
    }

    if outcome == 1 {
        event.votes_yes.push(*participant.key);
    } else {
        event.votes_no.push(*participant.key);
    }

    let yes_count = event.votes_yes.len() as u8;
    let no_count = event.votes_no.len() as u8;

    match event.mode {
        VSMode::Mutual => {
            if event.participant_count == 2 && (yes_count == 2 || no_count == 2) {
                event.status = VSStatus::Resolved;
                event.outcome = Some(if yes_count == 2 { 1 } else { 0 });
            }
        }
        VSMode::Consensus => {
            let threshold = if event.threshold == 0 {
                1
            } else {
                event.threshold
            };
            if yes_count >= threshold || no_count >= threshold {
                event.status = VSStatus::Resolved;
                event.outcome = Some(if yes_count >= threshold { 1 } else { 0 });
            }
        }
    }

    let mut account_data = vs_account.try_borrow_mut_data()?;
    event.pack(&mut account_data)?;
    Ok(())
}
