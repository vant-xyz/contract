use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction, system_program,
    sysvar::Sysvar,
};

use crate::{
    constants::{MARKET_SEED, MAX_MARKET_ID_LEN, MAX_TITLE_LEN, VS_EVENT_ACCOUNT_SIZE},
    error::MarketError,
    state::{VSEvent, VSMode, VSStatus},
    utils::{current_timestamp, read_string, read_u64, read_u8},
    validation::{validate_accounts, verify_pda},
};

pub fn process_create_vs_event<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    data: &[u8],
) -> ProgramResult {
    validate_accounts(accounts, 3, false, &[0])?;

    let accounts_iter = &mut accounts.iter();
    let vs_account = next_account_info(accounts_iter)?;
    let creator = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;

    if !creator.is_signer {
        return Err(MarketError::InvalidSigner.into());
    }
    if system_program_account.key != &system_program::id() {
        return Err(MarketError::InvalidAccount.into());
    }

    let mut offset = 0usize;
    let vs_id = read_string(data, &mut offset, MAX_MARKET_ID_LEN)?;
    let title = read_string(data, &mut offset, MAX_TITLE_LEN)?;
    let stake_cents = read_u64(data, &mut offset)?;
    let mode = VSMode::from_u8(read_u8(data, &mut offset)?)?;
    let threshold = read_u8(data, &mut offset)?;
    let join_deadline_utc = read_u64(data, &mut offset)?;
    let resolve_deadline_utc = read_u64(data, &mut offset)?;
    let participant_count = read_u8(data, &mut offset)?;

    if stake_cents == 0 || participant_count < 2 {
        return Err(MarketError::InvalidInstructionData.into());
    }

    let now = current_timestamp()?;
    if join_deadline_utc <= now || resolve_deadline_utc <= join_deadline_utc {
        return Err(MarketError::InvalidEndTime.into());
    }

    let seeds_base = &[MARKET_SEED, vs_id.as_bytes()];
    let bump = verify_pda(vs_account, seeds_base, program_id)?;

    let rent = Rent::get()?;
    let lamports_needed = rent.minimum_balance(VS_EVENT_ACCOUNT_SIZE);
    let signer_seeds: &[&[u8]] = &[MARKET_SEED, vs_id.as_bytes(), &[bump]];

    invoke_signed(
        &system_instruction::create_account(
            creator.key,
            vs_account.key,
            lamports_needed,
            VS_EVENT_ACCOUNT_SIZE as u64,
            program_id,
        ),
        &[
            creator.clone(),
            vs_account.clone(),
            system_program_account.clone(),
        ],
        &[signer_seeds],
    )?;

    let mut participants = Vec::with_capacity(participant_count as usize);
    participants.push(*creator.key);

    let event = VSEvent {
        vs_id,
        creator: *creator.key,
        title,
        stake_cents,
        mode,
        threshold,
        status: VSStatus::Open,
        created_at: now,
        join_deadline_utc,
        resolve_deadline_utc,
        participant_count,
        participants,
        outcome: None,
        outcome_description: String::new(),
        votes_yes: Vec::new(),
        votes_no: Vec::new(),
        bump,
    };

    let mut account_data = vs_account.try_borrow_mut_data()?;
    event.pack(&mut account_data)?;
    msg!("CreateVSEvent complete");
    Ok(())
}
