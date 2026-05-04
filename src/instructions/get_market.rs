use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

use crate::{
    constants::MARKET_SEED,
    state::{Direction, Market, MarketType, Outcome},
    validation::{validate_accounts, verify_pda, verify_program_owned},
};

/// Read-only instruction which loads a market account and logs all fields. VCS or an external service can call getTransaction on the tx signature to read the logs
/// Accounts: [0] market_account (read-only, PDA)
/// Instruction data layout (after discriminator): market_id: u16 len + bytes
pub fn process_get_market<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    market_id: &str,
) -> ProgramResult {
    msg!("=== GetMarket === MarketID={}", market_id);

    validate_accounts(accounts, 1, false, &[])?;

    let accounts_iter = &mut accounts.iter();
    let market_account = next_account_info(accounts_iter)?;

    let market_id_bytes = market_id.as_bytes();
    verify_pda(market_account, &[MARKET_SEED, market_id_bytes], program_id)?;
    verify_program_owned(market_account, program_id)?;

    let market = {
        let data = market_account.try_borrow_data()?;
        Market::unpack(&data)?
    };

    // logging all fields
    msg!("MARKET_ID:{}", market_id);
    msg!(
        "MARKET_TYPE:{}",
        match market.market_type {
            MarketType::CAPPM => "CAPPM",
            MarketType::GEM => "GEM",
        }
    );
    msg!("IS_RESOLVED:{}", market.is_resolved);
    msg!("CREATOR:{}", market.creator);
    msg!("APPROVED_SETTLER:{}", market.approved_settler);
    msg!("TITLE:{}", market.title);
    msg!("DESCRIPTION:{}", market.description);
    msg!("START_TIME:{}", market.start_time_utc);
    msg!("END_TIME:{}", market.end_time_utc);
    msg!("DURATION_SECONDS:{}", market.duration_seconds);
    msg!("DATA_PROVIDER:{}", market.data_provider);
    msg!("CREATED_AT:{}", market.created_at);
    msg!("ASSET:{}", market.asset);

    if let Some(direction) = market.direction {
        msg!(
            "DIRECTION:{}",
            match direction {
                Direction::Above => "Above",
                Direction::Below => "Below",
            }
        );
    }
    if let Some(target) = market.target_price {
        msg!("TARGET_PRICE:{}", target);
    }
    if let Some(current) = market.current_price {
        msg!("CURRENT_PRICE:{}", current);
    }
    if let Some(end) = market.end_price {
        msg!("END_PRICE:{}", end);
    }
    if let Some(outcome) = market.outcome {
        msg!(
            "OUTCOME:{}",
            match outcome {
                Outcome::Yes => "YES",
                Outcome::No => "NO",
            }
        );
    }
    if !market.outcome_description.is_empty() {
        msg!("OUTCOME_DESCRIPTION:{}", market.outcome_description);
    }

    msg!("GetMarket complete. MarketID={}", market_id);
    Ok(())
}
