use solana_program::program_error::ProgramError;

#[repr(u32)]
#[derive(Debug, Clone, PartialEq)]
pub enum MarketError {
    InvalidAccountCount = 0,
    InvalidSigner = 1,
    InvalidWritable = 2,
    InvalidOwner = 3,
    UninitializedAccount = 4,
    InvalidMarketType = 5,
    MarketAlreadyResolved = 6,
    UnauthorizedSettler = 7,
    InvalidSettlementSignature = 8,
    MarketNotExpired = 9,
    InvalidDataProvider = 10,
    InvalidTargetPrice = 11,
    InvalidEndTime = 12,
    ArithmeticOverflow = 13,
    SerializationError = 14,
    InvalidInstructionData = 15,
    InvalidAccountIndex = 16,
    InvalidAccount = 17,
    MarketNotStarted = 18,
    InvalidDirection = 19,
    InvalidSettlerPubkey = 20,
    InvalidPDA = 21,
    MarketNotResolvable = 22,
    InvalidOutcome = 23,
    VSParticipantAlreadyJoined = 24,
    VSParticipantLimitReached = 25,
    VSDuplicateVote = 26,
}

impl From<MarketError> for ProgramError {
    fn from(e: MarketError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl From<MarketError> for u32 {
    fn from(e: MarketError) -> u32 {
        e as u32
    }
}
