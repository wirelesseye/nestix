use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

pub struct ReconcileResult<T> {
    pub removed: Vec<T>,
    pub added: Vec<(T, Option<T>)>,
    pub moved: Vec<(T, Option<T>)>,
}

pub fn reconcile<T: Clone + Eq + PartialEq + Hash>(
    prev: &[T],
    next: &[T],
) -> ReconcileResult<T> {
    let prev_set: HashSet<_> = prev.iter().cloned().collect();
    let next_set: HashSet<_> = next.iter().cloned().collect();

    let removed: Vec<T> = prev_set.difference(&next_set).cloned().collect();

    let prev_index: HashMap<_, _> = prev
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, e)| (e, i))
        .collect();
    // let next_index: HashMap<_, _> = next
    //     .iter()
    //     .cloned()
    //     .enumerate()
    //     .map(|(i, e)| (e, i))
    //     .collect();

    let previous_in_next = |idx: usize| {
        if idx == 0 {
            None
        } else {
            Some(next[idx - 1].clone())
        }
    };

    let added: Vec<(T, Option<T>)> = next
        .iter()
        .filter(|e| !prev_set.contains(*e))
        .enumerate()
        .map(|(i, e)| (e.clone(), previous_in_next(i)))
        .collect();

    let moved: Vec<(T, Option<T>)> = next
        .iter()
        .filter(|e| prev_set.contains(*e)) // only elements that existed before
        .enumerate()
        .filter_map(|(i, e)| {
            let e = e.clone();
            let old_idx = prev_index[&e];

            // Compute previous neighbor in prev
            let old_prev = if old_idx == 0 {
                None
            } else {
                Some(prev[old_idx - 1].clone())
            };

            let new_prev = previous_in_next(i);

            // Movement occurs if previous neighbor changed
            if old_prev != new_prev {
                Some((e, new_prev))
            } else {
                None
            }
        })
        .collect();

    ReconcileResult {
        removed,
        added,
        moved,
    }
}
