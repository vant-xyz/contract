use solana_program::{msg, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    constants::{
        MAX_ASSET_LEN, MAX_DATA_PROVIDER_LEN, MAX_DESCRIPTION_LEN,
        MAX_OUTCOME_DESCRIPTION_LEN, MAX_TITLE_LEN,
    },
    error::MarketError,
    utils::*,
};

#[derive(Debug, Clone, PartialEq, Copy)]
#[repr(u8)]
pub enum MarketType {
    CAPPM = 0,
    GEM   = 1,
}

impl MarketType {
    pub fn from_u8(val: u8) -> Result<Self, ProgramError> {
        match val {
            0 => Ok(MarketType::CAPPM),
            1 => Ok(MarketType::GEM),
            _ => {
                msg!("Invalid MarketType byte: {}", val);
                Err(MarketError::InvalidMarketType.into())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
#[repr(u8)]
pub enum Direction {
    Above = 0,
    Below = 1,
}

impl Direction {
    pub fn from_u8(val: u8) -> Result<Self, ProgramError> {
        match val {
            0 => Ok(Direction::Above),
            1 => Ok(Direction::Below),
            _ => {
                msg!("Invalid Direction byte: {}", val);
                Err(MarketError::InvalidDirection.into())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
#[repr(u8)]
pub enum Outcome {
    Yes = 0,
    No  = 1,
}

impl Outcome {
    pub fn from_u8(val: u8) -> Result<Self, ProgramError> {
        match val {
            0 => Ok(Outcome::Yes),
            1 => Ok(Outcome::No),
            _ => {
                msg!("Invalid Outcome byte: {}", val);
                Err(MarketError::InvalidOutcome.into())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Market {
    pub market_type:         MarketType,
    pub is_resolved:         bool,
    pub creator:             Pubkey,
    pub approved_settler:    Pubkey,
    pub title:               String,
    pub description:         String,
    pub start_time_utc:      u64,
    pub end_time_utc:        u64,
    pub duration_seconds:    u64,
    pub data_provider:       String,
    pub created_at:          u64,
    pub bump:                u8,
    // CAPPM-specific
    pub asset:               String,        // e.g. "BTC", "ETH", "SOL"
    pub direction:           Option<Direction>,
    pub target_price:        Option<u64>,
    pub current_price:       Option<u64>,
    pub end_price:           Option<u64>,
    // Resolution
    pub outcome:             Option<Outcome>,
    pub outcome_description: String,
}

impl Market {
    pub fn pack(&self, dst: &mut [u8]) -> Result<(), ProgramError> {
        let mut offset = 0usize;

        write_u8(dst, &mut offset, self.market_type as u8)?;
        write_bool(dst, &mut offset, self.is_resolved)?;
        write_pubkey(dst, &mut offset, &self.creator)?;
        write_pubkey(dst, &mut offset, &self.approved_settler)?;
        write_string(dst, &mut offset, &self.title, MAX_TITLE_LEN)?;
        write_string(dst, &mut offset, &self.description, MAX_DESCRIPTION_LEN)?;
        write_u64(dst, &mut offset, self.start_time_utc)?;
        write_u64(dst, &mut offset, self.end_time_utc)?;
        write_u64(dst, &mut offset, self.duration_seconds)?;
        write_string(dst, &mut offset, &self.data_provider, MAX_DATA_PROVIDER_LEN)?;
        write_u64(dst, &mut offset, self.created_at)?;
        write_u8(dst, &mut offset, self.bump)?;
        write_string(dst, &mut offset, &self.asset, MAX_ASSET_LEN)?;

        match self.direction {
            Some(d) => {
                write_u8(dst, &mut offset, 1)?;
                write_u8(dst, &mut offset, d as u8)?;
            }
            None => {
                write_u8(dst, &mut offset, 0)?;
                write_u8(dst, &mut offset, 0)?;
            }
        }

        match self.target_price {
            Some(p) => {
                write_u8(dst, &mut offset, 1)?;
                write_u64(dst, &mut offset, p)?;
            }
            None => {
                write_u8(dst, &mut offset, 0)?;
                write_u64(dst, &mut offset, 0)?;
            }
        }

        match self.current_price {
            Some(p) => {
                write_u8(dst, &mut offset, 1)?;
                write_u64(dst, &mut offset, p)?;
            }
            None => {
                write_u8(dst, &mut offset, 0)?;
                write_u64(dst, &mut offset, 0)?;
            }
        }

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

        match self.outcome {
            Some(o) => {
                write_u8(dst, &mut offset, 1)?;
                write_u8(dst, &mut offset, o as u8)?;
            }
            None => {
                write_u8(dst, &mut offset, 0)?;
                write_u8(dst, &mut offset, 0)?;
            }
        }

        write_string(dst, &mut offset, &self.outcome_description, MAX_OUTCOME_DESCRIPTION_LEN)?;

        msg!("Market packed: {} bytes used", offset);
        Ok(())
    }

    pub fn unpack(src: &[u8]) -> Result<Self, ProgramError> {
        if src.is_empty() {
            msg!("Market::unpack: empty data");
            return Err(MarketError::UninitializedAccount.into());
        }

        let mut offset = 0usize;

        let market_type      = MarketType::from_u8(read_u8(src, &mut offset)?)?;
        let is_resolved      = read_bool(src, &mut offset)?;
        let creator          = read_pubkey(src, &mut offset)?;
        let approved_settler = read_pubkey(src, &mut offset)?;
        let title            = read_string(src, &mut offset, MAX_TITLE_LEN)?;
        let description      = read_string(src, &mut offset, MAX_DESCRIPTION_LEN)?;
        let start_time_utc   = read_u64(src, &mut offset)?;
        let end_time_utc     = read_u64(src, &mut offset)?;
        let duration_seconds = read_u64(src, &mut offset)?;
        let data_provider    = read_string(src, &mut offset, MAX_DATA_PROVIDER_LEN)?;
        let created_at       = read_u64(src, &mut offset)?;
        let bump             = read_u8(src, &mut offset)?;
        let asset            = read_string(src, &mut offset, MAX_ASSET_LEN)?;

        let direction_present = read_u8(src, &mut offset)?;
        let direction_val     = read_u8(src, &mut offset)?;
        let direction = if direction_present == 1 {
            Some(Direction::from_u8(direction_val)?)
        } else {
            None
        };

        let target_present = read_u8(src, &mut offset)?;
        let target_val     = read_u64(src, &mut offset)?;
        let target_price   = if target_present == 1 { Some(target_val) } else { None };

        let current_present = read_u8(src, &mut offset)?;
        let current_val     = read_u64(src, &mut offset)?;
        let current_price   = if current_present == 1 { Some(current_val) } else { None };

        let end_price_present = read_u8(src, &mut offset)?;
        let end_price_val     = read_u64(src, &mut offset)?;
        let end_price         = if end_price_present == 1 { Some(end_price_val) } else { None };

        let outcome_present = read_u8(src, &mut offset)?;
        let outcome_val     = read_u8(src, &mut offset)?;
        let outcome = if outcome_present == 1 {
            Some(Outcome::from_u8(outcome_val)?)
        } else {
            None
        };

        let outcome_description = read_string(src, &mut offset, MAX_OUTCOME_DESCRIPTION_LEN)?;

        Ok(Market {
            market_type,
            is_resolved,
            creator,
            approved_settler,
            title,
            description,
            start_time_utc,
            end_time_utc,
            duration_seconds,
            data_provider,
            created_at,
            bump,
            asset,
            direction,
            target_price,
            current_price,
            end_price,
            outcome,
            outcome_description,
        })
    }
}