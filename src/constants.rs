use solana_program::pubkey::Pubkey;

pub const APPROVED_SETTLER: Pubkey = solana_program::pubkey!("82yc6SY3Hrx1Z4nwQFyy2aVPprqYF3GQvrBZDNzHCaP9");
pub const ER_VALIDATOR: Pubkey = solana_program::pubkey!("MEUGGrYPxKk17hCr7wpT6s8dtNokZj5U2L57vjYMS8e");
pub const MARKET_SEED: &[u8]     = b"market";
pub const SETTLEMENT_SEED: &[u8] = b"settlement";
pub const MARKET_ACCOUNT_SIZE:      usize = 3000;
pub const SETTLEMENT_ACCOUNT_SIZE:  usize = 700;
pub const MAX_TITLE_LEN:               usize = 256;
pub const MAX_DESCRIPTION_LEN:         usize = 1024;
pub const MAX_DATA_PROVIDER_LEN:       usize = 64;
pub const MAX_OUTCOME_DESCRIPTION_LEN: usize = 512;
pub const MAX_MARKET_ID_LEN:           usize = 128;
pub const MAX_ASSET_LEN:               usize = 10;
pub const APPROVED_DATA_PROVIDERS: &[&str] = &["coinbase", "kalshi"];
pub const CU_BUDGET_TARGET: u64 = 150_000;