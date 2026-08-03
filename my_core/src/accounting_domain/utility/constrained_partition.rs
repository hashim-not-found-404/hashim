use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Add;
use std::ops::Sub;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    LHS,
    RHS,
    Unknown,
}

fn build_dp<N>(weights: &[N]) -> Vec<HashSet<N>>
where
    N: Copy + Add<Output = N> + Sub<Output = N> + Eq + Hash + Default,
{
    let mut dp = Vec::with_capacity(weights.len() + 1);
    let mut base = HashSet::new();
    base.insert(N::default());
    dp.push(base);

    for &w in weights {
        let prev = dp.last().unwrap();
        let mut next = prev.clone();
        for &sum in prev.iter() {
            next.insert(sum + w);
        }
        dp.push(next);
    }
    dp
}

fn reconstruct_indices<N>(weights: &[N], target: N, dp: &[HashSet<N>]) -> Vec<usize>
where
    N: Copy + Add<Output = N> + Sub<Output = N> + Eq + Hash,
{
    let n = weights.len();
    let mut indices = Vec::new();
    let mut remaining = target;

    for i in (1..=n).rev() {
        let prev_set = &dp[i - 1];
        if prev_set.contains(&remaining) {
            continue;
        } else {
            let prev_remaining = remaining - weights[i - 1];
            if prev_set.contains(&prev_remaining) {
                indices.push(i - 1);
                remaining = prev_remaining;
            }
        }
    }
    indices.reverse();
    indices
}

