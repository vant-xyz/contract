use solana_program::{
    account_info::AccountInfo, ed25519_program, hash::hashv, msg, program_error::ProgramError,
    pubkey::Pubkey, sysvar::clock::Clock, sysvar::Sysvar,
};

use crate::error::MarketError;

#[inline]
pub fn write_u8(dst: &mut [u8], offset: &mut usize, val: u8) -> Result<(), ProgramError> {
    if *offset >= dst.len() {
        msg!("write_u8: buffer overflow at offset {}", offset);
        return Err(MarketError::SerializationError.into());
    }
    dst[*offset] = val;
    *offset = offset
        .checked_add(1)
        .ok_or(MarketError::ArithmeticOverflow)?;
    Ok(())
}

#[inline]
pub fn write_bool(dst: &mut [u8], offset: &mut usize, val: bool) -> Result<(), ProgramError> {
    write_u8(dst, offset, val as u8)
}

#[inline]
pub fn write_u64(dst: &mut [u8], offset: &mut usize, val: u64) -> Result<(), ProgramError> {
    let end = offset
        .checked_add(8)
        .ok_or(MarketError::ArithmeticOverflow)?;
    if end > dst.len() {
        msg!("write_u64: buffer overflow at offset {}", offset);
        return Err(MarketError::SerializationError.into());
    }
    dst[*offset..end].copy_from_slice(&val.to_le_bytes());
    *offset = end;
    Ok(())
}

#[inline]
pub fn write_pubkey(dst: &mut [u8], offset: &mut usize, key: &Pubkey) -> Result<(), ProgramError> {
    let end = offset
        .checked_add(32)
        .ok_or(MarketError::ArithmeticOverflow)?;
    if end > dst.len() {
        msg!("write_pubkey: buffer overflow at offset {}", offset);
        return Err(MarketError::SerializationError.into());
    }
    dst[*offset..end].copy_from_slice(key.as_ref());
    *offset = end;
    Ok(())
}

#[inline]
pub fn write_string(
    dst: &mut [u8],
    offset: &mut usize,
    s: &str,
    max_len: usize,
) -> Result<(), ProgramError> {
    let bytes = s.as_bytes();
    let len = bytes.len().min(max_len);
    let len_u16 = len as u16;

    let end = offset
        .checked_add(2)
        .and_then(|e| e.checked_add(len))
        .ok_or(MarketError::ArithmeticOverflow)?;

    if end > dst.len() {
        msg!("write_string: buffer overflow at offset {}", offset);
        return Err(MarketError::SerializationError.into());
    }

    dst[*offset..*offset + 2].copy_from_slice(&len_u16.to_le_bytes());
    *offset = offset
        .checked_add(2)
        .ok_or(MarketError::ArithmeticOverflow)?;
    dst[*offset..*offset + len].copy_from_slice(&bytes[..len]);
    *offset = offset
        .checked_add(len)
        .ok_or(MarketError::ArithmeticOverflow)?;
    Ok(())
}

#[inline]
pub fn write_bytes32(
    dst: &mut [u8],
    offset: &mut usize,
    val: &[u8; 32],
) -> Result<(), ProgramError> {
    let end = offset
        .checked_add(32)
        .ok_or(MarketError::ArithmeticOverflow)?;
    if end > dst.len() {
        msg!("write_bytes32: buffer overflow at offset {}", offset);
        return Err(MarketError::SerializationError.into());
    }
    dst[*offset..end].copy_from_slice(val);
    *offset = end;
    Ok(())
}

#[inline]
pub fn read_u8(src: &[u8], offset: &mut usize) -> Result<u8, ProgramError> {
    if *offset >= src.len() {
        msg!("read_u8: buffer underflow at offset {}", offset);
        return Err(MarketError::SerializationError.into());
    }
    let val = src[*offset];
    *offset = offset
        .checked_add(1)
        .ok_or(MarketError::ArithmeticOverflow)?;
    Ok(val)
}

#[inline]
pub fn read_bool(src: &[u8], offset: &mut usize) -> Result<bool, ProgramError> {
    Ok(read_u8(src, offset)? != 0)
}

#[inline]
pub fn read_u64(src: &[u8], offset: &mut usize) -> Result<u64, ProgramError> {
    let end = offset
        .checked_add(8)
        .ok_or(MarketError::ArithmeticOverflow)?;
    if end > src.len() {
        msg!("read_u64: buffer underflow at offset {}", offset);
        return Err(MarketError::SerializationError.into());
    }
    let bytes: [u8; 8] = src[*offset..end]
        .try_into()
        .map_err(|_| MarketError::SerializationError)?;
    *offset = end;
    Ok(u64::from_le_bytes(bytes))
}

#[inline]
pub fn read_pubkey(src: &[u8], offset: &mut usize) -> Result<Pubkey, ProgramError> {
    let end = offset
        .checked_add(32)
        .ok_or(MarketError::ArithmeticOverflow)?;
    if end > src.len() {
        msg!("read_pubkey: buffer underflow at offset {}", offset);
        return Err(MarketError::SerializationError.into());
    }
    let bytes: [u8; 32] = src[*offset..end]
        .try_into()
        .map_err(|_| MarketError::SerializationError)?;
    *offset = end;
    Ok(Pubkey::from(bytes))
}

