/// Builds DP table for a slice of weights.
fn build_dp(weights: &[usize], max_sum: usize) -> Vec<Vec<bool>> {
    let n = weights.len();
    let mut dp = vec![vec![false; max_sum + 1]; n + 1];
    dp[0][0] = true;

    for i in 0..n {
        for s in 0..=max_sum {
            dp[i + 1][s] = dp[i][s];
            if s >= weights[i] && dp[i][s - weights[i]] {
                dp[i + 1][s] = true;
            }
        }
    }
    dp
}

/// Reconstructs indices of items that sum to `target`.
fn reconstruct_indices(weights: &[usize], target: usize, dp: &[Vec<bool>]) -> Vec<usize> {
    let n = weights.len();
    let mut indices = Vec::new();
    let mut s = target;

    for i in (1..=n).rev() {
        if s >= weights[i - 1] && dp[i - 1][s - weights[i - 1]] {
            indices.push(i - 1);
            s -= weights[i - 1];
        }
    }
    indices.reverse();
    indices
}

/// Recursively splits an equation into the maximum number of sub‑equations.
///
/// # Arguments
/// * `lhs` – left‑hand side items (any type `T`)
/// * `rhs` – right‑hand side items (any type `T`)
/// * `weight` – reference to a closure that extracts a `usize` weight from each item
///
/// # Returns
/// A vector of `(Vec<T>, Vec<T>)` pairs, each representing a balanced equation.
pub fn split_to_max<T: Clone, F: Fn(&T) -> usize>(
    lhs: &[T],
    rhs: &[T],
    weight: &F, // ← now always a reference
) -> Vec<(Vec<T>, Vec<T>)> {
    if lhs.is_empty() || rhs.is_empty() {
        return Vec::new();
    }

    let l_w: Vec<usize> = lhs.iter().map(weight).collect();
    let r_w: Vec<usize> = rhs.iter().map(weight).collect();

    let sum_l: usize = l_w.iter().sum();
    let sum_r: usize = r_w.iter().sum();

    if sum_l != sum_r || sum_l == 0 {
        return vec![(lhs.to_vec(), rhs.to_vec())];
    }

    let total = sum_l;
    let dp_l = build_dp(&l_w, total);
    let dp_r = build_dp(&r_w, total);

    let split_sum = (1..total).find(|&s| dp_l[l_w.len()][s] && dp_r[r_w.len()][s]);

    let Some(s) = split_sum else {
        return vec![(lhs.to_vec(), rhs.to_vec())];
    };

    let l_idx = reconstruct_indices(&l_w, s, &dp_l);
    let r_idx = reconstruct_indices(&r_w, s, &dp_r);

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

    // Recursive calls – pass `weight` (already a reference)
    let mut result = split_to_max(&l1, &r1, weight);
    result.extend(split_to_max(&l2, &r2, weight));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_with_symbols() {
        let lhs = vec!['A', 'D', 'E'];
        let rhs = vec!['A', 'C', 'F'];

        let weight = |c: &char| (*c as u8 - b'A' + 1) as usize;

        let equations = split_to_max(&lhs, &rhs, &weight); // ← pass as reference

        assert_eq!(equations.len(), 2);
        for (l, r) in equations {
            assert_eq!(l.iter().map(&weight).sum::<usize>(), r.iter().map(&weight).sum::<usize>());
        }
    }

    #[test]
    fn with_integers() {
        let lhs = vec![1, 4, 5];
        let rhs = vec![2, 3, 5];
        let weight = |x: &usize| *x;

        let equations = split_to_max(&lhs, &rhs, &weight);
        assert_eq!(equations.len(), 2);
        for (l, r) in equations {
            assert_eq!(l.iter().sum::<usize>(), r.iter().sum::<usize>());
        }
    }

    #[test]
    fn atomic() {
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
    fn mixed_values() {
        let lhs = vec![1, 2, 4, 8];
        let rhs = vec![3, 5, 7];
        let weight = |x: &usize| *x;
        let equations = split_to_max(&lhs, &rhs, &weight);
        assert_eq!(equations.len(), 2);
        for (l, r) in equations {
            assert_eq!(l.iter().sum::<usize>(), r.iter().sum::<usize>());
        }
    }

    #[test]
    fn main() {
        #[derive(Clone, Debug)]
        struct A {
            name: char,
            age:  u64,
        }

        let lhs = vec![
            A {
                name: 'A',
                age:  1,
            },
            A {
                name: 'D',
                age:  1,
            },
            A {
                name: 'E',
                age:  1,
            },
        ];
        let rhs = vec![
            A {
                name: 'B',
                age:  1,
            },
            A {
                name: 'C',
                age:  1,
            },
            A {
                name: 'F',
                age:  1,
            },
        ];

        let weight = |c: &A| (c.age) as usize;

        let equations = split_to_max(&lhs, &rhs, &weight);

        println!("Maximum number of sub‑equations: {}", equations.len());
        for (i, (l, r)) in equations.iter().enumerate() {
            println!("Equation {}: {:?} = {:?}", i + 1, l, r);
        }
    }
}
