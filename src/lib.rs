#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct NodeIndex(pub usize);

impl NodeIndex {
    #[inline(always)]
    pub const fn is_leaf(&self) -> bool {
        ops::is_leaf(self.0)
    }

    #[inline(always)]
    pub const fn level(&self) -> u8 {
        ops::level(self.0)
    }

    #[inline(always)]
    pub const fn left(&self) -> Option<Self> {
        let Some(left_index) = ops::left(self.0) else {
            return None;
        };

        Some(Self(left_index))
    }

    #[inline(always)]
    pub const fn right(&self) -> Option<Self> {
        let Some(right_index) = ops::right(self.0) else {
            return None;
        };

        Some(Self(right_index))
    }

    #[inline(always)]
    pub const fn children(&self) -> Option<(Self, Self)> {
        let Some((left_index, right_index)) = ops::children(self.0) else {
            return None;
        };

        Some((Self(left_index), Self(right_index)))
    }

    #[inline(always)]
    pub const fn common_ancestor(&self, other: &Self) -> Self {
        Self(ops::common_ancestor(self.0, other.0))
    }

    #[inline(always)]
    pub const fn parent(&self, subroot_idx: &Self) -> Option<Self> {
        let Some(parent) = ops::parent_unchecked(self.0, subroot_idx.0) else {
            return None;
        };

        Some(Self(parent))
    }

    #[inline(always)]
    pub const fn to_leaf_index(&self) -> Option<LeafIndex> {
        if !self.is_leaf() {
            return None;
        }

        Some(LeafIndex((self.0 / 2) as u32))
    }

    #[inline(always)]
    pub const fn from_leaf_index(leaf_index: LeafIndex) -> Self {
        Self((leaf_index.0 as usize) * 2)
    }

    #[inline(always)]
    pub const fn sibling(&self, leaf_count: usize) -> Option<Self> {
        let Some(sibling_index) = ops::sibling(self.0, leaf_count) else {
            return None;
        };
        Some(Self(sibling_index))
    }
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct LeafIndex(pub u32);

impl From<LeafIndex> for NodeIndex {
    #[inline]
    fn from(value: LeafIndex) -> Self {
        Self::from_leaf_index(value)
    }
}

pub mod ops {
    /// Returns the height of that node in the tree
    #[inline(always)]
    pub const fn level(node_index: usize) -> u8 {
        node_index.trailing_ones() as u8
    }

    /// Returns the root node
    #[inline(always)]
    pub const fn root(leaf_count: usize) -> usize {
        let nw = node_width(leaf_count);
        if nw == 0 {
            return 0;
        }
        (1usize << (nw.ilog2())).saturating_sub(1)
    }

    #[inline(always)]
    pub const fn is_leaf(node_index: usize) -> bool {
        node_index.is_multiple_of(2)
    }

    /// Number of nodes needed to represent a tree with [leaf_count] leaves.
    #[inline(always)]
    pub const fn node_width(leaf_count: usize) -> usize {
        if leaf_count == 0 {
            return 0;
        }

        // 2*(n - 1) + 1
        (leaf_count as usize)
        .wrapping_sub(1) // since leaf_count is >= 1
        .saturating_mul(2)
        .saturating_add(1)
    }

    #[inline(always)]
    const fn parent_from_idx(node_index: usize) -> usize {
        let lzb = crate::bits::last_zero_bit(node_index);
        (lzb | node_index) & !lzb.wrapping_shl(1)
    }

    /// Get the parent of a node, return [None] if the node is the root
    #[inline(always)]
    pub const fn parent_unchecked(node_index: usize, root_index: usize) -> Option<usize> {
        if node_index == root_index {
            return None;
        }
        Some(parent_from_idx(node_index))
    }

    /// Get the parent of a node, return [None] if the node is the root
    #[inline(always)]
    pub const fn parent(node_index: usize, leaf_count: usize) -> Option<usize> {
        if node_index > ((leaf_count as usize).saturating_sub(1) * 2) {
            return None;
        }
        parent_unchecked(node_index, root(leaf_count))
    }

    /// Given a node, return the left/right child of his parent, return [None] when root
    #[inline(always)]
    pub const fn sibling_unchecked(node_index: usize, leaf_count: usize) -> Option<usize> {
        let Some(parent) = parent_unchecked(node_index, root(leaf_count)) else {
            return None;
        };
        let parent = parent as isize;
        let d = parent.overflowing_sub(node_index as isize).0;
        let sibling = parent.overflowing_add(d).0;
        Some(sibling as usize)
    }

