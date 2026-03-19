use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    system_program,
    sysvar::{instructions::ID as INSTRUCTIONS_SYSVAR_ID, Sysvar},
};

use crate::{
    constants::{
        APPROVED_SETTLER, MARKET_SEED, SETTLEMENT_ACCOUNT_SIZE, SETTLEMENT_SEED,
        MAX_OUTCOME_DESCRIPTION_LEN,
    },
    error::MarketError,
    state::{Market, Outcome, SettlementLog},
    utils::{
        current_timestamp, read_signature, read_string, read_u8,
        sha256, verify_settlement_signature_via_sysvar,
    },
    validation::{validate_accounts, verify_pda, verify_program_owned},
};

pub fn process_settle_market_gem(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
    market_id: &str,
) -> ProgramResult {
    msg!("=== SettleMarketGEM === MarketID={}", market_id);

    validate_accounts(accounts, 5, false, &[0, 1])?;

    let accounts_iter          = &mut accounts.iter();
    let market_account         = next_account_info(accounts_iter)?;
    let settlement_log_account = next_account_info(accounts_iter)?;
    let settler                = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;
    let instructions_sysvar    = next_account_info(accounts_iter)?;

    if !settler.is_signer {
        msg!("Settler must be a signer");
        return Err(MarketError::InvalidSigner.into());
    }

    if settler.key != &APPROVED_SETTLER {
        msg!(
            "Unauthorized settler: expected {}, got {}",
            APPROVED_SETTLER,
            settler.key
        );
        return Err(MarketError::UnauthorizedSettler.into());
    }

    if system_program_account.key != &system_program::id() {
        msg!("Invalid system_program account");
        return Err(MarketError::InvalidAccount.into());
    }

    if instructions_sysvar.key != &INSTRUCTIONS_SYSVAR_ID {
        msg!("Invalid instructions sysvar account");
        return Err(MarketError::InvalidAccount.into());
    }
    if data.is_empty() {
        msg!("Empty instruction data");
        return Err(MarketError::InvalidInstructionData.into());
    }

    let mut offset = 0usize;

    let outcome_byte        = read_u8(data, &mut offset)?;
    let outcome             = Outcome::from_u8(outcome_byte)?;
    let outcome_description = read_string(data, &mut offset, MAX_OUTCOME_DESCRIPTION_LEN)?;
    let settlement_signature = read_signature(data, &mut offset)?;

    msg!("Outcome: {:?}", outcome);
    msg!("Description: {}", outcome_description);

    let market_id_bytes = market_id.as_bytes();
    verify_pda(market_account, &[MARKET_SEED, market_id_bytes], program_id)?;

    verify_program_owned(market_account, program_id)?;

    let mut market = {
        let data = market_account.try_borrow_data()?;
        Market::unpack(&data)?
    };

    if market.is_resolved {
        msg!("Market {} is already resolved", market_id);
        return Err(MarketError::MarketAlreadyResolved.into());
    }

    let now = current_timestamp()?;
    if now < market.end_time_utc {
        msg!(
            "Market {} has not expired yet (end={}, now={})",
            market_id,
            market.end_time_utc,
            now
        );
        return Err(MarketError::MarketNotExpired.into());
    }

    if settler.key != &market.approved_settler {
        msg!(
            "Settler {} does not match market.approved_settler {}",
            settler.key,
            market.approved_settler
        );
        return Err(MarketError::UnauthorizedSettler.into());
    }

    let outcome_str = match outcome {
        Outcome::Yes => "YES",
        Outcome::No  => "NO",
    };
    let expected_message = format!("VANT_GEM_SETTLEMENT:{}:{}", market_id, outcome_str);
    msg!("Expected settlement message: {}", expected_message);

    verify_settlement_signature_via_sysvar(
        instructions_sysvar,
        &APPROVED_SETTLER,
        expected_message.as_bytes(),
    )?;

    market.is_resolved         = true;
    market.outcome             = Some(outcome);
    market.outcome_description = outcome_description.clone();
    market.end_price           = None;

    {
        let mut account_data = market_account.try_borrow_mut_data()?;
        market.pack(&mut account_data)?;
    }

    msg!("Market state updated: is_resolved=true, outcome={:?}", outcome);

    let settlement_bump = verify_pda(
        settlement_log_account,
        &[SETTLEMENT_SEED, market_id_bytes],
        program_id,
    )?;

    let rent = Rent::get()?;
    let lamports_needed = rent.minimum_balance(SETTLEMENT_ACCOUNT_SIZE);

    let settlement_signer_seeds: &[&[u8]] =
        &[SETTLEMENT_SEED, market_id_bytes, &[settlement_bump]];

    invoke_signed(
        &system_instruction::create_account(
            settler.key,
            settlement_log_account.key,
            lamports_needed,
            SETTLEMENT_ACCOUNT_SIZE as u64,
            program_id,
        ),
        &[
            settler.clone(),
            settlement_log_account.clone(),
            system_program_account.clone(),
        ],
        &[settlement_signer_seeds],
    )?;

    msg!("SettlementLog PDA account created");

    let sig_hash = sha256(&settlement_signature);
    let msg_hash = sha256(expected_message.as_bytes());

    let log = SettlementLog {
        market:              *market_account.key,
        settled_at:          now,
        settled_by:          *settler.key,
        end_price:           None,
        outcome,
        outcome_description,
        signature_hash:      sig_hash,
        message_hash:        msg_hash,
        bump:                settlement_bump,
    };

    {
        let mut log_data = settlement_log_account.try_borrow_mut_data()?;
        log.pack(&mut log_data)?;
    }

    msg!("SettleMarketGEM complete. MarketID={}, Outcome={:?}", market_id, outcome);
    Ok(())
}