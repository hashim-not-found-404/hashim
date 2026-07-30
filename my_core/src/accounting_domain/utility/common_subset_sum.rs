/// Builds a DP table: dp[i][s] = true if sum `s` can be made using the first `i` elements.
fn build_dp(vals: &[usize], max_sum: usize) -> Vec<Vec<bool>> {
    let n = vals.len();
    let mut dp = vec![vec![false; max_sum + 1]; n + 1];
    dp[0][0] = true;

    for i in 0..n {
        for s in 0..=max_sum {
            dp[i + 1][s] = dp[i][s]; // skip item i
            if s >= vals[i] && dp[i][s - vals[i]] {
                dp[i + 1][s] = true; // take item i
            }
        }
    }
    dp
}

/// Reconstructs the indices of elements that sum to `target`.
/// Assumes `dp[vals.len()][target]` is true.
fn reconstruct_indices(vals: &[usize], target: usize, dp: &[Vec<bool>]) -> Vec<usize> {
    let n = vals.len();
    let mut indices = Vec::new();
    let mut s = target;

    for i in (1..=n).rev() {
        if s >= vals[i - 1] && dp[i - 1][s - vals[i - 1]] {
            indices.push(i - 1);
            s -= vals[i - 1];
        }
        // else: item i-1 was not used, so we skip it
    }
    indices.reverse(); // return in increasing index order
    indices
}

/// Recursively splits until no further split is possible.
/// Returns a vector of all irreducible equations: (LHS, RHS).
pub fn split_to_max(lhs: &[usize], rhs: &[usize]) -> Vec<(Vec<usize>, Vec<usize>)> {
    // Base case: if either side is empty, there is no equation.
    if lhs.is_empty() || rhs.is_empty() {
        return Vec::new();
    }

    let sum_l: usize = lhs.iter().sum();
    let sum_r: usize = rhs.iter().sum();

    // Must have equal total sums, otherwise it's invalid – treat as atomic.
    if sum_l != sum_r || sum_l == 0 {
        return vec![(lhs.to_vec(), rhs.to_vec())];
    }

    let total = sum_l;

    let dp_l = build_dp(lhs, total);
    let dp_r = build_dp(rhs, total);

    // Find the smallest non‑trivial common sum S (1 ≤ S < total).
    let split_sum = (1..total).find(|&s| dp_l[lhs.len()][s] && dp_r[rhs.len()][s]);

    let Some(s) = split_sum else {
        // No non‑trivial split exists – this equation is irreducible.
        return vec![(lhs.to_vec(), rhs.to_vec())];
    };

    // Reconstruct the indices for the chosen subset on each side.
    let l_idx = reconstruct_indices(lhs, s, &dp_l);
    let r_idx = reconstruct_indices(rhs, s, &dp_r);

    // Split the LHS into two parts: used (l1) and leftover (l2).
    let mut l1 = Vec::new();
    let mut l2 = Vec::new();
    for (i, &val) in lhs.iter().enumerate() {
        if l_idx.contains(&i) {
            l1.push(val);
        } else {
            l2.push(val);
        }
    }

    // Split the RHS into two parts: used (r1) and leftover (r2).
    let mut r1 = Vec::new();
    let mut r2 = Vec::new();
    for (i, &val) in rhs.iter().enumerate() {
        if r_idx.contains(&i) {
            r1.push(val);
        } else {
            r2.push(val);
        }
    }

    // Recurse on both halves and combine the results.
    let mut result = split_to_max(&l1, &r1);
    result.extend(split_to_max(&l2, &r2));
    result
}

#[cfg(test)]
mod tests {
    use super::split_to_max;

    #[test]
    fn example_from_prompt() {
        // A=1, D=4, E=5  and  B=2, C=3, F=5
        let lhs = vec![1, 4, 5];
        let rhs = vec![2, 3, 5];
        let equations = split_to_max(&lhs, &rhs);

        // We expect two equations: 5 = 2+3  and  1+4 = 5
        assert_eq!(equations.len(), 2);

        for (l, r) in equations {
            assert_eq!(l.iter().sum::<usize>(), r.iter().sum::<usize>());
        }
    }

    #[test]
    fn maximum_split_example() {
        // L = [1,2,3], R = [1,2,3] → can be split into 3 equations: 1=1, 2=2, 3=3
        let lhs = vec![1, 2, 3];
        let rhs = vec![1, 2, 3];
        let equations = split_to_max(&lhs, &rhs);

        assert_eq!(equations.len(), 3);

        for (l, r) in equations {
            assert_eq!(l.iter().sum::<usize>(), r.iter().sum::<usize>());
        }
    }

    #[test]
    fn atomic_equation() {
        // L = [1,2,6], R = [4,5] → total=9, only common sums are 0 and 9 → no split
        let lhs = vec![1, 2, 6];
        let rhs = vec![4, 5];
        let equations = split_to_max(&lhs, &rhs);

        assert_eq!(equations.len(), 1);
        assert_eq!(equations[0].0, lhs);
        assert_eq!(equations[0].1, rhs);
    }

    #[test]
    fn duplicates() {
        // L = [1,1,1,1], R = [2,2] → max splits = 2: each [1,1] = [2]
        let lhs = vec![1, 1, 1, 1];
        let rhs = vec![2, 2];
        let equations = split_to_max(&lhs, &rhs);

        assert_eq!(equations.len(), 2);

        for (l, r) in equations {
            assert_eq!(l.iter().sum::<usize>(), r.iter().sum::<usize>());
        }
    }

    #[test]
    fn mixed_values() {
        // L = [1,2,4,8], R = [3,5,7] → total=15, max splits = 2
        let lhs = vec![1, 2, 4, 8];
        let rhs = vec![3, 5, 7];
        let equations = split_to_max(&lhs, &rhs);

        assert_eq!(equations.len(), 2);

        for (l, r) in equations {
            assert_eq!(l.iter().sum::<usize>(), r.iter().sum::<usize>());
        }
    }

    #[test]
    fn main() {
        // Example: A=1, D=4, E=5  and  B=2, C=3, F=5
        let lhs = vec![1, 4, 5];
        let rhs = vec![2, 3, 5];

        let equations = split_to_max(&lhs, &rhs);

        println!("Maximum number of sub‑equations: {}", equations.len());
        for (i, (l, r)) in equations.iter().enumerate() {
            println!("Equation {}: {:?} = {:?}", i + 1, l, r);
        }
    }
}