    /// Given a node, return the left/right child of his parent, return [None] when root
    #[inline(always)]
    pub const fn sibling(node_index: usize, leaf_count: usize) -> Option<usize> {
        if node_index > ((leaf_count as usize).saturating_sub(1) * 2) {
            return None;
        }
        sibling_unchecked(node_index, leaf_count)
    }

    #[inline(always)]
    pub const fn left(node_index: usize) -> Option<usize> {
        if is_leaf(node_index) {
            return None;
        }
        let lzb = crate::bits::last_zero_bit(node_index);
        let left = node_index & !lzb.wrapping_shr(1);
        Some(left)
    }

    #[inline(always)]
    pub const fn right(node_index: usize) -> Option<usize> {
        if is_leaf(node_index) {
            return None;
        }
        let lzb = crate::bits::last_zero_bit(node_index);
        let right = (node_index | lzb) & !lzb.wrapping_shr(1);
        Some(right)
    }

    /// Returns (left, right)
    #[inline(always)]
    pub const fn children(node_index: usize) -> Option<(usize, usize)> {
        if is_leaf(node_index) {
            return None;
        }

        let lzb = crate::bits::last_zero_bit(node_index);
        let mask = !lzb.wrapping_shr(1);
        let left = node_index & mask;
        let right = (node_index | lzb) & mask;

        Some((left, right))
    }

    pub fn direct_path_to_subroot(mut node_index: usize, subroot_idx: usize) -> Option<impl Iterator<Item = usize>> {
        if subroot_idx == node_index {
            return None;
        }

        Some(std::iter::from_fn(move || {
            node_index = parent_unchecked(node_index, subroot_idx)?;
            Some(node_index)
        }))
    }

    #[inline(always)]
    pub fn direct_path_unchecked(node_index: usize, leaf_count: usize) -> Option<impl Iterator<Item = usize>> {
        direct_path_to_subroot(node_index, root(leaf_count))
    }

    pub fn direct_path(node_index: usize, leaf_count: usize) -> Option<impl Iterator<Item = usize>> {
        if node_index > ((leaf_count as usize).saturating_sub(1) * 2) {
            return None;
        }
        direct_path_unchecked(node_index, leaf_count)
    }

    #[inline(always)]
    pub const fn child_with_direction(node_index: usize, direction: bool, level: u8) -> usize {
        let f = 2 ^ (1usize.wrapping_shl(direction as u32) | 1);
        let lvl = level.wrapping_sub(1);
        let f = f.wrapping_shl(lvl as u32);
        node_index ^ f
    }

    #[inline(always)]
    pub fn copath_unchecked(node_index: usize, leaf_count: usize) -> Option<impl Iterator<Item = usize>> {
        direct_path_unchecked(node_index, leaf_count).map(|dp_iter| core::iter::once(node_index).chain(dp_iter).filter_map(move |idx| sibling(idx, leaf_count)))
    }

    #[inline(always)]
    pub fn copath(node_index: usize, leaf_count: usize) -> Option<impl Iterator<Item = usize>> {
        if node_index > ((leaf_count as usize).saturating_sub(1) * 2) {
            return None;
        }
        copath_unchecked(node_index, leaf_count)
    }

    #[inline(always)]
    pub const fn common_ancestor(node_index: usize, other: usize) -> usize {
        if node_index == other {
            return node_index;
        }
        let d = (node_index ^ other).isolate_highest_one();
        (node_index & !d) | (d.wrapping_sub(1))
    }
}

mod bits {
    #[inline(always)]
    pub const fn last_set_bit(n: usize) -> usize {
        n.wrapping_sub(n.wrapping_sub(1) & n)
    }