#[inline]
pub fn read_string(src: &[u8], offset: &mut usize, max_len: usize) -> Result<String, ProgramError> {
    let len_end = offset
        .checked_add(2)
        .ok_or(MarketError::ArithmeticOverflow)?;
    if len_end > src.len() {
        msg!(
            "read_string: buffer underflow reading length at offset {}",
            offset
        );
        return Err(MarketError::SerializationError.into());
    }
    let len = u16::from_le_bytes(
        src[*offset..len_end]
            .try_into()
            .map_err(|_| MarketError::SerializationError)?,
    ) as usize;
    *offset = len_end;

    if len > max_len {
        msg!("read_string: string length {} exceeds max {}", len, max_len);
        return Err(MarketError::SerializationError.into());
    }

    let str_end = offset
        .checked_add(len)
        .ok_or(MarketError::ArithmeticOverflow)?;
    if str_end > src.len() {
        msg!(
            "read_string: buffer underflow reading string data at offset {}",
            offset
        );
        return Err(MarketError::SerializationError.into());
    }

    let s = core::str::from_utf8(&src[*offset..str_end])
        .map_err(|_| MarketError::SerializationError)?
        .to_string();
    *offset = str_end;
    Ok(s)
}

#[inline]
pub fn read_bytes32(src: &[u8], offset: &mut usize) -> Result<[u8; 32], ProgramError> {
    let end = offset
        .checked_add(32)
        .ok_or(MarketError::ArithmeticOverflow)?;
    if end > src.len() {
        msg!("read_bytes32: buffer underflow at offset {}", offset);
        return Err(MarketError::SerializationError.into());
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&src[*offset..end]);
    *offset = end;
    Ok(arr)
}

pub fn sha256(data: &[u8]) -> [u8; 32] {
    hashv(&[data]).to_bytes()
}

pub fn verify_settlement_signature_via_sysvar(
    instructions_sysvar: &AccountInfo,
    expected_signer: &Pubkey,
    expected_message: &[u8],
) -> Result<(), ProgramError> {
    use solana_program::sysvar::instructions::{
        load_current_index_checked, load_instruction_at_checked,
    };

    let current_index = load_current_index_checked(instructions_sysvar)
        .map_err(|_| MarketError::InvalidSettlementSignature)?;

    msg!("Current instruction index: {}", current_index);

    if current_index == 0 {
        msg!("No prior instruction found — ed25519 verify instruction missing");
        return Err(MarketError::InvalidSettlementSignature.into());
    }

    let ed25519_ix_index = current_index
        .checked_sub(1)
        .ok_or(MarketError::InvalidSettlementSignature)?;

    let ed25519_ix = load_instruction_at_checked(ed25519_ix_index as usize, instructions_sysvar)
        .map_err(|_| {
            msg!(
                "Failed to load ed25519 instruction at index {}",
                ed25519_ix_index
            );
            MarketError::InvalidSettlementSignature
        })?;
    if ed25519_ix.program_id != ed25519_program::id() {
        msg!(
            "Instruction at index {} is not ed25519_program (got {})",
            ed25519_ix_index,
            ed25519_ix.program_id
        );
        return Err(MarketError::InvalidSettlementSignature.into());
    }

    let data = &ed25519_ix.data;

    if data.len() < 16 {
        msg!("ed25519 instruction data too short: {} bytes", data.len());
        return Err(MarketError::InvalidSettlementSignature.into());
    }

    let num_signatures = data[0];
    if num_signatures != 1 {
        msg!("Expected exactly 1 signature, got {}", num_signatures);
        return Err(MarketError::InvalidSettlementSignature.into());
    }

    let pubkey_offset = u16::from_le_bytes([data[6], data[7]]) as usize;
    let msg_offset = u16::from_le_bytes([data[10], data[11]]) as usize;
    let msg_size = u16::from_le_bytes([data[12], data[13]]) as usize;

    let pubkey_end = pubkey_offset
        .checked_add(32)
        .ok_or(MarketError::InvalidSettlementSignature)?;
    if pubkey_end > data.len() {
        msg!("ed25519 data too short for pubkey");
        return Err(MarketError::InvalidSettlementSignature.into());
    }
    let pubkey_bytes: &[u8; 32] = data[pubkey_offset..pubkey_end]
        .try_into()
        .map_err(|_| MarketError::InvalidSettlementSignature)?;
    let signer_in_ix = Pubkey::from(*pubkey_bytes);

    if &signer_in_ix != expected_signer {
        msg!(
            "Pubkey mismatch: expected {}, got {}",
            expected_signer,
            signer_in_ix
        );
        return Err(MarketError::InvalidSettlementSignature.into());
    }

    let msg_end = msg_offset
        .checked_add(msg_size)
        .ok_or(MarketError::InvalidSettlementSignature)?;
    if msg_end > data.len() {
        msg!("ed25519 data too short for message");
        return Err(MarketError::InvalidSettlementSignature.into());
    }
    let message_in_ix = &data[msg_offset..msg_end];

    if message_in_ix != expected_message {
        msg!("Message mismatch in ed25519 instruction");
        msg!("Expected: {:?}", expected_message);
        msg!("Got:      {:?}", message_in_ix);
        return Err(MarketError::InvalidSettlementSignature.into());
    }

    msg!("Ed25519 signature verified successfully via instructions sysvar");
    Ok(())
}

pub fn current_timestamp() -> Result<u64, ProgramError> {
    let clock = Clock::get().map_err(|e| {
        msg!("Failed to get Clock sysvar: {:?}", e);
        e
    })?;
    Ok(clock.unix_timestamp as u64)
}

pub fn read_signature(src: &[u8], offset: &mut usize) -> Result<[u8; 64], ProgramError> {
    let end = offset
        .checked_add(64)
        .ok_or(MarketError::ArithmeticOverflow)?;
    if end > src.len() {
        msg!("read_signature: buffer underflow at offset {}", offset);
        return Err(MarketError::InvalidInstructionData.into());
    }
    let mut sig = [0u8; 64];
    sig.copy_from_slice(&src[*offset..end]);
    *offset = end;
    Ok(sig)
}
