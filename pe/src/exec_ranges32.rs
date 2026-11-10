//! One-entry accelerator for immutable loaded-image executable section bounds.
//! Dynamic executable allocations and instruction bytes are deliberately not cached.
use std::cell::Cell;

pub(super) struct StaticExecutableRegions<'a> {
    regions: &'a [(usize, usize, u32)],
    recent: Cell<Option<(usize, usize)>>,
}
impl<'a> StaticExecutableRegions<'a> {
    pub(super) fn new(regions: &'a [(usize, usize, u32)]) -> Self {
        Self {
            regions,
            recent: Cell::new(None),
        }
    }
    pub(super) fn contains(&self, start: usize, end: usize) -> bool {
        if start >= end {
            return false;
        }
        if self
            .recent
            .get()
            .is_some_and(|(a, b)| start >= a && end <= b)
        {
            return true;
        }
        for &(a, b, rights) in self.regions {
            if rights & 0x20000000 == 0 {
                continue;
            }
            let Some(page_end) = b
                .checked_add(super::PAGE - 1)
                .map(|v| v & !(super::PAGE - 1))
            else {
                continue;
            };
            if start >= a && end <= page_end {
                self.recent.set(Some((a, page_end)));
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const X: u32 = 0x20000000;
    fn linear(regions: &[(usize, usize, u32)], start: usize, end: usize) -> bool {
        start < end
            && regions.iter().any(|&(a, b, r)| {
                r & X != 0
                    && start >= a
                    && end <= (b + super::super::PAGE - 1) & !(super::super::PAGE - 1)
            })
    }
    #[test]
    fn cached_and_missed_ranges_match_full_section_scan() {
        // Unsorted, overlapping, disjoint, readable-only and page-rounded sections.
        let regions = [
            (0x9000, 0x91ab, X),
            (0x2000, 0x2800, 0x40000000),
            (0x1000, 0x1801, X),
            (0x1800, 0x3000, X),
            (0x5000, 0x6000, X),
        ];
        let cache = StaticExecutableRegions::new(&regions);
        let points = [
            0, 0xfff, 0x1000, 0x17ff, 0x1800, 0x1fff, 0x2000, 0x2fff, 0x3000, 0x5000, 0x5fff,
            0x6000, 0x9000, 0x91ab, 0x9fff, 0xa000,
        ];
        for _ in 0..3 {
            for &a in &points {
                for &b in points.iter().rev() {
                    assert_eq!(
                        cache.contains(a, b),
                        linear(&regions, a, b),
                        "{a:#x}..{b:#x}"
                    );
                }
            }
        }
    }
    #[test]
    fn cached_permission_never_extends_across_a_gap_or_adjacent_sections() {
        let cache = StaticExecutableRegions::new(&[
            (0x1000, 0x2000, X),
            (0x2000, 0x3000, X),
            (0x4000, 0x5000, X),
        ]);
        assert!(cache.contains(0x1fff, 0x2000));
        assert!(!cache.contains(0x1fff, 0x2001));
        assert!(cache.contains(0x2000, 0x2001));
        assert!(!cache.contains(0x2fff, 0x4001));
        assert!(!cache.contains(0x3000, 0x3001));
        assert!(!cache.contains(0x2000, 0x2000));
        assert!(!cache.contains(0x2001, 0x2000));
    }
    #[test]
    fn ownership_is_per_context_and_unrepresentable_page_ends_are_rejected() {
        let first = StaticExecutableRegions::new(&[(0x1000, 0x2000, X)]);
        assert!(first.contains(0x1000, 0x1001));
        let other = StaticExecutableRegions::new(&[(0x1000, 0x2000, 0x40000000)]);
        assert!(!other.contains(0x1000, 0x1001));
        let biased = StaticExecutableRegions::new(&[(0x8000001000, 0x8000002000, X)]);
        assert!(biased.contains(0x8000001fff, 0x8000002000));
        assert!(!biased.contains(0x1fff, 0x2000));
        let overflow = StaticExecutableRegions::new(&[(usize::MAX - 0x1000, usize::MAX, X)]);
        assert!(!overflow.contains(usize::MAX - 1, usize::MAX));
        assert!(!StaticExecutableRegions::new(&[]).contains(0x1000, 0x1001));
    }
}
