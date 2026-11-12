`getblockchaininfo.consensus` adds versioned `consensus_profile_id`, a
deterministic typed-parameter `profile_fingerprint`, `initial_target_bits`,
and exact decimal-string `subsidy_initial_sats`. The values come from the
effective chain parameters and consensus subsidy function. Hash recipe and
independent Python/Rust RPC vector are documented in `docs/rpc.md`.
