use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

use crate::error::MarketError;

pub const UNDELEGATE_CALLBACK_DISCRIMINATOR: [u8; 8] = [196, 28, 41, 206, 48, 37, 51, 167];

pub fn process_undelegate_callback<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    data: &[u8],
) -> ProgramResult {
    if data.len() < UNDELEGATE_CALLBACK_DISCRIMINATOR.len()
        || data[..UNDELEGATE_CALLBACK_DISCRIMINATOR.len()] != UNDELEGATE_CALLBACK_DISCRIMINATOR
    {
        msg!("Invalid undelegate callback discriminator payload");
        return Err(MarketError::InvalidInstructionData.into());
    }

    msg!(
        "MagicBlock undelegate callback handled (accounts={})",
        accounts.len()
    );

    Ok(())
}
