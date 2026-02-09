use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

#[derive(Debug)]
pub struct ReconcileResult {
    pub removed: Vec<usize>,
    pub mapping: Vec<Option<usize>>,
}

pub fn reconcile<T: Eq + Hash, P, N>(prev: &P, next: &N) -> ReconcileResult
where
    for<'a> &'a P: IntoIterator<Item = &'a T>,
    for<'a> &'a N: IntoIterator<Item = &'a T>,
{
    let next_set: HashSet<_> = next.into_iter().collect();
    let prev_index: HashMap<_, _> = prev.into_iter().enumerate().map(|(i, e)| (e, i)).collect();

    let mapping = next
        .into_iter()
        .map(|e| prev_index.get(e).copied())
        .collect::<Vec<_>>();

    let removed = prev
        .into_iter()
        .enumerate()
        .filter(|(_, e)| !next_set.contains(*e))
        .map(|(i, _)| i)
        .collect::<Vec<_>>();

    ReconcileResult { removed, mapping }
}
