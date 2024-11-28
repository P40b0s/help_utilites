use std::{collections::BTreeSet, iter::FromIterator};

pub fn exclude<T>(items: &mut Vec<T>, to_remove: Vec<T>)
where
    T: std::cmp::Ord,
{
    let to_remove = BTreeSet::from_iter(to_remove);
    items.retain(|e| !to_remove.contains(e));
}

pub fn exclude_fn<T, F>(items: &mut Vec<T>, to_remove: Vec<T>, cmp_func: F)
where
    T: std::cmp::Ord,
    F: FnOnce(T, T) -> bool
{
    let to_remove = BTreeSet::from_iter(to_remove);
    items.retain(|e| !to_remove.contains(e));
}