use bitcoin::CompactTarget;
use rbitcoin_consensus::header::apply_regtest_pow_overlay;
use rbitcoin_consensus::params::ChainParams;

fn bits(v: u32) -> CompactTarget {
    CompactTarget::from_consensus(v)
}

/// With retargeting on, the overlay initializes the first period only. On a
/// chain the profile validated, no parent past the first block carries the
/// pow-limit bits (the floor caps every retarget below it), so the boundary
/// and inheritance own the difficulty from the first boundary on. Bits equal
/// to the pow limit at a later height can only come from a datadir mined
/// under default regtest: the overlay steps them onto the profile at the
/// next non-boundary height, the same way it seeds height 1.
#[test]
fn retargeting_overlay_initializes_the_first_period_only() {
    let mut params = ChainParams::regtest();
    params.btc.no_pow_retargeting = false;
    params.btc.allow_min_difficulty_blocks = false;
    params.btc.pow_target_spacing = 10;
    params.btc.pow_target_timespan = 40;
    params
        .apply_regtest_pow_bits(0x207f_fffe)
        .expect("harder than the pow limit");
    let overlay = bits(0x207f_fffe);
    let limit = bits(0x207f_ffff);
    // No min-difficulty gap: the header time is one spacing after the parent.
    let (prev_t, t) = (1_000, 1_010);
    assert_eq!(
        apply_regtest_pow_overlay(&params, 3, limit, prev_t, t),
        overlay,
        "below the interval the overlay initializes the period"
    );
    assert_eq!(
        apply_regtest_pow_overlay(&params, 4, limit, prev_t, t),
        limit,
        "a boundary keeps the computed bits"
    );
    assert_eq!(
        apply_regtest_pow_overlay(&params, 5, limit, prev_t, t),
        overlay,
        "pow-limit bits past the interval are a default-regtest datadir: \
         they step onto the overlay instead of extending to the boundary, \
         where the retarget multiply from the pow limit would overflow"
    );
}

/// Without retargeting (default regtest) the overlay stays the fixed
/// difficulty it always was, at every height.
#[test]
fn fixed_difficulty_overlay_keeps_every_height() {
    let mut params = ChainParams::regtest();
    params
        .apply_regtest_pow_bits(0x207f_fffe)
        .expect("harder than the pow limit");
    let overlay = bits(0x207f_fffe);
    let limit = bits(0x207f_ffff);
    let (prev_t, t) = (1_000, 1_010);
    for h in [3u32, 5, 23, 2_017] {
        assert_eq!(
            apply_regtest_pow_overlay(&params, h, limit, prev_t, t),
            overlay,
            "height {h} keeps the fixed difficulty"
        );
    }
}
