use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

#[derive(Debug)]
pub struct ReconcileResult {
    pub removed: Vec<usize>,
    pub added: Vec<usize>,
    pub moved: Vec<usize>,
}

pub fn reconcile<T: Eq + Hash>(
    prev: &[T],
    next: &[T],
) -> ReconcileResult {
    let prev_set: HashSet<_> = prev.iter().collect();
    let next_set: HashSet<_> = next.iter().collect();

    let prev_index: HashMap<_, _> =
        prev.iter().enumerate().map(|(i, e)| (e, i)).collect();
    // let next_index: HashMap<_, _> =
    //     next.iter().enumerate().map(|(i, e)| (e, i)).collect();

    let removed = prev
        .iter()
        .enumerate()
        .filter(|(_, e)| !next_set.contains(*e))
        .map(|(i, _)| i)
        .collect::<Vec<_>>();

    let added = next
        .iter()
        .enumerate()
        .filter(|(_, e)| !prev_set.contains(*e))
        .map(|(i, _)| i)
        .collect::<Vec<_>>();

    // indices in next whose predecessor changed
    let moved = next
        .iter()
        .enumerate()
        .filter(|(_, e)| prev_set.contains(*e)) // only existing elements
        .filter_map(|(next_i, e)| {
            let prev_i = prev_index[e];

            let old_pred = if prev_i == 0 {
                None
            } else {
                Some(&prev[prev_i - 1])
            };

            let new_pred = if next_i == 0 {
                None
            } else {
                Some(&next[next_i - 1])
            };

            // movement is defined by predecessor change
            if old_pred != new_pred {
                Some(next_i)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    ReconcileResult {
        removed,
        added,
        moved,
    }
}
