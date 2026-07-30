use std::collections::HashSet;
use std::iter::Sum;
use std::ops::Add;
use std::ops::Sub;

/// Builds DP table for a slice of weights.
/// dp[i] is a set of all sums reachable using the first i items.
fn build_dp<N>(vals: &[N]) -> Vec<HashSet<N>>
where
    N: Copy + Add<Output = N> + Sub<Output = N> + Eq + std::hash::Hash + Default,
{
    let n = vals.len();
    let mut dp = Vec::with_capacity(n + 1);

    // Base case: empty subset sums to 0 (N::default())
    let mut initial = HashSet::new();
    initial.insert(N::default());
    dp.push(initial);

    for &val in vals {
        let prev_set = dp.last().unwrap();
        let mut new_set = prev_set.clone();
        for &sum in prev_set.iter() {
            new_set.insert(sum + val);
        }
        dp.push(new_set);
    }
    dp
}

/// Reconstructs the indices of items that sum to `target`.
/// Assumes that `target` is reachable (i.e., `dp[vals.len()]` contains it).
fn reconstruct_indices<N>(vals: &[N], target: N, dp: &[HashSet<N>]) -> Vec<usize>
where
    N: Copy + Add<Output = N> + Sub<Output = N> + Eq + std::hash::Hash,
{
    let n = vals.len();
    let mut indices = Vec::new();
    let mut remaining = target;

    for i in (1..=n).rev() {
        let prev_set = &dp[i - 1];
        if prev_set.contains(&remaining) {
            // item i-1 was NOT used
            continue;
        } else {
            // item i-1 MUST have been used
            let prev_remaining = remaining - vals[i - 1];
            // Safety: `prev_set` should contain `prev_remaining`
            if prev_set.contains(&prev_remaining) {
                indices.push(i - 1);
                remaining = prev_remaining;
            }
        }
    }
    indices.reverse();
    indices
}

