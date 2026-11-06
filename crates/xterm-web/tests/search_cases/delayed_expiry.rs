// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use xterm_core::search::CACHE_TTL_MS;

#[test]
fn overdue_cache_access_precedes_delayed_timer_expiry_in_both_directions() {
    for previous in [false, true] {
        for delayed in [CACHE_TTL_MS - 1, CACHE_TTL_MS, CACHE_TTL_MS * 3] {
            let h = Harness::new(20, 3, None);
            h.write("foo\r");
            let find = || {
                h.host.terminal.clear_selection().unwrap();
                if previous {
                    h.addon.find_previous(&term("foo"), None).unwrap()
                } else {
                    h.addon.find_next(&term("foo"), None).unwrap()
                }
            };
            assert!(find());
            assert_eq!(h.host.clock.pending(), 1);
            // ECH changes cells without moving the cursor or invalidating the
            // source cache. The selected result must still come from that cache.
            h.write("\x1b[3X");
            assert_eq!(h.host.clock.pending(), 1);
            h.host.clock.now.set(delayed);
            assert!(find(), "previous={previous} delayed={delayed}");
            assert_eq!(h.host.clock.pending(), 1);
            let first_fire = delayed.max(CACHE_TTL_MS);
            h.host.clock.now.set(first_fire);
            assert!(h.host.clock.fire_due_now().unwrap());
            assert_eq!(h.host.clock.pending(), 1);
            h.host.clock.now.set(delayed + CACHE_TTL_MS - 1);
            assert!(!h.host.clock.fire_due_now().unwrap());
            h.host.clock.now.set(delayed + CACHE_TTL_MS);
            assert!(h.host.clock.fire_due_now().unwrap());
            assert_eq!(h.host.clock.pending(), 0);
            assert!(!find(), "expiry must expose erased cells");
            assert_eq!(h.host.clock.pending(), 1);
            h.addon.dispose().unwrap();
            assert_eq!(h.host.clock.pending(), 0);
        }
    }
}
