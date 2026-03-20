use solana_program::{msg, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    constants::MAX_OUTCOME_DESCRIPTION_LEN,
    error::MarketError,
    state::market::Outcome,
    utils::*,
};

#[derive(Debug, Clone)]
pub struct SettlementLog {
    
    pub market: Pubkey,
    pub settled_at: u64,
    pub settled_by: Pubkey,
    pub end_price: Option<u64>,
    pub outcome: Outcome,
    pub outcome_description: String,
    pub signature_hash: [u8; 32],
    pub message_hash: [u8; 32],
    pub bump: u8,
}

impl SettlementLog {
    pub fn pack(&self, dst: &mut [u8]) -> Result<(), ProgramError> {
        let mut offset = 0usize;

        write_pubkey(dst, &mut offset, &self.market)?;
        write_u64(dst, &mut offset, self.settled_at)?;
        write_pubkey(dst, &mut offset, &self.settled_by)?;

        match self.end_price {
            Some(p) => {
                write_u8(dst, &mut offset, 1)?;
                write_u64(dst, &mut offset, p)?;
            }
            None => {
                write_u8(dst, &mut offset, 0)?;
                write_u64(dst, &mut offset, 0)?;
            }
        }

        write_u8(dst, &mut offset, self.outcome as u8)?;
        write_string(dst, &mut offset, &self.outcome_description, MAX_OUTCOME_DESCRIPTION_LEN)?;
        write_bytes32(dst, &mut offset, &self.signature_hash)?;
        write_bytes32(dst, &mut offset, &self.message_hash)?;
        write_u8(dst, &mut offset, self.bump)?;

        msg!("SettlementLog packed: {} bytes used", offset);
        Ok(())
    }

    
    pub fn unpack(src: &[u8]) -> Result<Self, ProgramError> {
        if src.is_empty() {
            msg!("SettlementLog::unpack: empty data");
            return Err(MarketError::UninitializedAccount.into());
        }

        let mut offset = 0usize;

        let market     = read_pubkey(src, &mut offset)?;
        let settled_at = read_u64(src, &mut offset)?;
        let settled_by = read_pubkey(src, &mut offset)?;

        let end_price_present = read_u8(src, &mut offset)?;
        let end_price_val     = read_u64(src, &mut offset)?;
        let end_price         = if end_price_present == 1 { Some(end_price_val) } else { None };

        let outcome_byte        = read_u8(src, &mut offset)?;
        let outcome             = Outcome::from_u8(outcome_byte)?;
        let outcome_description = read_string(src, &mut offset, MAX_OUTCOME_DESCRIPTION_LEN)?;
        let signature_hash      = read_bytes32(src, &mut offset)?;
        let message_hash        = read_bytes32(src, &mut offset)?;
        let bump                = read_u8(src, &mut offset)?;

        Ok(SettlementLog {
            market,
            settled_at,
            settled_by,
            end_price,
            outcome,
            outcome_description,
            signature_hash,
            message_hash,
            bump,
        })
    }
}