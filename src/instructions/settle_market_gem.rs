use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::invoke,
    pubkey::Pubkey,
    sysvar::instructions::ID as INSTRUCTIONS_SYSVAR_ID,
};

use crate::{
    constants::{APPROVED_SETTLER, MARKET_SEED, MAX_OUTCOME_DESCRIPTION_LEN},
    error::MarketError,
    state::{Market, Outcome},
    utils::{current_timestamp, read_signature, read_string, read_u8, verify_settlement_signature_via_sysvar},
    validation::{validate_accounts, verify_pda},
};

const MAGIC_PROGRAM_ID: Pubkey =
    solana_program::pubkey!("Magic11111111111111111111111111111111111111");
const MAGIC_CONTEXT_ID: Pubkey =
    solana_program::pubkey!("MagicContext1111111111111111111111111111111");

const COMMIT_AND_UNDELEGATE_DATA: [u8; 4] = [2, 0, 0, 0];

pub fn process_settle_market_gem<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    data: &[u8],
    market_id: &str,
) -> ProgramResult {
    msg!("=== SettleMarketGEM === MarketID={}", market_id);

    // accounts: market_account, settler, instructions_sysvar, magic_program, magic_context
    validate_accounts(accounts, 5, false, &[0, 1, 4])?;

    let accounts_iter       = &mut accounts.iter();
    let market_account      = next_account_info(accounts_iter)?;
    let settler             = next_account_info(accounts_iter)?;
    let instructions_sysvar = next_account_info(accounts_iter)?;
    let magic_program       = next_account_info(accounts_iter)?;
    let magic_context       = next_account_info(accounts_iter)?;

    if !settler.is_signer {
        msg!("Settler must be a signer");
        return Err(MarketError::InvalidSigner.into());
    }

    if settler.key != &APPROVED_SETTLER {
        msg!("Unauthorized settler: expected {}, got {}", APPROVED_SETTLER, settler.key);
        return Err(MarketError::UnauthorizedSettler.into());
    }

    if instructions_sysvar.key != &INSTRUCTIONS_SYSVAR_ID {
        msg!("Invalid instructions sysvar account");
        return Err(MarketError::InvalidAccount.into());
    }

    if magic_program.key != &MAGIC_PROGRAM_ID {
        msg!("Invalid magic_program account");
        return Err(MarketError::InvalidAccount.into());
    }

    if magic_context.key != &MAGIC_CONTEXT_ID {
        msg!("Invalid magic_context account");
        return Err(MarketError::InvalidAccount.into());
    }

    if data.is_empty() {
        msg!("Empty instruction data");
        return Err(MarketError::InvalidInstructionData.into());
    }

    let mut offset = 0usize;

    let outcome_byte         = read_u8(data, &mut offset)?;
    let outcome              = Outcome::from_u8(outcome_byte)?;
    let outcome_description  = read_string(data, &mut offset, MAX_OUTCOME_DESCRIPTION_LEN)?;
    let _settlement_signature = read_signature(data, &mut offset)?;

    msg!("Outcome: {:?}", outcome);
    msg!("Description: {}", outcome_description);

    let market_id_bytes = market_id.as_bytes();
    verify_pda(market_account, &[MARKET_SEED, market_id_bytes], program_id)?;

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
        msg!("Market {} has not expired yet (end={}, now={})", market_id, market.end_time_utc, now);
        return Err(MarketError::MarketNotExpired.into());
    }

    if settler.key != &market.approved_settler {
        msg!("Settler {} does not match market.approved_settler {}", settler.key, market.approved_settler);
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
    market.outcome_description = outcome_description;
    market.end_price           = None;

    {
        let mut account_data = market_account.try_borrow_mut_data()?;
        market.pack(&mut account_data)?;
    }

    msg!("Market state updated: is_resolved=true, outcome={:?}", outcome);

    let commit_ix = Instruction {
        program_id: MAGIC_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*settler.key, true),
            AccountMeta::new(*magic_context.key, false),
            AccountMeta::new(*market_account.key, false),
        ],
        data: COMMIT_AND_UNDELEGATE_DATA.to_vec(),
    };

    invoke(
        &commit_ix,
        &[settler.clone(), magic_context.clone(), market_account.clone(), magic_program.clone()],
    )?;

    msg!("SettleMarketGEM complete. MarketID={}, Outcome={:?}", market_id, outcome);
    Ok(())
}
