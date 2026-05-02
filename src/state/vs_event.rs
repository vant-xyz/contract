use solana_program::{msg, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    constants::{
        MAX_MARKET_ID_LEN, MAX_OUTCOME_DESCRIPTION_LEN, MAX_TITLE_LEN, MAX_VS_PARTICIPANTS,
    },
    error::MarketError,
    utils::*,
};

#[derive(Debug, Clone, PartialEq, Copy)]
#[repr(u8)]
pub enum VSMode {
    Mutual = 0,
    Consensus = 1,
}

impl VSMode {
    pub fn from_u8(v: u8) -> Result<Self, ProgramError> {
        match v {
            0 => Ok(VSMode::Mutual),
            1 => Ok(VSMode::Consensus),
            _ => Err(MarketError::InvalidInstructionData.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
#[repr(u8)]
pub enum VSStatus {
    Open = 0,
    Active = 1,
    Resolved = 2,
    Cancelled = 3,
    Disputed = 4,
}

impl VSStatus {
    pub fn from_u8(v: u8) -> Result<Self, ProgramError> {
        match v {
            0 => Ok(VSStatus::Open),
            1 => Ok(VSStatus::Active),
            2 => Ok(VSStatus::Resolved),
            3 => Ok(VSStatus::Cancelled),
            4 => Ok(VSStatus::Disputed),
            _ => Err(MarketError::InvalidInstructionData.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VSEvent {
    pub vs_id: String,
    pub creator: Pubkey,
    pub title: String,
    pub stake_cents: u64,
    pub mode: VSMode,
    pub threshold: u8,
    pub status: VSStatus,
    pub created_at: u64,
    pub join_deadline_utc: u64,
    pub resolve_deadline_utc: u64,
    pub participant_count: u8,
    pub participants: Vec<Pubkey>,
    pub outcome: Option<u8>,
    pub outcome_description: String,
    pub votes_yes: Vec<Pubkey>,
    pub votes_no: Vec<Pubkey>,
    pub bump: u8,
}

impl VSEvent {
    pub fn pack(&self, dst: &mut [u8]) -> Result<(), ProgramError> {
        let mut o = 0usize;

        write_string(dst, &mut o, &self.vs_id, MAX_MARKET_ID_LEN)?;
        write_pubkey(dst, &mut o, &self.creator)?;
        write_string(dst, &mut o, &self.title, MAX_TITLE_LEN)?;
        write_u64(dst, &mut o, self.stake_cents)?;
        write_u8(dst, &mut o, self.mode as u8)?;
        write_u8(dst, &mut o, self.threshold)?;
        write_u8(dst, &mut o, self.status as u8)?;
        write_u64(dst, &mut o, self.created_at)?;
        write_u64(dst, &mut o, self.join_deadline_utc)?;
        write_u64(dst, &mut o, self.resolve_deadline_utc)?;
        write_u8(dst, &mut o, self.participant_count)?;

        if self.participants.len() > MAX_VS_PARTICIPANTS {
            return Err(MarketError::InvalidInstructionData.into());
        }
        write_u8(dst, &mut o, self.participants.len() as u8)?;
        for p in &self.participants {
            write_pubkey(dst, &mut o, p)?;
        }

        match self.outcome {
            Some(v) => {
                write_u8(dst, &mut o, 1)?;
                write_u8(dst, &mut o, v)?;
            }
            None => {
                write_u8(dst, &mut o, 0)?;
                write_u8(dst, &mut o, 0)?;
            }
        }

        write_string(
            dst,
            &mut o,
            &self.outcome_description,
            MAX_OUTCOME_DESCRIPTION_LEN,
        )?;

        if self.votes_yes.len() > MAX_VS_PARTICIPANTS || self.votes_no.len() > MAX_VS_PARTICIPANTS {
            return Err(MarketError::InvalidInstructionData.into());
        }
        write_u8(dst, &mut o, self.votes_yes.len() as u8)?;
        for p in &self.votes_yes {
            write_pubkey(dst, &mut o, p)?;
        }
        write_u8(dst, &mut o, self.votes_no.len() as u8)?;
        for p in &self.votes_no {
            write_pubkey(dst, &mut o, p)?;
        }

        write_u8(dst, &mut o, self.bump)?;
        msg!("VSEvent packed: {} bytes used", o);
        Ok(())
    }

    pub fn unpack(src: &[u8]) -> Result<Self, ProgramError> {
        if src.is_empty() {
            return Err(MarketError::UninitializedAccount.into());
        }

        let mut o = 0usize;
        let vs_id = read_string(src, &mut o, MAX_MARKET_ID_LEN)?;
        let creator = read_pubkey(src, &mut o)?;
        let title = read_string(src, &mut o, MAX_TITLE_LEN)?;
        let stake_cents = read_u64(src, &mut o)?;
        let mode = VSMode::from_u8(read_u8(src, &mut o)?)?;
        let threshold = read_u8(src, &mut o)?;
        let status = VSStatus::from_u8(read_u8(src, &mut o)?)?;
        let created_at = read_u64(src, &mut o)?;
        let join_deadline_utc = read_u64(src, &mut o)?;
        let resolve_deadline_utc = read_u64(src, &mut o)?;
        let participant_count = read_u8(src, &mut o)?;

        let participants_len = read_u8(src, &mut o)? as usize;
        if participants_len > MAX_VS_PARTICIPANTS {
            return Err(MarketError::InvalidInstructionData.into());
        }
        let mut participants = Vec::with_capacity(participants_len);
        for _ in 0..participants_len {
            participants.push(read_pubkey(src, &mut o)?);
        }

        let outcome_present = read_u8(src, &mut o)?;
        let outcome_val = read_u8(src, &mut o)?;
        let outcome = if outcome_present == 1 {
            Some(outcome_val)
        } else {
            None
        };

        let outcome_description = read_string(src, &mut o, MAX_OUTCOME_DESCRIPTION_LEN)?;

        let yes_len = read_u8(src, &mut o)? as usize;
        if yes_len > MAX_VS_PARTICIPANTS {
            return Err(MarketError::InvalidInstructionData.into());
        }
        let mut votes_yes = Vec::with_capacity(yes_len);
        for _ in 0..yes_len {
            votes_yes.push(read_pubkey(src, &mut o)?);
        }

        let no_len = read_u8(src, &mut o)? as usize;
        if no_len > MAX_VS_PARTICIPANTS {
            return Err(MarketError::InvalidInstructionData.into());
        }
        let mut votes_no = Vec::with_capacity(no_len);
        for _ in 0..no_len {
            votes_no.push(read_pubkey(src, &mut o)?);
        }

        let bump = read_u8(src, &mut o)?;

        Ok(Self {
            vs_id,
            creator,
            title,
            stake_cents,
            mode,
            threshold,
            status,
            created_at,
            join_deadline_utc,
            resolve_deadline_utc,
            participant_count,
            participants,
            outcome,
            outcome_description,
            votes_yes,
            votes_no,
            bump,
        })
    }

    pub fn has_participant(&self, pk: &Pubkey) -> bool {
        self.participants.iter().any(|p| p == pk)
    }
}
