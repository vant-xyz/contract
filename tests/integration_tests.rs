#[cfg(test)]
mod tests {
    use solana_program::{program_error::ProgramError, pubkey::Pubkey};

    use vant_crypto::{
        constants::{APPROVED_SETTLER, MARKET_ACCOUNT_SIZE, SETTLEMENT_ACCOUNT_SIZE},
        error::MarketError,
        state::{Direction, Market, MarketType, Outcome, SettlementLog, VSEvent, VSMode, VSStatus},
        utils::sha256,
    };

    fn make_market_cappm(
        is_resolved: bool,
        direction: Direction,
        target_price: u64,
        end_time_utc: u64,
        end_price: Option<u64>,
        outcome: Option<Outcome>,
    ) -> Market {
        Market {
            market_type: MarketType::CAPPM,
            is_resolved,
            creator: Pubkey::new_unique(),
            approved_settler: APPROVED_SETTLER,
            title: "Will BTC > $95,000?".to_string(),
            description: "Based on Coinbase BTC-USD price".to_string(),
            start_time_utc: 1_700_000_000,
            end_time_utc,
            duration_seconds: 86400,
            data_provider: "coinbase".to_string(),
            created_at: 1_700_000_000,
            bump: 255,
            direction: Some(direction),
            asset: "BTC".to_string(),
            target_price: Some(target_price),
            current_price: Some(9_400_000),
            end_price,
            outcome,
            outcome_description: String::new(),
        }
    }

    fn make_market_gem(is_resolved: bool) -> Market {
        Market {
            market_type: MarketType::GEM,
            is_resolved,
            creator: Pubkey::new_unique(),
            approved_settler: APPROVED_SETTLER,
            title: "Will Team X win?".to_string(),
            description: "Premier League match".to_string(),
            start_time_utc: 1_700_000_000,
            end_time_utc: 1_700_086_400,
            duration_seconds: 86400,
            data_provider: "kalshi".to_string(),
            created_at: 1_700_000_000,
            bump: 254,
            direction: None,
            target_price: None,
            current_price: None,
            end_price: None,
            asset: String::new(),
            outcome: None,
            outcome_description: String::new(),
        }
    }

    #[test]
    fn test_market_cappm_pack_unpack_roundtrip() {
        let original = make_market_cappm(
            false,
            Direction::Above,
            9_500_000,
            1_800_000_000,
            None,
            None,
        );

        let mut buf = vec![0u8; MARKET_ACCOUNT_SIZE];
        original.pack(&mut buf).expect("pack should succeed");

        let recovered = Market::unpack(&buf).expect("unpack should succeed");

        assert_eq!(recovered.market_type as u8, MarketType::CAPPM as u8);
        assert_eq!(recovered.is_resolved, false);
        assert_eq!(recovered.title, "Will BTC > $95,000?");
        assert_eq!(recovered.description, "Based on Coinbase BTC-USD price");
        assert_eq!(recovered.start_time_utc, 1_700_000_000);
        assert_eq!(recovered.end_time_utc, 1_800_000_000);
        assert_eq!(recovered.duration_seconds, 86400);
        assert_eq!(recovered.data_provider, "coinbase");
        assert_eq!(recovered.bump, 255);
        assert_eq!(recovered.direction, Some(Direction::Above));
        assert_eq!(recovered.target_price, Some(9_500_000));
        assert_eq!(recovered.current_price, Some(9_400_000));
        assert_eq!(recovered.end_price, None);
        assert_eq!(recovered.outcome, None);
        assert_eq!(recovered.approved_settler, APPROVED_SETTLER);
    }

    #[test]
    fn test_market_cappm_resolved_pack_unpack() {
        let original = make_market_cappm(
            true,
            Direction::Below,
            9_500_000,
            1_800_000_000,
            Some(9_200_000),
            Some(Outcome::Yes),
        );

        let mut buf = vec![0u8; MARKET_ACCOUNT_SIZE];
        original.pack(&mut buf).expect("pack should succeed");

        let recovered = Market::unpack(&buf).expect("unpack should succeed");

        assert_eq!(recovered.is_resolved, true);
        assert_eq!(recovered.direction, Some(Direction::Below));
        assert_eq!(recovered.end_price, Some(9_200_000));
        assert_eq!(recovered.outcome, Some(Outcome::Yes));
    }

