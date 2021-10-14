//! Generic merge function called by code from Toql derive.
//! Used to merge a collection of structs into another collection of structs by equal keys

use std::collections::HashMap;

pub fn merge<A, K, B, F, G>(
    entities: &mut Vec<A>,
    pkeys: &[K],
    merges: Vec<B>,
    mkeys: &[K],
    init: F,
    assign: G,
) where
    F: Fn(&mut A),
    G: Fn(&mut A, B),
    K: std::cmp::Eq + std::hash::Hash,
{
    let mut index: HashMap<&K, &mut A> = HashMap::new();

    // build up index
    entities
        .iter_mut()
        .zip(pkeys.iter())
        .for_each(|(mut e, k)| {
            init(&mut e);
            index.insert(k, e);
        });

    // Now compare keys and merge into
    merges.into_iter().zip(mkeys.iter()).for_each(|(e, k)| {
        if index.contains_key(k) {
            let entity: &mut A = index.get_mut(k).unwrap();
            assign(entity, e);
        }
    });
}
