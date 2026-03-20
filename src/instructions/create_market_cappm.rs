use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    system_program,
    sysvar::Sysvar,
};

use crate::{
    constants::{
        APPROVED_DATA_PROVIDERS, APPROVED_SETTLER, MARKET_ACCOUNT_SIZE, MARKET_SEED,
        MAX_DATA_PROVIDER_LEN, MAX_DESCRIPTION_LEN, MAX_MARKET_ID_LEN, MAX_TITLE_LEN,
    },
    error::MarketError,
    state::{Direction, Market, MarketType},
    utils::{current_timestamp, read_string, read_u64, read_u8},
    validation::{validate_accounts, verify_pda},
};

pub fn process_create_market_cappm<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    data: &[u8],
) -> ProgramResult {
    msg!("=== CreateMarketCAPPM ===");

    validate_accounts(
        accounts,
        3,
        false,
        &[0],
    )?;

    let accounts_iter          = &mut accounts.iter();
    let market_account         = next_account_info(accounts_iter)?;
    let creator                = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;

    if !creator.is_signer {
        msg!("Creator must be a signer");
        return Err(MarketError::InvalidSigner.into());
    }

    if system_program_account.key != &system_program::id() {
        msg!("Invalid system_program account");
        return Err(MarketError::InvalidAccount.into());
    }

    if data.is_empty() {
        msg!("Empty instruction data");
        return Err(MarketError::InvalidInstructionData.into());
    }

    let mut offset = 0usize;

    let market_id        = read_string(data, &mut offset, MAX_MARKET_ID_LEN)?;
    let title            = read_string(data, &mut offset, MAX_TITLE_LEN)?;
    let description      = read_string(data, &mut offset, MAX_DESCRIPTION_LEN)?;
    let start_time_utc   = read_u64(data, &mut offset)?;
    let duration_seconds = read_u64(data, &mut offset)?;
    let direction_byte   = read_u8(data, &mut offset)?;
    let target_price     = read_u64(data, &mut offset)?;
    let data_provider    = read_string(data, &mut offset, MAX_DATA_PROVIDER_LEN)?;
    let current_price    = read_u64(data, &mut offset)?;

    msg!("MarketID: {}", market_id);
    msg!("Title: {}", title);
    msg!("StartTime: {}, Duration: {}s", start_time_utc, duration_seconds);

    let now = current_timestamp()?;

    if start_time_utc <= now {
        msg!(
            "start_time_utc ({}) must be in the future (now={})",
            start_time_utc,
            now
        );
        return Err(MarketError::InvalidEndTime.into());
    }

    if target_price == 0 {
        msg!("target_price must be > 0");
        return Err(MarketError::InvalidTargetPrice.into());
    }

    if !APPROVED_DATA_PROVIDERS.contains(&data_provider.as_str()) {
        msg!("Unapproved data_provider: {}", data_provider);
        return Err(MarketError::InvalidDataProvider.into());
    }

    let direction = Direction::from_u8(direction_byte)?;

    let end_time_utc = start_time_utc
        .checked_add(duration_seconds)
        .ok_or(MarketError::ArithmeticOverflow)?;

    msg!("EndTime: {}", end_time_utc);

    let market_id_bytes = market_id.as_bytes();
    let bump = verify_pda(
        market_account,
        &[MARKET_SEED, market_id_bytes],
        program_id,
    )?;

    let rent = Rent::get()?;
    let lamports_needed = rent.minimum_balance(MARKET_ACCOUNT_SIZE);

    msg!(
        "Creating market account: {} lamports for {} bytes",
        lamports_needed,
        MARKET_ACCOUNT_SIZE
    );

    let signer_seeds: &[&[u8]] = &[MARKET_SEED, market_id_bytes, &[bump]];

    invoke_signed(
        &system_instruction::create_account(
            creator.key,
            market_account.key,
            lamports_needed,
            MARKET_ACCOUNT_SIZE as u64,
            program_id,
        ),
        &[creator.clone(), market_account.clone(), system_program_account.clone()],
        &[signer_seeds],
    )?;

    msg!("Market PDA account created successfully");

    let market = Market {
        market_type:         MarketType::CAPPM,
        is_resolved:         false,
        creator:             *creator.key,
        approved_settler:    APPROVED_SETTLER,
        title,
        description,
        start_time_utc,
        end_time_utc,
        duration_seconds,
        data_provider,
        created_at:          now,
        bump,
        direction:           Some(direction),
        target_price:        Some(target_price),
        current_price:       Some(current_price),
        end_price:           None,
        outcome:             None,
        outcome_description: String::new(),
    };

    let mut account_data = market_account.try_borrow_mut_data()?;
    market.pack(&mut account_data)?;

    msg!("CreateMarketCAPPM complete. MarketID={}", market_id);
    Ok(())
}