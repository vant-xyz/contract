#![deny(unused_must_use)]
#![deny(clippy::arithmetic_side_effects)]

use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;
pub mod validation;

use error::MarketError;
use instructions::{
    process_create_market_cappm, process_create_market_gem,
    process_settle_market_cappm, process_settle_market_gem,
};
use utils::read_string;
use constants::MAX_MARKET_ID_LEN;

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        msg!("Error: Empty instruction data");
        return Err(MarketError::InvalidInstructionData.into());
    }

    let discriminator = instruction_data[0];
    let data          = &instruction_data[1..];

    msg!("Vant: processing instruction discriminator={}", discriminator);

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
            let market_id = read_string(data, &mut offset, MAX_MARKET_ID_LEN)
                .map_err(|_| {
                    msg!("Failed to read market_id from settle CAPPM data");
                    MarketError::InvalidInstructionData
                })?;
            process_settle_market_cappm(program_id, accounts, &data[offset..], &market_id)
        }
        3 => {
            msg!("Dispatching SettleMarketGEM");
            let mut offset = 0usize;
            let market_id = read_string(data, &mut offset, MAX_MARKET_ID_LEN)
                .map_err(|_| {
                    msg!("Failed to read market_id from settle GEM data");
                    MarketError::InvalidInstructionData
                })?;
            process_settle_market_gem(program_id, accounts, &data[offset..], &market_id)
        }

        _ => {
            msg!("Error: Unknown instruction discriminator: {}", discriminator);
            Err(MarketError::InvalidInstructionData.into())
        }
    }
}