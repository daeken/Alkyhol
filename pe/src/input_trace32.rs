//! Bounded optional input-state diagnostics; never changes API results.
#[derive(Default)]
pub(super) struct StateTrace {
    count: u32,
    last: Option<(u32, Vec<u8>)>,
}
impl StateTrace {
    pub(super) fn changed(&mut self, status: u32, bytes: &[u8]) -> bool {
        if self.count >= 128
            || self
                .last
                .as_ref()
                .is_some_and(|(s, b)| *s == status && b == bytes)
        {
            return false;
        }
        self.count += 1;
        self.last = Some((status, bytes.to_vec()));
        true
    }
}

#[cfg(test)]
mod trace_tests {
    use super::*;
    #[test]
    fn input_state_trace_is_bounded_and_deduplicates_status_and_payload() {
        let mut t = StateTrace::default();
        assert!(t.changed(0x8007000c, &[]));
        assert!(!t.changed(0x8007000c, &[]));
        assert!(t.changed(0, &[0; 20]));
        assert!(!t.changed(0, &[0; 20]));
        let mut down = [0; 20];
        down[12] = 128;
        assert!(t.changed(0, &down));
        assert!(t.changed(0, &[0; 20]));
        for i in 4..128 {
            assert!(t.changed(i, &[]));
        }
        assert_eq!(t.count, 128);
        assert!(!t.changed(0, &down));
    }
}
