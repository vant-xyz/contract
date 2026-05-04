#![deny(unused_must_use)]
#![deny(clippy::arithmetic_side_effects)]

use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};
#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;
pub mod validation;

use constants::MAX_MARKET_ID_LEN;
use error::MarketError;
use instructions::{
    process_cancel_vs_event, process_confirm_vs_outcome, process_create_market_cappm,
    process_create_market_gem, process_create_vs_event, process_delegate_market,
    process_get_market, process_join_vs_event, process_resolve_vs_event,
    process_settle_market_cappm, process_settle_market_gem, process_undelegate_callback,
    UNDELEGATE_CALLBACK_DISCRIMINATOR,
};
use utils::read_string;

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "Vantic OVM Program",
    project_url: "https://vantic.xyz",
    contacts: "email:security@vantic.xyz",
    policy: "https://vantic.xyz/privacy-policy",
    preferred_languages: "en",
    source_code: "https://github.com/vant-xyz/contract",
    source_revision: "682f894543bfe69380f8d157873266d5c22ea36f"
}

pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.len() >= UNDELEGATE_CALLBACK_DISCRIMINATOR.len()
        && instruction_data[..UNDELEGATE_CALLBACK_DISCRIMINATOR.len()]
            == UNDELEGATE_CALLBACK_DISCRIMINATOR
    {
        msg!("Dispatching UndelegateCallback");
        return process_undelegate_callback(program_id, accounts, instruction_data);
    }

    if instruction_data.is_empty() {
        msg!("Error: Empty instruction data");
        return Err(MarketError::InvalidInstructionData.into());
    }

    let discriminator = instruction_data[0];
    let data = &instruction_data[1..];

    msg!(
        "Vant: processing instruction discriminator={}",
        discriminator
    );

    match discriminator {
        0 => {
            msg!("Dispatching CreateMarketCAPPM");
            process_create_market_cappm(program_id, accounts, data)
        }
        1 => {
            msg!("Dispatching CreateMarketGEM");
            process_create_market_gem(program_id, accounts, data)
        }
        2 => {
            msg!("Dispatching SettleMarketCAPPM");
            let mut offset = 0usize;
            let market_id = read_string(data, &mut offset, MAX_MARKET_ID_LEN).map_err(|_| {
                msg!("Failed to read market_id from settle CAPPM data");
                MarketError::InvalidInstructionData
            })?;
            process_settle_market_cappm(program_id, accounts, &data[offset..], &market_id)
        }
        3 => {
            msg!("Dispatching SettleMarketGEM");
            let mut offset = 0usize;
            let market_id = read_string(data, &mut offset, MAX_MARKET_ID_LEN).map_err(|_| {
                msg!("Failed to read market_id from settle GEM data");
                MarketError::InvalidInstructionData
            })?;
            process_settle_market_gem(program_id, accounts, &data[offset..], &market_id)
        }
        4 => {
            msg!("Dispatching GetMarket");
            let mut offset = 0usize;
            let market_id = read_string(data, &mut offset, MAX_MARKET_ID_LEN).map_err(|_| {
                msg!("Failed to read market_id from get market data");
                MarketError::InvalidInstructionData
            })?;
            process_get_market(program_id, accounts, &market_id)
        }
        5 => {
            msg!("Dispatching DelegateMarket");
            process_delegate_market(program_id, accounts, data)
        }
        6 => {
            msg!("Dispatching CreateVSEvent");
            process_create_vs_event(program_id, accounts, data)
        }
        7 => {
            msg!("Dispatching JoinVSEvent");
            process_join_vs_event(program_id, accounts, data)
        }
        8 => {
            msg!("Dispatching ConfirmVSOutcome");
            process_confirm_vs_outcome(program_id, accounts, data)
        }
        9 => {
            msg!("Dispatching ResolveVSEvent");
            process_resolve_vs_event(program_id, accounts, data)
        }
        10 => {
            msg!("Dispatching CancelVSEvent");
            process_cancel_vs_event(program_id, accounts, data)
        }
        _ => {
            msg!(
                "Error: Unknown instruction discriminator: {}",
                discriminator
            );
            Err(MarketError::InvalidInstructionData.into())
        }
    }
}
