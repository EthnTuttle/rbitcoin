use bitcoin::hashes::{sha256, Hash};
use bitcoin::Network;
use rbitcoin_consensus::{block_subsidy, genesis_block, ChainParams};

pub(super) struct EffectiveProfile {
    pub id: String,
    pub fingerprint: String,
    pub initial_target_bits: u32,
    pub subsidy_initial_sats: u64,
}

impl EffectiveProfile {
    pub(super) fn new(cp: &ChainParams) -> Self {
        let variant = if cp.network == Network::Regtest {
            if cp.no_pow_retargeting() {
                "fixed"
            } else {
                "retarget"
            }
        } else {
            "chain"
        };
        let mut profile = Self {
            id: format!(
                "rbitcoin-consensus/1/{}/{variant}",
                network_name(cp.network)
            ),
            fingerprint: String::new(),
            initial_target_bits: cp
                .regtest_pow_bits_overlay
                .filter(|_| cp.network == Network::Regtest)
                .unwrap_or_else(|| genesis_block(cp).header.bits)
                .to_consensus(),
            subsidy_initial_sats: u64::try_from(block_subsidy(0, cp))
                .expect("initial consensus subsidy is nonnegative"),
        };
        profile.fingerprint = sha256::Hash::hash(&profile.preimage(cp)).to_string();
        profile
    }

    fn preimage(&self, cp: &ChainParams) -> Vec<u8> {
        let mut bytes = b"rbitcoin-consensus-facts/1\0".to_vec();
        variable_bytes(&mut bytes, self.id.as_bytes());
        variable_bytes(&mut bytes, network_name(cp.btc.network).as_bytes());
        bytes.extend_from_slice(cp.genesis_hash.as_byte_array());
        bytes.extend_from_slice(&cp.genesis_coinbase_txid);
        bytes.extend_from_slice(&self.initial_target_bits.to_be_bytes());
        optional_bytes(
            &mut bytes,
            cp.regtest_pow_bits_overlay
                .map(|bits| bits.to_consensus().to_be_bytes())
                .as_ref()
                .map(|bits| bits.as_slice()),
        );
        bytes.extend_from_slice(&cp.pow_limit.to_be_bytes());
        bytes.extend_from_slice(&cp.btc.max_attainable_target.to_be_bytes());
        bytes.extend_from_slice(&cp.btc.pow_target_spacing.to_be_bytes());
        bytes.extend_from_slice(&cp.btc.pow_target_timespan.to_be_bytes());
        bytes.extend_from_slice(&cp.difficulty_adjustment_interval().to_be_bytes());
        bytes.extend_from_slice(&[
            u8::from(cp.no_pow_retargeting()),
            u8::from(cp.allow_min_difficulty_blocks()),
            u8::from(cp.enforce_bip94),
        ]);
        bytes.extend_from_slice(&self.subsidy_initial_sats.to_be_bytes());
        for value in [
            cp.subsidy_halving_interval(),
            cp.coinbase_maturity(),
            cp.btc.bip16_time,
            cp.btc.bip34_height,
            cp.btc.bip66_height,
            cp.btc.bip65_height,
            cp.csv_height(),
            cp.segwit_height(),
            cp.taproot_height(),
            cp.btc.rule_change_activation_threshold,
            cp.btc.miner_confirmation_window,
        ] {
            bytes.extend_from_slice(&value.to_be_bytes());
        }
        optional_bytes(
            &mut bytes,
            cp.bip34_hash.as_ref().map(|h| h.as_byte_array().as_slice()),
        );
        optional_bytes(
            &mut bytes,
            cp.signet_challenge.as_ref().map(|s| s.as_bytes()),
        );
        bytes.push(u8::from(cp.testdummy.is_some()));
        if let Some(deployment) = cp.testdummy {
            bytes.push(deployment.bit);
            bytes.extend_from_slice(&deployment.period.to_be_bytes());
            bytes.extend_from_slice(&deployment.threshold.to_be_bytes());
        }
        bytes
    }
}

fn network_name(network: Network) -> &'static str {
    match network {
        Network::Bitcoin => "main",
        Network::Testnet => "test",
        Network::Testnet4 => "testnet4",
        Network::Signet => "signet",
        Network::Regtest => "regtest",
    }
}

fn variable_bytes(out: &mut Vec<u8>, value: &[u8]) {
    out.extend_from_slice(
        &u32::try_from(value.len())
            .expect("consensus field fits u32")
            .to_be_bytes(),
    );
    out.extend_from_slice(value);
}

fn optional_bytes(out: &mut Vec<u8>, value: Option<&[u8]>) {
    out.push(u8::from(value.is_some()));
    if let Some(value) = value {
        variable_bytes(out, value);
    }
}
