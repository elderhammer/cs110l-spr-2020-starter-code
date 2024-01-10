/* The following exercises were borrowed from Will Crichton's CS 242 Rust lab. */

use std::collections::HashSet;
use std::vec::Vec;

fn main() {
    println!("Hi! Try running \"cargo test\" to run tests.");
}

/// Implement add_n, which takes a vector of numbers and some number n.
/// The function should return a new vector whose elements are the numbers
/// in the original vector v with n added to each number.
fn add_n(v: Vec<i32>, n: i32) -> Vec<i32> {
    // unimplemented!()

    // ❌ 没审题
    // let mut v = v;
    // for number in v.iter_mut() {
    //     *number += n;
    // }
    // v

    // ✅
    let mut nv = Vec::new();
    for number in v {
        nv.push(number + n);
    }
    nv
}

/// Implement add_n_inplace, which does the same thing as add_n,
/// but modifies v directly (in place) and does not return anything.
fn add_n_inplace(v: &mut Vec<i32>, n: i32) {
    // unimplemented!()

    // 🚧 需要额外生成一个迭代器
    // for number in v.iter_mut() {
    //     *number += n;
    // }

    // ✅ 只需 range + 下标访问
    for i in 0..v.len() {
        v[i] = v[i] + n;
    }
}

/// Implement dedup that removes duplicate elements from a vector in-place (i.e. modifies v directly).
/// If an element is repeated anywhere in the vector, you should keep the element that appears first.
/// You may want to use a HashSet for this.
fn dedup(v: &mut Vec<i32>) {
    // unimplemented!()

    // 🚧 remove 耗时 O(n^2)
    // let mut numbers: HashSet<i32> = HashSet::new();
    // let mut idxs: Vec<i32> = Vec::new();
    // for i in 0..v.len() {
    //     if numbers.contains(&v[i]) {
    //         idxs.push(i as i32);
    //     } else {
    //         numbers.insert(v[i]);
    //     }
    // }
    // let mut j = 0i32;
    // for i in 0..idxs.len() {
    //     v.remove((idxs[i] - j) as usize);
    //     j = j + 1;
    // }

    // 相比于上面的逻辑，用空间换了时间
    let mut numbers: HashSet<i32> = HashSet::new();
    let mut nvec: Vec<i32> = Vec::new();
    for i in 0..v.len() {
        if !numbers.contains(&v[i]) {
            numbers.insert(v[i]);
            nvec.push(v[i]);
        }
    }
    v.clear();
    for i in 0..nvec.len() {
        v.push(nvec[i]);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_n() {
        assert_eq!(add_n(vec![1], 2), vec![3]);
    }

    #[test]
    fn test_add_n_inplace() {
        let mut v = vec![1];
        add_n_inplace(&mut v, 2);
        assert_eq!(v, vec![3]);
    }

    #[test]
    fn test_dedup() {
        let mut v = vec![3, 1, 0, 1, 4, 4];
        dedup(&mut v);
        assert_eq!(v, vec![3, 1, 0, 4]);
    }
}