/// Assigns every `Unknown` item to either `LHS` or `RHS` to make the sum
/// difference as small as possible. Fixed items are left untouched.
pub fn assign_partition<T, N, Fw, Fg, Fs>(items: &mut [T], weight: Fw, get_side: Fg, set_side: Fs)
where
    T: Clone, // only needed for the DP reconstruction (we clone unknown items)
    N: Copy + Add<Output = N> + Sub<Output = N> + Eq + Hash + Ord + Default,
    Fw: Fn(&T) -> N,
    Fg: Fn(&T) -> Side,
    Fs: Fn(&mut T, Side),
{
    // 1. Separate fixed sums and collect unknown items (with their weights and indices).
    let mut fixed_lhs_sum = N::default();
    let mut fixed_rhs_sum = N::default();
    let mut unknown_indices = Vec::new();
    let mut unknown_weights = Vec::new();

    for (idx, item) in items.iter().enumerate() {
        match get_side(item) {
            Side::LHS => fixed_lhs_sum = fixed_lhs_sum + weight(item),
            Side::RHS => fixed_rhs_sum = fixed_rhs_sum + weight(item),
            Side::Unknown => {
                unknown_indices.push(idx);
                unknown_weights.push(weight(item));
            }
        }
    }

    // 2. If there are no unknowns, nothing to do.
    if unknown_indices.is_empty() {
        return;
    }

    // 3. Compute target for subset sum.
    let total_unknown = unknown_weights.iter().fold(N::default(), |acc, &w| acc + w);
    let target2 = fixed_rhs_sum + total_unknown - fixed_lhs_sum;

    // 4. Build DP table and find best subset sum.
    let dp = build_dp(&unknown_weights);
    let last_set = dp.last().unwrap();

    let mut best_x = N::default();
    let mut best_diff = N::default();
    let mut first = true;

    for &x in last_set.iter() {
        let twice = x + x;
        let diff = if twice >= target2 {
            twice - target2
        } else {
            target2 - twice
        };
        if first || diff < best_diff {
            best_diff = diff;
            best_x = x;
            first = false;
        }
    }

    // 5. Reconstruct which unknown indices should go to LHS.
    let chosen_indices = reconstruct_indices(&unknown_weights, best_x, &dp);
    let selected_set: HashSet<usize> = chosen_indices.into_iter().collect();

    // 6. Assign sides to all unknown items.
    for (pos, &idx) in unknown_indices.iter().enumerate() {
        let side = if selected_set.contains(&pos) {
            Side::LHS
        } else {
            Side::RHS
        };
        set_side(&mut items[idx], side);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper struct for testing
    #[derive(Clone, Debug, PartialEq)]
    struct Item {
        name:  char,
        value: i64,
        side:  Side,
    }

    impl Item {
        fn new(name: char, value: i64, side: Side) -> Self {
            Item {
                name,
                value,
                side,
            }
        }
    }

    // --- Test helpers ---
    fn weight(item: &Item) -> i64 {
        item.value
    }

    fn get_side(item: &Item) -> Side {
        item.side
    }

    fn set_side(item: &mut Item, side: Side) {
        item.side = side;
    }

    fn sum_lhs(items: &[Item]) -> i64 {
        items.iter().filter(|it| it.side == Side::LHS).map(|it| it.value).sum()
    }

    fn sum_rhs(items: &[Item]) -> i64 {
        items.iter().filter(|it| it.side == Side::RHS).map(|it| it.value).sum()
    }

    // --- Tests ---

    #[test]
    fn no_unknowns_already_balanced() {
        let mut items = vec![Item::new('A', 5, Side::LHS), Item::new('B', 5, Side::RHS)];
        assign_partition(&mut items, weight, get_side, set_side);
        assert_eq!(sum_lhs(&items), 5);
        assert_eq!(sum_rhs(&items), 5);
        // Sides unchanged
        assert_eq!(items[0].side, Side::LHS);
        assert_eq!(items[1].side, Side::RHS);
    }

    #[test]
    fn no_unknowns_unbalanced() {
        let mut items = vec![Item::new('A', 10, Side::LHS), Item::new('B', 6, Side::RHS)];
        assign_partition(&mut items, weight, get_side, set_side);
        assert_eq!(sum_lhs(&items), 10);
        assert_eq!(sum_rhs(&items), 6);
        // Sides unchanged
        assert_eq!(items[0].side, Side::LHS);
        assert_eq!(items[1].side, Side::RHS);
    }

    #[test]
    fn all_unknown_balanced() {
        let mut items = vec![
            Item::new('A', 1, Side::Unknown),
            Item::new('B', 2, Side::Unknown),
            Item::new('C', 3, Side::Unknown),
        ];
        assign_partition(&mut items, weight, get_side, set_side);
        let lhs = sum_lhs(&items);
        let rhs = sum_rhs(&items);
        assert_eq!(lhs, rhs); // 3 vs 1+2
        // Check that no item is still Unknown
        assert!(items.iter().all(|it| it.side != Side::Unknown));
    }

    #[test]
    fn all_unknown_unbalanced() {
        let mut items = vec![
            Item::new('A', 1, Side::Unknown),
            Item::new('B', 2, Side::Unknown),
            Item::new('C', 7, Side::Unknown),
        ];
        assign_partition(&mut items, weight, get_side, set_side);
        let lhs = sum_lhs(&items);
        let rhs = sum_rhs(&items);
        let diff = (lhs - rhs).abs();
        // The best possible split is {1,2} vs {7} → diff 4
        assert_eq!(diff, 4);
        assert!(items.iter().all(|it| it.side != Side::Unknown));
    }

    #[test]
    fn mixed_fixed_unknown_balanced() {
        let mut items = vec![
            Item::new('A', 5, Side::LHS),
            Item::new('B', 3, Side::RHS),
            Item::new('C', 2, Side::Unknown),
            Item::new('D', 4, Side::Unknown),
        ];
        assign_partition(&mut items, weight, get_side, set_side);
        assert_eq!(sum_lhs(&items), 7);
        assert_eq!(sum_rhs(&items), 7);
        // Fixed items unchanged
        assert_eq!(items[0].side, Side::LHS);
        assert_eq!(items[1].side, Side::RHS);
        // Unknowns assigned
        assert!(items[2].side != Side::Unknown);
        assert!(items[3].side != Side::Unknown);
    }

    #[test]
    fn mixed_fixed_unknown_unbalanced() {
        // LHS fixed = 10, RHS fixed = 1, unknowns = 2,3 → total = 16, half = 8
        // We need LHS sum = 8 → need x = -2 (impossible) → best is x=0 → LHS=10, RHS=6 diff=4
        let mut items = vec![
            Item::new('A', 10, Side::LHS),
            Item::new('B', 1, Side::RHS),
            Item::new('C', 2, Side::Unknown),
            Item::new('D', 3, Side::Unknown),
        ];
        assign_partition(&mut items, weight, get_side, set_side);
        let lhs = sum_lhs(&items);
        let rhs = sum_rhs(&items);
        assert_eq!((lhs - rhs).abs(), 4); // 10 vs 6
        // Fixed unchanged
        assert_eq!(items[0].side, Side::LHS);
        assert_eq!(items[1].side, Side::RHS);
        // Unknowns assigned (likely C and D go to RHS)
        assert!(items[2].side != Side::Unknown);
        assert!(items[3].side != Side::Unknown);
    }

    #[test]
    fn fixed_sums_already_equal_with_unknowns() {
        // LHS=5, RHS=5, unknowns=2,3 → total=15, half=7.5 → need x=2.5 → best x=2 or 3 → diff=1
        let mut items = vec![
            Item::new('A', 5, Side::LHS),
            Item::new('B', 5, Side::RHS),
            Item::new('C', 2, Side::Unknown),
            Item::new('D', 3, Side::Unknown),
        ];
        assign_partition(&mut items, weight, get_side, set_side);
        let lhs = sum_lhs(&items);
        let rhs = sum_rhs(&items);
        let diff = (lhs - rhs).abs();
        assert_eq!(diff, 1);
        // Fixed unchanged
        assert_eq!(items[0].side, Side::LHS);
        assert_eq!(items[1].side, Side::RHS);
    }

    #[test]
    fn duplicates() {
        let mut items = vec![
            Item::new('A', 1, Side::Unknown),
            Item::new('B', 1, Side::Unknown),
            Item::new('C', 1, Side::Unknown),
            Item::new('D', 1, Side::Unknown),
        ];
        assign_partition(&mut items, weight, get_side, set_side);
        let lhs = sum_lhs(&items);
        let rhs = sum_rhs(&items);
        assert_eq!(lhs, rhs); // total=4, split 2 and 2
        assert!(items.iter().all(|it| it.side != Side::Unknown));
    }

    #[test]
    fn negative_values() {
        // LHS=5, RHS=3, unknowns=-2, 4 → total=10, half=5 → need x=0 (since LHS fixed=5)
        // We can get x=0? possible sums: -2, 4, 2 → no 0 → best x=2 (diff=2) or x=-2 (diff=4) → choose x=2 diff=2
        let mut items = vec![
            Item::new('A', 5, Side::LHS),
            Item::new('B', 3, Side::RHS),
            Item::new('C', -2, Side::Unknown),
            Item::new('D', 4, Side::Unknown),
        ];
        assign_partition(&mut items, weight, get_side, set_side);
        let lhs = sum_lhs(&items);
        let rhs = sum_rhs(&items);
        let diff = (lhs - rhs).abs();
        // The optimal diff is |5 + (-2) - (3 + 4)|? Actually if we put -2 on LHS, LHS=3, RHS=7 diff=4.
        // If we put 4 on LHS, LHS=9, RHS=1 diff=8.
        // If we put both on LHS: LHS=7, RHS=3 diff=4.
        // If neither: LHS=5, RHS=7 diff=2.
        // So best diff=2.
        assert_eq!(diff, 0);
        // Fixed unchanged
        assert_eq!(items[0].side, Side::LHS);
        assert_eq!(items[1].side, Side::RHS);
        // Unknowns assigned
        assert!(items[2].side != Side::Unknown);
        assert!(items[3].side != Side::Unknown);
    }

    #[test]
    fn large_numbers() {
        let mut items = vec![
            Item::new('A', 1_000_000, Side::LHS),
            Item::new('B', 999_999, Side::RHS),
            Item::new('C', 1, Side::Unknown),
        ];
        assign_partition(&mut items, weight, get_side, set_side);
        let lhs = sum_lhs(&items);
        let rhs = sum_rhs(&items);
        assert_eq!(lhs, rhs); // 1,000,001 vs 1,000,000? Wait, total=2,000,000? Actually 1,000,000 + 999,999 + 1 = 2,000,000, half=1,000,000. LHS fixed=1,000,000, so x=0 needed, so item C goes to RHS, LHS=1,000,000, RHS=1,000,000.
        assert_eq!(lhs, 1_000_000);
        assert_eq!(rhs, 1_000_000);
    }

    #[test]
    fn empty_slice() {
        let mut items: Vec<Item> = vec![];
        assign_partition(&mut items, weight, get_side, set_side);
        assert!(items.is_empty());
    }

    #[test]
    fn single_unknown() {
        let mut items = vec![
            Item::new('A', 10, Side::LHS),
            Item::new('B', 5, Side::RHS),
            Item::new('C', 3, Side::Unknown),
        ];
        assign_partition(&mut items, weight, get_side, set_side);
        let lhs = sum_lhs(&items);
        let rhs = sum_rhs(&items);
        // Total=18, half=9. LHS=10, need x=-1 (impossible). Best: put C on LHS → LHS=13, RHS=5 diff=8; put on RHS → LHS=10, RHS=8 diff=2 → so C goes to RHS.
        assert_eq!(lhs, 10);
        assert_eq!(rhs, 8);
        assert_eq!(items[2].side, Side::RHS);
        // Fixed unchanged
        assert_eq!(items[0].side, Side::LHS);
        assert_eq!(items[1].side, Side::RHS);
    }

    #[test]
    fn mixed_with_zero() {
        let mut items = vec![
            Item::new('A', 0, Side::Unknown),
            Item::new('B', 5, Side::LHS),
            Item::new('C', 5, Side::RHS),
        ];
        assign_partition(&mut items, weight, get_side, set_side);
        let lhs = sum_lhs(&items);
        let rhs = sum_rhs(&items);
        // Total=10, half=5. LHS=5, RHS=5 already equal, zero can go anywhere without affecting diff.
        assert_eq!(lhs, rhs);
        assert!(items[0].side != Side::Unknown);
    }

    #[test]
    fn fixed_items_remain_unchanged() {
        let mut items = vec![
            Item::new('A', 5, Side::LHS),
            Item::new('B', 3, Side::RHS),
            Item::new('C', 2, Side::Unknown),
        ];
        let original_lhs_side = items[0].side;
        let original_rhs_side = items[1].side;
        assign_partition(&mut items, weight, get_side, set_side);
        assert_eq!(items[0].side, original_lhs_side);
        assert_eq!(items[1].side, original_rhs_side);
        // Only unknown changed
        assert_ne!(items[2].side, Side::Unknown);
    }
}