    #[test]
    fn test_market_gem_pack_unpack_roundtrip() {
        let original = make_market_gem(false);

        let mut buf = vec![0u8; MARKET_ACCOUNT_SIZE];
        original.pack(&mut buf).expect("pack should succeed");

        let recovered = Market::unpack(&buf).expect("unpack should succeed");

        assert_eq!(recovered.market_type as u8, MarketType::GEM as u8);
        assert_eq!(recovered.is_resolved, false);
        assert_eq!(recovered.direction, None);
        assert_eq!(recovered.target_price, None);
        assert_eq!(recovered.current_price, None);
        assert_eq!(recovered.end_price, None);
        assert_eq!(recovered.outcome, None);
        assert_eq!(recovered.data_provider, "kalshi");
    }

    #[test]
    fn test_market_gem_resolved_pack_unpack() {
        let mut market = make_market_gem(true);
        market.outcome = Some(Outcome::No);
        market.outcome_description = "Team X lost 0-2 per ESPN".to_string();

        let mut buf = vec![0u8; MARKET_ACCOUNT_SIZE];
        market.pack(&mut buf).expect("pack should succeed");

        let recovered = Market::unpack(&buf).expect("unpack should succeed");

        assert_eq!(recovered.outcome, Some(Outcome::No));
        assert_eq!(recovered.outcome_description, "Team X lost 0-2 per ESPN");
    }

    #[test]
    fn test_settlement_log_cappm_pack_unpack_roundtrip() {
        let log = SettlementLog {
            market: Pubkey::new_unique(),
            settled_at: 1_800_000_100,
            settled_by: APPROVED_SETTLER,
            end_price: Some(9_620_000),
            outcome: Outcome::Yes,
            outcome_description: "BTC closed at $96200.00 on Coinbase".to_string(),
            signature_hash: [0xABu8; 32],
            message_hash: [0xCDu8; 32],
            bump: 253,
        };

        let mut buf = vec![0u8; SETTLEMENT_ACCOUNT_SIZE];
        log.pack(&mut buf).expect("pack should succeed");

        let recovered = SettlementLog::unpack(&buf).expect("unpack should succeed");

        assert_eq!(recovered.market, log.market);
        assert_eq!(recovered.settled_at, 1_800_000_100);
        assert_eq!(recovered.settled_by, APPROVED_SETTLER);
        assert_eq!(recovered.end_price, Some(9_620_000));
        assert_eq!(recovered.outcome as u8, Outcome::Yes as u8);
        assert_eq!(
            recovered.outcome_description,
            "BTC closed at $96200.00 on Coinbase"
        );
        assert_eq!(recovered.signature_hash, [0xABu8; 32]);
        assert_eq!(recovered.message_hash, [0xCDu8; 32]);
        assert_eq!(recovered.bump, 253);
    }

    #[test]
    fn test_settlement_log_gem_pack_unpack_roundtrip() {
        let log = SettlementLog {
            market: Pubkey::new_unique(),
            settled_at: 1_800_000_200,
            settled_by: APPROVED_SETTLER,
            end_price: None, // GEM has no price
            outcome: Outcome::No,
            outcome_description: "Team X lost 0-3 per ESPN".to_string(),
            signature_hash: [0x11u8; 32],
            message_hash: [0x22u8; 32],
            bump: 252,
        };

        let mut buf = vec![0u8; SETTLEMENT_ACCOUNT_SIZE];
        log.pack(&mut buf).expect("pack should succeed");

        let recovered = SettlementLog::unpack(&buf).expect("unpack should succeed");

        assert_eq!(recovered.end_price, None);
        assert_eq!(recovered.outcome as u8, Outcome::No as u8);
    }

    #[test]
    fn test_cappm_outcome_above_yes() {
        let end_price = 9_620_000u64;
        let target = 9_500_000u64;
        let direction = Direction::Above;
        let outcome = determine_cappm_outcome(direction, end_price, target);
        assert_eq!(outcome, Outcome::Yes, "end >= target (Above) should be YES");
    }

    #[test]
    fn test_cappm_outcome_above_exact_price_is_yes() {
        let end_price = 9_500_000u64;
        let target = 9_500_000u64;
        let outcome = determine_cappm_outcome(Direction::Above, end_price, target);
        assert_eq!(
            outcome,
            Outcome::Yes,
            "exact target price (Above) should be YES"
        );
    }

    #[test]
    fn test_cappm_outcome_above_no() {
        let end_price = 9_499_999u64;
        let target = 9_500_000u64;
        let outcome = determine_cappm_outcome(Direction::Above, end_price, target);
        assert_eq!(outcome, Outcome::No, "end < target (Above) should be NO");
    }

