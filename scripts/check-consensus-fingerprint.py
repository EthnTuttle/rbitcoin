#!/usr/bin/env python3
"""Independent v1 consensus-fingerprint encoder; input is the typed vector JSON."""
import hashlib
import json
from pathlib import Path
import struct
import sys


def encode(f):
    def u32(n):
        return struct.pack('>I', n)

    def blob(b):
        return u32(len(b)) + b

    def optional(b):
        return b'\x00' if b is None else b'\x01' + blob(b)

    def display_hash(h):
        return bytes.fromhex(h)[::-1]

    out = b'rbitcoin-consensus-facts/1\x00'
    out += blob(f['consensus_profile_id'].encode('ascii'))
    out += blob(f['btc_network'].encode('ascii'))
    out += display_hash(f['genesis_hash'])
    out += display_hash(f['genesis_coinbase_txid'])
    out += u32(int(f['initial_target_bits'], 16))
    overlay = f['regtest_pow_bits_overlay']
    out += optional(None if overlay is None else u32(int(overlay, 16)))
    out += bytes.fromhex(f['pow_limit_exact'])
    out += bytes.fromhex(f['retarget_floor_exact'])
    out += struct.pack('>QQI', f['pow_target_spacing'], f['pow_target_timespan'],
                       f['difficulty_adjustment_interval'])
    for k in ('no_pow_retargeting', 'allow_min_difficulty_blocks', 'enforce_bip94'):
        assert type(f[k]) is bool
        out += bytes([int(f[k])])
    out += struct.pack('>Q', int(f['subsidy_initial_sats']))
    for k in ('subsidy_halving_interval', 'coinbase_maturity', 'bip16_time',
              'bip34', 'bip66', 'bip65', 'csv', 'segwit', 'taproot',
              'rule_change_activation_threshold', 'miner_confirmation_window'):
        out += u32(f[k])
    out += optional(None if f['bip34_hash'] is None else display_hash(f['bip34_hash']))
    out += optional(None if f['signet_challenge'] is None else bytes.fromhex(f['signet_challenge']))
    deployment = f['testdummy']
    out += b'\x00' if deployment is None else b'\x01' + struct.pack(
        '>BII', deployment['bit'], deployment['period'], deployment['threshold'])
    return out


if __name__ == '__main__':
    path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parents[1] / (
        'crates/rbitcoin-rpc/tests/fixtures/consensus-fingerprint-v1.json')
    vector = json.loads(path.read_text())
    encoded = encode(vector['facts'])
    digest = hashlib.sha256(encoded).hexdigest()
    assert encoded.hex() == vector['preimage_hex'], encoded.hex()
    assert digest == vector['sha256'], digest
    print('PASS consensus-fingerprint-v1:', len(encoded), 'bytes, sha256', digest)
