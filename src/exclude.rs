use std::{collections::BTreeSet, iter::FromIterator};

pub fn exclude<T>(mut items: Vec<T>, to_remove: Vec<T>) -> Vec<T>
where
    T: std::cmp::Ord,
{
    let to_remove = BTreeSet::from_iter(to_remove);
    items.retain(|e| !to_remove.contains(e));
    items
}