    #[test]
    fn test_cappm_outcome_below_yes() {
        let end_price = 9_400_000u64;
        let target = 9_500_000u64;
        let outcome = determine_cappm_outcome(Direction::Below, end_price, target);
        assert_eq!(outcome, Outcome::Yes, "end < target (Below) should be YES");
    }

    #[test]
    fn test_cappm_outcome_below_exact_price_is_no() {
        let end_price = 9_500_000u64;
        let target = 9_500_000u64;
        let outcome = determine_cappm_outcome(Direction::Below, end_price, target);
        assert_eq!(
            outcome,
            Outcome::No,
            "exact target price (Below) should be NO"
        );
    }

    #[test]
    fn test_cappm_outcome_below_no() {
        let end_price = 9_500_001u64;
        let target = 9_500_000u64;
        let outcome = determine_cappm_outcome(Direction::Below, end_price, target);
        assert_eq!(outcome, Outcome::No, "end >= target (Below) should be NO");
    }

    #[test]
    fn test_vs_event_pack_unpack_roundtrip() {
        let p1 = Pubkey::new_unique();
        let p2 = Pubkey::new_unique();
        let ev = VSEvent {
            vs_id: "VS_abc123".to_string(),
            creator: p1,
            title: "Will Barca win?".to_string(),
            stake_cents: 5000,
            mode: VSMode::Mutual,
            threshold: 2,
            status: VSStatus::Active,
            created_at: 1_800_001_000,
            join_deadline_utc: 1_800_001_600,
            resolve_deadline_utc: 1_800_002_000,
            participant_count: 2,
            participants: vec![p1, p2],
            outcome: Some(1),
            outcome_description: "YES by mutual confirm".to_string(),
            votes_yes: vec![p1, p2],
            votes_no: vec![],
            bump: 250,
        };

        let mut buf = vec![0u8; MARKET_ACCOUNT_SIZE];
        ev.pack(&mut buf).expect("pack should succeed");
        let got = VSEvent::unpack(&buf).expect("unpack should succeed");
        assert_eq!(got.vs_id, "VS_abc123");
        assert_eq!(got.mode, VSMode::Mutual);
        assert_eq!(got.status, VSStatus::Active);
        assert_eq!(got.participant_count, 2);
        assert_eq!(got.participants.len(), 2);
        assert_eq!(got.outcome, Some(1));
    }

    #[test]
    fn test_market_already_resolved_error() {
        let market = make_market_cappm(
            true,
            Direction::Above,
            9_500_000,
            1_700_000_000,
            Some(9_600_000),
            Some(Outcome::Yes),
        );

        let err = check_market_not_resolved(&market).unwrap_err();
        assert_eq!(
            err,
            ProgramError::Custom(MarketError::MarketAlreadyResolved as u32)
        );
    }

    #[test]
    fn test_market_not_expired_error() {
        let far_future = u64::MAX;
        let market = make_market_cappm(false, Direction::Above, 9_500_000, far_future, None, None);
        let now = 0u64;
        let err = check_market_expired(&market, now).unwrap_err();
        assert_eq!(
            err,
            ProgramError::Custom(MarketError::MarketNotExpired as u32)
        );
    }

    #[test]
    fn test_market_expiry_boundary() {
        let end_time = 1_800_000_000u64;
        let market = make_market_cappm(false, Direction::Above, 9_500_000, end_time, None, None);
        let now = end_time;
        assert!(
            check_market_expired(&market, now).is_ok(),
            "now==end_time should be expired"
        );

        let now_before = end_time.checked_sub(1).expect("end_time > 0");
        let err = check_market_expired(&market, now_before).unwrap_err();
        assert_eq!(
            err,
            ProgramError::Custom(MarketError::MarketNotExpired as u32)
        );
    }

    #[test]
    fn test_unauthorized_settler_error() {
        let wrong_settler = Pubkey::new_unique();
        let market = make_market_cappm(false, Direction::Above, 9_500_000, 0, None, None);

        let err = check_settler_authorized(&market, &wrong_settler).unwrap_err();
        assert_eq!(
            err,
            ProgramError::Custom(MarketError::UnauthorizedSettler as u32)
        );
    }

    #[test]
    fn test_authorized_settler_ok() {
        let market = make_market_cappm(false, Direction::Above, 9_500_000, 0, None, None);
        assert!(check_settler_authorized(&market, &APPROVED_SETTLER).is_ok());
    }