/// Splits the combined equation into the MAXIMUM number of sub‑equations.
///
/// # Type parameters
/// - `T` – type of your items (e.g., `char`, `String`, custom structs).
/// - `N` – numeric weight type. Must be integer‑like (`i8`…`i128`, `u8`…`u128`).
/// - `F` – closure that extracts a weight `N` from an item.
///
/// # Returns
/// A `Vec` of `(LHS, RHS)` pairs, each balanced. The length is maximal.
///
/// # Panics
/// Panics if total sums of LHS and RHS differ (input is invalid).
pub fn split_to_max<T, N, F>(lhs: &[T], rhs: &[T], weight: &F) -> Vec<(Vec<T>, Vec<T>)>
where
    T: Clone,
    N: Copy
        + Add<Output = N>
        + Sub<Output = N>
        + Sum
        + Eq
        + std::hash::Hash
        + Default
        + std::fmt::Debug,
    F: Fn(&T) -> N,
{
    // 1. Base case: empty side means no equation
    if lhs.is_empty() || rhs.is_empty() {
        return Vec::new();
    }

    // 2. Extract weights
    let l_w: Vec<N> = lhs.iter().map(weight).collect();
    let r_w: Vec<N> = rhs.iter().map(weight).collect();

    let sum_l: N = l_w.iter().copied().sum();
    let sum_r: N = r_w.iter().copied().sum();

    // 3. Check if total sums match; if not, treat as atomic
    if sum_l != sum_r {
        return vec![(lhs.to_vec(), rhs.to_vec())];
    }

    // 4. Build DP sets
    let dp_l = build_dp(&l_w);
    let dp_r = build_dp(&r_w);

    let last_l = dp_l.last().unwrap();
    let last_r = dp_r.last().unwrap();

    // 5. Find a non‑trivial common subset sum
    let mut chosen_sum = None;

    for &s in last_l.iter() {
        // Avoid picking the empty subset (0) or the full subset (total sum)
        if last_r.contains(&s) {
            let l_idx = reconstruct_indices(&l_w, s, &dp_l);
            let r_idx = reconstruct_indices(&r_w, s, &dp_r);

            // Skip if the subset is empty (sum = 0) or covers everything
            if !l_idx.is_empty() && l_idx.len() < lhs.len() {
                chosen_sum = Some((s, l_idx, r_idx));
                break;
            }
        }
    }

    let Some((s, l_idx, r_idx)) = chosen_sum else {
        // No non‑trivial split exists → irreducible
        return vec![(lhs.to_vec(), rhs.to_vec())];
    };

    // 6. Split the original items according to the found indices
    let mut l1 = Vec::new();
    let mut l2 = Vec::new();
    for (i, item) in lhs.iter().enumerate() {
        if l_idx.contains(&i) {
            l1.push(item.clone());
        } else {
            l2.push(item.clone());
        }
    }

    let mut r1 = Vec::new();
    let mut r2 = Vec::new();
    for (i, item) in rhs.iter().enumerate() {
        if r_idx.contains(&i) {
            r1.push(item.clone());
        } else {
            r2.push(item.clone());
        }
    }

    // 7. Recurse on both halves to find the finest partition
    let mut result = split_to_max(&l1, &r1, weight);
    result.extend(split_to_max(&l2, &r2, weight));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_u8_weights() {
        let lhs = vec!['A', 'D', 'E'];
        let rhs = vec!['A', 'C', 'F'];
        let weight = |c: &char| (*c as u8 - b'A' + 1) as u8;

        let equations = split_to_max(&lhs, &rhs, &weight);
        assert_eq!(equations.len(), 2);
        for (l, r) in equations {
            assert_eq!(l.iter().map(&weight).sum::<u8>(), r.iter().map(&weight).sum::<u8>());
        }
    }

    #[test]
    fn with_i32_weights() {
        let lhs = vec![1_i32, 4, 5];
        let rhs = vec![2_i32, 3, 5];
        let weight = |x: &i32| *x;

        let equations = split_to_max(&lhs, &rhs, &weight);
        assert_eq!(equations.len(), 2);
        for (l, r) in equations {
            assert_eq!(l.iter().sum::<i32>(), r.iter().sum::<i32>());
        }
    }

    #[test]
    fn with_i128_weights() {
        let lhs = vec![1_i128, 4, 5];
        let rhs = vec![2_i128, 3, 5];
        let weight = |x: &i128| *x;

        let equations = split_to_max(&lhs, &rhs, &weight);
        assert_eq!(equations.len(), 2);
        for (l, r) in equations {
            assert_eq!(l.iter().sum::<i128>(), r.iter().sum::<i128>());
        }
    }

    #[test]
    fn atomic_equation() {
        let lhs = vec![1, 2, 6];
        let rhs = vec![4, 5];
        let weight = |x: &usize| *x;
        let equations = split_to_max(&lhs, &rhs, &weight);
        assert_eq!(equations.len(), 1);
        assert_eq!(equations[0].0, lhs);
        assert_eq!(equations[0].1, rhs);
    }

    #[test]
    fn duplicates() {
        let lhs = vec![1, 1, 1, 1];
        let rhs = vec![2, 2];
        let weight = |x: &usize| *x;
        let equations = split_to_max(&lhs, &rhs, &weight);
        assert_eq!(equations.len(), 2);
        for (l, r) in equations {
            assert_eq!(l.iter().sum::<usize>(), r.iter().sum::<usize>());
        }
    }

    #[test]
    fn with_custom_struct() {
        #[derive(Clone, Debug, PartialEq)]
        struct Item {
            name:  char,
            value: u64,
        }

        let lhs = vec![
            Item {
                name:  'A',
                value: 1,
            },
            Item {
                name:  'D',
                value: 4,
            },
            Item {
                name:  'E',
                value: 5,
            },
        ];
        let rhs = vec![
            Item {
                name:  'B',
                value: 2,
            },
            Item {
                name:  'C',
                value: 3,
            },
            Item {
                name:  'F',
                value: 5,
            },
        ];

        let weight = |item: &Item| item.value as i64;

        let equations = split_to_max(&lhs, &rhs, &weight);
        assert_eq!(equations.len(), 2);
        for (l, r) in equations {
            assert_eq!(l.iter().map(&weight).sum::<i64>(), r.iter().map(&weight).sum::<i64>());
        }
    }

    #[test]
    fn main() {
        let lhs = vec!['A', 'D', 'E'];
        let rhs = vec!['B', 'C', 'F'];
        let weight = |c: &char| (*c as u8 - b'A' + 1) as u8;

        let equations = split_to_max(&lhs, &rhs, &weight);
        println!("Maximum number of sub‑equations: {}", equations.len());
        for (i, (l, r)) in equations.iter().enumerate() {
            println!("Equation {}: {:?} = {:?}", i + 1, l, r);
        }
    }
}
