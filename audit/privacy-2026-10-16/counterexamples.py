#!/usr/bin/env python3
"""Reproduce counterexamples to prose sketches; no production-code exploit.

Run from any directory. Output includes the inspected soft3 base revision and
the exact script SHA-256. All quantities are illustrative algebra, not timing
or security measurements. Source provenance is in sources.json.
"""
import hashlib
import json
from pathlib import Path

P = 2**64 - 2**32 + 1
BASE = "8525b17"  # expanded in sources.json


def centered(value):
    value %= P
    return value if value <= P // 2 else value - P


def main():
    # The two components of the displayed BBG commitment cancel a*rho.
    # Use zero noise to exhibit its algebra, not to model an RLWE distribution.
    a, rho, value = 19, 23, 7
    c0, c1 = (a * rho + value) % P, (-a * rho) % P
    assert (c0 + c1) % P == value

    # A constant polynomial 1 is a nonzero imbalance, with norm 1 < q/2.
    # It passes every usual coefficient norm used by the prose's proposed test.
    imbalance = 1
    accepts_nonzero = abs(centered(imbalance)) * 2 < P
    assert accepts_nonzero

    # This canonical u64 is nonzero in integer money and zero in Goldilocks.
    integer_output = P
    assert 0 <= integer_output < 2**64 and integer_output != 0
    assert integer_output % P == 0

    # Exact releases reveal a singleton delta, regardless of encryption.
    previous, delta = 100, 7
    current = previous + delta
    assert current - previous == delta
    contributions = [11, 13, 17]
    assert sum(contributions) - sum(contributions[:2]) == contributions[2]

    # A live-only set after reset accepts an already consumed identifier.
    # A correctly authenticated lifetime set would reject it.
    spent = {"fixed-note-nullifier"}
    live_after_reset = set()
    assert "fixed-note-nullifier" in spent
    assert "fixed-note-nullifier" not in live_after_reset

    # Goldilocks reduction also aliases distinct identifiers.
    assert 0 != P and 0 % P == P % P

    result = {
        "scope": "counterexamples to simplified prose; not production exploits",
        "soft3_base_prefix": BASE,
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "goldilocks_prime": P,
        "bbg_displayed_components_cancel_to_value": (c0 + c1) % P,
        "bbg_norm_accepts_nonzero_imbalance": accepts_nonzero,
        "field_only_conservation_accepts_integer_output": integer_output,
        "singleton_release_recovers_delta": current - previous,
        "colluders_recover_remaining_contribution": sum(contributions) - sum(contributions[:2]),
        "live_only_reset_forgets_spend": True,
        "single_field_identifier_alias": [0, P],
    }
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