    #[test]
    fn test_invalid_data_provider_error() {
        let bad_provider = "binance"; // not in approved list
        let err = check_data_provider(bad_provider).unwrap_err();
        assert_eq!(
            err,
            ProgramError::Custom(MarketError::InvalidDataProvider as u32)
        );
    }

    #[test]
    fn test_valid_data_providers() {
        assert!(check_data_provider("coinbase").is_ok());
        assert!(check_data_provider("kalshi").is_ok());
    }

    #[test]
    fn test_invalid_target_price_zero() {
        let err = check_target_price(0).unwrap_err();
        assert_eq!(
            err,
            ProgramError::Custom(MarketError::InvalidTargetPrice as u32)
        );
    }

    #[test]
    fn test_valid_target_price() {
        assert!(check_target_price(1).is_ok());
        assert!(check_target_price(9_500_000).is_ok());
        assert!(check_target_price(u64::MAX).is_ok());
    }

    #[test]
    fn test_start_time_in_past_error() {
        let now = 1_800_000_000u64;
        let start_time_in_past = 1_700_000_000u64;
        let err = check_start_time(start_time_in_past, now).unwrap_err();
        assert_eq!(
            err,
            ProgramError::Custom(MarketError::InvalidEndTime as u32)
        );
    }

    #[test]
    fn test_start_time_equal_now_error() {
        let now = 1_800_000_000u64;
        let err = check_start_time(now, now).unwrap_err();
        assert_eq!(
            err,
            ProgramError::Custom(MarketError::InvalidEndTime as u32)
        );
    }

    #[test]
    fn test_start_time_future_ok() {
        let now = 1_800_000_000u64;
        let future = now.checked_add(1).expect("no overflow");
        assert!(check_start_time(future, now).is_ok());
    }

    #[test]
    fn test_invalid_direction_byte_error() {
        let err = vant_crypto::state::Direction::from_u8(99).unwrap_err();
        assert_eq!(
            err,
            ProgramError::Custom(MarketError::InvalidDirection as u32)
        );
    }

    #[test]
    fn test_invalid_market_type_byte_error() {
        let err = vant_crypto::state::MarketType::from_u8(99).unwrap_err();
        assert_eq!(
            err,
            ProgramError::Custom(MarketError::InvalidMarketType as u32)
        );
    }

    #[test]
    fn test_invalid_outcome_byte_error() {
        let err = vant_crypto::state::Outcome::from_u8(99).unwrap_err();
        assert_eq!(
            err,
            ProgramError::Custom(MarketError::InvalidOutcome as u32)
        );
    }

    #[test]
    fn test_error_codes_are_unique() {
        use MarketError::*;
        let codes: Vec<u32> = vec![
            InvalidAccountCount as u32,
            InvalidSigner as u32,
            InvalidWritable as u32,
            InvalidOwner as u32,
            UninitializedAccount as u32,
            InvalidMarketType as u32,
            MarketAlreadyResolved as u32,
            UnauthorizedSettler as u32,
            InvalidSettlementSignature as u32,
            MarketNotExpired as u32,
            InvalidDataProvider as u32,
            InvalidTargetPrice as u32,
            InvalidEndTime as u32,
            ArithmeticOverflow as u32,
            SerializationError as u32,
            InvalidInstructionData as u32,
            InvalidAccountIndex as u32,
            InvalidAccount as u32,
            MarketNotStarted as u32,
            InvalidDirection as u32,
            InvalidSettlerPubkey as u32,
            InvalidPDA as u32,
            MarketNotResolvable as u32,
            InvalidOutcome as u32,
        ];
        let mut seen = std::collections::HashSet::new();
        for code in &codes {
            assert!(seen.insert(code), "Duplicate error code: {}", code);
        }
    }

    #[test]
    fn test_market_error_into_program_error() {
        let err: ProgramError = MarketError::MarketAlreadyResolved.into();
        assert_eq!(err, ProgramError::Custom(6));

        let err: ProgramError = MarketError::UnauthorizedSettler.into();
        assert_eq!(err, ProgramError::Custom(7));

        let err: ProgramError = MarketError::InvalidSettlementSignature.into();
        assert_eq!(err, ProgramError::Custom(8));
    }

    #[test]
    fn test_sha256_known_value() {
        let hash = sha256(b"");
        let expected = [
            0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
            0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
            0x78, 0x52, 0xb8, 0x55,
        ];
        assert_eq!(hash, expected, "sha256('') mismatch");
    }