    #[inline(always)]
    pub const fn last_zero_bit(n: usize) -> usize {
        last_set_bit(n + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::{bounds::*, naive::*};
    use itertools::*;

    mod level {
        use super::*;

        #[test]
        fn should_succeed() {
            for i in 0usize..100_000 {
                assert_eq!(crate::ops::level(i) as u32, level_naive(i), "failed for node index {}", i);
            }
        }

        #[test]
        fn should_succeed_at_boundaries() {
            assert_eq!(crate::ops::level(NODE_INDEX_MAX as usize) as u32, level_naive(NODE_INDEX_MAX as usize));
            assert_eq!(crate::ops::level(0) as u32, level_naive(0));
        }
    }

    mod root {
        use super::*;

        #[test]
        fn should_succeed() {
            for lc in leaf_count_range() {
                assert_eq!(crate::ops::root(lc), root_naive(lc));
                assert_eq!(crate::ops::root(lc), lc as usize - 1);
            }
        }

        #[test]
        fn should_succeed_at_boundaries() {
            assert_eq!(crate::ops::root(0), root_naive(0));
            assert_eq!(crate::ops::root(0), 0);
            assert_eq!(crate::ops::root(LEAF_COUNT_MAX), root_naive(LEAF_COUNT_MAX));
            assert_eq!(crate::ops::root(LEAF_COUNT_MAX), ROOT_MAX);
        }
    }

    mod node_width {
        use super::*;

        #[test]
        fn should_succeed() {
            assert_eq!(crate::ops::node_width(1), 1);
            assert_eq!(crate::ops::node_width(2), 3);
        }

        #[test]
        fn should_succeed_at_boundaries() {
            assert_eq!(crate::ops::node_width(0), 0);
            assert_eq!(crate::ops::node_width(LEAF_COUNT_MAX), NODE_WIDTH_MAX);
            assert_eq!(crate::ops::node_width(usize::MAX), NODE_WIDTH_MAX);
        }
    }

    mod parent {
        use super::*;

        #[test]
        fn should_fail_when_root() {
            for (r, lc) in root_range() {
                assert!(crate::ops::parent_unchecked(r, crate::ops::root(lc)).is_none());
                assert!(parent_naive(r, lc).is_none());
            }
        }

        #[test]
        fn should_succeed_for_leaves() {
            let lc = LEAF_COUNT_MAX;
            for left_leaf in (0..=u16::MAX as usize).step_by(4) {
                assert_eq!(crate::ops::parent_unchecked(left_leaf, lc), parent_naive(left_leaf, lc));
                assert_eq!(crate::ops::parent_unchecked(left_leaf, lc), Some(left_leaf + 1));
            }
            for right_leaf in (2..=u16::MAX as usize).step_by(4) {
                assert_eq!(crate::ops::parent_unchecked(right_leaf, lc), parent_naive(right_leaf, lc));
                assert_eq!(crate::ops::parent_unchecked(right_leaf, lc), Some(right_leaf - 1));
            }
        }

        #[test]
        fn should_succeed_at_boundaries() {
            assert!(crate::ops::parent_unchecked(NODE_INDEX_MAX, LEAF_COUNT_MAX).is_some());
            assert_eq!(crate::ops::parent_unchecked(NODE_INDEX_MAX, LEAF_COUNT_MAX), parent_naive(NODE_INDEX_MAX, LEAF_COUNT_MAX));
        }
    }

    mod direct_path {
        use std::collections::VecDeque;

        use super::*;

        #[test]
        fn should_succeed() {
            for (lc, i) in leaf_count_range_with_node_index().take(100_000) {
                assert_eq!(
                    crate::ops::direct_path_unchecked(i, lc).map(|iter| iter.collect()),
                    direct_path_naive(i, lc).map(|v| v.into_iter().collect::<Vec<_>>())
                );
            }
        }

        #[test]
        fn should_succeed_for_remarkable_values() {
            let lc = 8usize;
            let values = [
                (0usize, vec![1usize, 3, 7]),
                (1, vec![3, 7]),
                (2, vec![1, 3, 7]),
                (3, vec![7]),
                (4, vec![5, 3, 7]),
                (5, vec![3, 7]),
                (6, vec![5, 3, 7]),
                (8, vec![9, 11, 7]),
                (9, vec![11, 7]),
                (10, vec![9, 11, 7]),
                (11, vec![7]),
                (12, vec![13, 11, 7]),
                (13, vec![11, 7]),
                (14, vec![13, 11, 7]),
            ]
            .map(|(i, e)| (i, VecDeque::from_iter(e)));
            for (i, expected) in values {
                assert_eq!(
                    crate::ops::direct_path_unchecked(i, lc).map(|iter| iter.collect()),
                    direct_path_naive(i, lc).map(|v| v.into_iter().collect::<Vec<_>>())
                );
                assert_eq!(
                    crate::ops::direct_path_unchecked(i, lc).map(|iter| iter.collect::<Vec<_>>()),
                    Some(expected.into_iter().collect())
                );
            }
        }

        #[test]
        fn should_succeed_at_boundaries() {
            let values = [
                (0, 1),
                (0, 2),
                (1, 2),
                (NODE_INDEX_MAX - 2, LEAF_COUNT_MAX),
                (NODE_INDEX_MAX - 1, LEAF_COUNT_MAX),
                (NODE_INDEX_MAX, LEAF_COUNT_MAX),
            ];
            for (i, lc) in values {
                assert_eq!(
                    crate::ops::direct_path_unchecked(i, lc).map(|iter| iter.collect()),
                    direct_path_naive(i, lc).map(|v| v.into_iter().collect::<Vec<_>>())
                );
            }
        }

        #[test]
        fn should_fail_for_roots() {
            for (r, lc) in root_range() {
                assert!(crate::ops::direct_path_unchecked(r, lc).is_none());
                assert_eq!(
                    crate::ops::direct_path_unchecked(r, lc).map(|iter| iter.collect()),
                    direct_path_naive(r, lc).map(|v| v.into_iter().collect::<Vec<_>>())
                );
            }
        }
    }

    mod copath {
        use std::collections::VecDeque;

        use super::*;

        #[test]
        fn should_succeed() {
            for (lc, i) in leaf_count_range_with_node_index().take(10) {
                assert_eq!(crate::ops::copath_unchecked(i, lc).map(|iter| iter.collect()), copath_naive(i, lc));
            }
        }

        #[test]
        fn should_succeed_for_remarkable_values() {
            let lc = 8usize;
            let values = [
                (0usize, vec![2usize, 5, 11]),
                (1, vec![5, 11]),
                (2, vec![0, 5, 11]),
                (3, vec![11]),
                (4, vec![6, 1, 11]),
                (5, vec![1, 11]),
                (6, vec![4, 1, 11]),
                (8, vec![10, 13, 3]),
                (9, vec![13, 3]),
                (10, vec![8, 13, 3]),
                (11, vec![3]),
                (12, vec![14, 9, 3]),
                (13, vec![9, 3]),
                (14, vec![12, 9, 3]),
            ]
            .map(|(i, e)| (i, VecDeque::from_iter(e)));
            for (i, expected) in values {
                assert_eq!(crate::ops::copath_unchecked(i, lc).map(|iter| iter.collect()), copath_naive(i, lc));
                assert_eq!(crate::ops::copath_unchecked(i, lc).map(|iter| iter.collect()), Some(expected));
            }
        }

        #[test]
        fn should_succeed_at_boundaries() {
            let values = [
                (0, 1),
                (0, 2),
                (1, 2),
                (NODE_INDEX_MAX - 2, LEAF_COUNT_MAX),
                (NODE_INDEX_MAX - 1, LEAF_COUNT_MAX),
                (NODE_INDEX_MAX, LEAF_COUNT_MAX),
            ];
            for (i, lc) in values {
                assert_eq!(crate::ops::copath_unchecked(i, lc).map(|iter| iter.collect()), copath_naive(i, lc));
            }
        }

        #[test]
        fn should_fail_for_roots() {
            for (r, lc) in root_range() {
                assert!(crate::ops::copath_unchecked(r, lc).is_none());
                assert_eq!(crate::ops::copath_unchecked(r, lc).map(|iter| iter.collect()), copath_naive(r, lc));
            }
        }
    }

    mod common_ancestor {
        use super::*;

        #[test]
        fn should_succeed() {
            let e = 10;
            for a in level_range(0).take(1 << e) {
                for b in level_range(0).take(1 << e) {
                    assert_eq!(crate::ops::common_ancestor(a, b), common_ancestor_naive(a, b));
                }
            }
        }

        #[test]
        fn should_succeed_at_boundaries() {
            let values = [(0usize, 2), (0, NODE_INDEX_MAX as usize)];
            for (a, b) in values {
                assert_eq!(crate::ops::common_ancestor(a, b), common_ancestor_naive(a, b));
            }
        }

        fn level_range(level: u32) -> impl Iterator<Item = usize> {
            let lower = (1 << level) - 1;
            let step = 1 << (level + 1);
            (lower..=NODE_INDEX_MAX).step_by(step).dedup()
        }
    }

    mod left_right {
        use super::*;

        #[test]
        fn should_succeed() {
            for node_index in (0..u32::MAX as usize).step_by(100) {
                assert_eq!(crate::ops::left(node_index), left_naive(node_index));
                assert_eq!(crate::ops::right(node_index), right_naive(node_index));
            }
        }

        #[test]
        fn should_succeed_at_boundaries() {
            let values = [0, 1, NODE_INDEX_MAX as usize - 1, NODE_INDEX_MAX as usize];
            for node_index in values {
                assert_eq!(crate::ops::left(node_index), left_naive(node_index));
                assert_eq!(crate::ops::right(node_index), right_naive(node_index));
            }
        }
    }
}

pub mod bounds {
    pub const NODE_INDEX_MAX: usize = usize::MAX - 1;
    pub const LEAF_COUNT_MAX: usize = (NODE_INDEX_MAX / 2) + 1;
    pub const NODE_WIDTH_MAX: usize = (LEAF_COUNT_MAX - 1) * 2 + 1;
    pub const ROOT_MAX: usize = LEAF_COUNT_MAX - 1;
    pub const LEVEL_MAX: u8 = usize::BITS as u8 - 1;

    pub const LEAF_COUNT_BITS: u32 = usize::BITS - 1;
    pub const ROOT_BITS: u32 = usize::BITS - 1;

    pub fn leaf_count_range() -> impl Iterator<Item = usize> {
        (0..=LEAF_COUNT_BITS).map(|sh| 1 << sh)
    }

    // returns an iterator of (root, leaf_count)
    pub fn root_range() -> impl Iterator<Item = (usize, usize)> {
        (0..=ROOT_BITS as usize).map(|e| (1usize << e) - 1).map(|root| (root, root + 1))
    }

    pub fn leaf_count_range_with_node_index() -> impl Iterator<Item = (usize, usize)> {
        leaf_count_range().flat_map(|lc| (0..=lc.saturating_sub(1) * 2).map(move |i| (lc, i)))
    }
}

#[cfg(any(test, feature = "bench"))]
pub mod naive {
    use super::*;
    use std::collections::VecDeque;

    #[inline(always)]
    pub fn level_naive(node_index: usize) -> u32 {
        if node_index & 0x01 == 0 {
            return 0;
        }

        let mut k = 0;
        while node_index.checked_shr(k).is_some() && (node_index >> k) & 0x01 == 1 {
            k += 1;
        }
        k
    }

    #[inline(always)]
    pub fn root_naive(leaf_count: usize) -> usize {
        if leaf_count == 0 {
            return 0;
        }
        let width = ops::node_width(leaf_count);
        let pow2 = 1usize << width.ilog2();
        pow2.wrapping_sub(1)
    }

    #[inline(always)]
    pub fn parent_naive(node_index: usize, leaf_count: usize) -> Option<usize> {
        if node_index == root_naive(leaf_count) {
            return None;
        }

        let k = level_naive(node_index);
        let b = (node_index >> (k + 1)) & 0x01;
        Some((node_index | (1 << k)) ^ (b << (k + 1)))
    }

    #[inline(always)]
    pub fn sibling_naive(node_index: usize, leaf_count: usize) -> Option<usize> {
        let parent = parent_naive(node_index, leaf_count)?;
        if node_index < parent { right_naive(parent) } else { left_naive(parent) }
    }

    #[inline(always)]
    pub fn left_naive(node_index: usize) -> Option<usize> {
        let k = level_naive(node_index);
        if k == 0 {
            return None;
        }
        let node_index = node_index ^ (0x01 << k.wrapping_sub(1));
        Some(node_index)
    }

    #[inline(always)]
    pub fn right_naive(node_index: usize) -> Option<usize> {
        let k = level_naive(node_index);
        if k == 0 {
            return None;
        }
        let node_index = node_index ^ (0x03 << k.wrapping_sub(1));
        Some(node_index)
    }

    #[inline(always)]
    pub fn direct_path_naive(mut node_index: usize, leaf_count: usize) -> Option<VecDeque<usize>> {
        let root = root_naive(leaf_count);
        if node_index == root {
            return None;
        }

        let mut ret = VecDeque::new();
        while node_index != root {
            match parent_naive(node_index, leaf_count) {
                Some(parent_idx) => node_index = parent_idx,
                None => return None,
            }
            ret.push_back(node_index);
        }

        Some(ret)
    }

    pub fn copath_naive(node_index: usize, leaf_count: usize) -> Option<VecDeque<usize>> {
        if node_index == crate::ops::root(leaf_count) {
            return None;
        }

        let mut path = direct_path_naive(node_index, leaf_count)?;
        path.insert(0, node_index);
        let _ = path.pop_back();

        path.into_iter().map(|path_idx| ops::sibling_unchecked(path_idx, leaf_count)).collect()
    }

    pub fn common_ancestor_naive(mut node_index: usize, mut other: usize) -> usize {
        let self_lvl = level_naive(node_index).saturating_add(1);
        let other_lvl = level_naive(other).saturating_add(1);
        if self_lvl <= other_lvl && (node_index >> other_lvl) == (other >> other_lvl) {
            return other;
        } else if other_lvl <= self_lvl && (node_index >> self_lvl) == (other >> self_lvl) {
            return node_index;
        }

        let mut k = 0u32;
        while node_index != other {
            node_index >>= 1;
            other >>= 1;
            k = k.saturating_add(1);
        }

        let s = 1usize << k.saturating_sub(1);
        (node_index.overflowing_shl(k).0).saturating_add(s).saturating_sub(1)
    }
}
