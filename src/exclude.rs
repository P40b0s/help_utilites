use std::{collections::BTreeSet, iter::FromIterator};

///delete objects from vec1 if is exists in vec2 if derive partialeq
pub fn exclude<T>(items: &mut Vec<T>, to_remove: Vec<T>)
where
    T: std::cmp::Ord,
{
    let to_remove = BTreeSet::from_iter(to_remove);
    items.retain(|e| !to_remove.contains(e));
}
///delete objects from vec1 if is exists in vec2 based on compare fn `cmp_func`
pub fn exclude_fn<T, F>(items: &mut Vec<T>, to_remove: Vec<T>, cmp_func: F)
where
    T: std::cmp::Ord,
    F: FnOnce(T, T) -> bool
{
    let to_remove = BTreeSet::from_iter(to_remove);
    items.retain(|e| !to_remove.contains(e));
}