    #[test]
    fn test_sha256_settlement_message() {
        let msg = b"VANT_CAPPM_SETTLEMENT:BTC_95000_20260320:9620000";
        let hash = sha256(msg);
        assert_ne!(hash, [0u8; 32]);
        assert_eq!(sha256(msg), hash, "sha256 must be deterministic");
    }

    #[test]
    fn test_sha256_different_messages_produce_different_hashes() {
        let h1 = sha256(b"VANT_CAPPM_SETTLEMENT:BTC_1:9500000");
        let h2 = sha256(b"VANT_CAPPM_SETTLEMENT:BTC_1:9500001");
        assert_ne!(h1, h2, "Different messages must hash differently");
    }

    #[test]
    fn test_market_account_size_sufficient() {
        let market = Market {
            market_type: MarketType::CAPPM,
            is_resolved: true,
            creator: Pubkey::new_unique(),
            approved_settler: APPROVED_SETTLER,
            title: "A".repeat(256),
            description: "B".repeat(1024),
            start_time_utc: u64::MAX,
            end_time_utc: u64::MAX,
            duration_seconds: u64::MAX,
            data_provider: "C".repeat(64),
            created_at: u64::MAX,
            bump: 255,
            asset: "A".repeat(10),
            direction: Some(Direction::Below),
            target_price: Some(u64::MAX),
            current_price: Some(u64::MAX),
            end_price: Some(u64::MAX),
            outcome: Some(Outcome::Yes),
            outcome_description: "D".repeat(512),
        };

        let mut buf = vec![0u8; MARKET_ACCOUNT_SIZE];
        market
            .pack(&mut buf)
            .expect("Max-size market must fit in MARKET_ACCOUNT_SIZE");
    }

    #[test]
    fn test_settlement_account_size_sufficient() {
        let log = SettlementLog {
            market: Pubkey::new_unique(),
            settled_at: u64::MAX,
            settled_by: APPROVED_SETTLER,
            end_price: Some(u64::MAX),
            outcome: Outcome::No,
            outcome_description: "E".repeat(512),
            signature_hash: [0xFFu8; 32],
            message_hash: [0xFFu8; 32],
            bump: 255,
        };

        let mut buf = vec![0u8; SETTLEMENT_ACCOUNT_SIZE];
        log.pack(&mut buf)
            .expect("Max-size SettlementLog must fit in SETTLEMENT_ACCOUNT_SIZE");
    }

    fn determine_cappm_outcome(direction: Direction, end_price: u64, target_price: u64) -> Outcome {
        match direction {
            Direction::Above => {
                if end_price >= target_price {
                    Outcome::Yes
                } else {
                    Outcome::No
                }
            }
            Direction::Below => {
                if end_price < target_price {
                    Outcome::Yes
                } else {
                    Outcome::No
                }
            }
        }
    }

    fn check_market_not_resolved(market: &Market) -> Result<(), ProgramError> {
        if market.is_resolved {
            return Err(ProgramError::Custom(
                MarketError::MarketAlreadyResolved as u32,
            ));
        }
        Ok(())
    }

    fn check_market_expired(market: &Market, now: u64) -> Result<(), ProgramError> {
        if now < market.end_time_utc {
            return Err(ProgramError::Custom(MarketError::MarketNotExpired as u32));
        }
        Ok(())
    }

    fn check_settler_authorized(market: &Market, settler: &Pubkey) -> Result<(), ProgramError> {
        if settler != &market.approved_settler {
            return Err(ProgramError::Custom(
                MarketError::UnauthorizedSettler as u32,
            ));
        }
        Ok(())
    }

    fn check_data_provider(provider: &str) -> Result<(), ProgramError> {
        use vant_crypto::constants::APPROVED_DATA_PROVIDERS;
        if !APPROVED_DATA_PROVIDERS.contains(&provider) {
            return Err(ProgramError::Custom(
                MarketError::InvalidDataProvider as u32,
            ));
        }
        Ok(())
    }

    fn check_target_price(price: u64) -> Result<(), ProgramError> {
        if price == 0 {
            return Err(ProgramError::Custom(MarketError::InvalidTargetPrice as u32));
        }
        Ok(())
    }

    fn check_start_time(start_time: u64, now: u64) -> Result<(), ProgramError> {
        if start_time <= now {
            return Err(ProgramError::Custom(MarketError::InvalidEndTime as u32));
        }
        Ok(())
    }
}
