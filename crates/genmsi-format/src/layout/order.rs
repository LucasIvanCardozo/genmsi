//! The order of the blocks inside a layer.
//!
//! The blocks of a layer are put in the order that makes the fewest links
//! cross: a block goes near the blocks it is joined to. The order is worked
//! out with the **barycenter** of the neighbours (the average of the places
//! of the blocks a block is joined to), sweeping the layers up and down, and
//! then with the **transpose**: two neighbours in a layer are swapped when the
//! swap crosses fewer links.
//!
//! There is no luck in it: the same model always gives the same order.

use std::collections::BTreeMap;

/// How many times the layers are swept up and down.
const SWEEPS: usize = 4;

/// How many times the neighbours of a layer are tried.
const TRANSPOSE_ROUNDS: usize = 8;

/// Put the blocks of every layer in order.
///
/// The layers come in the order of the flow; the blocks inside them are
/// reordered. The edges are the links between the blocks of the layers.
pub fn order(layers: &mut [Vec<u32>], edges: &[(u32, u32)]) {
    if layers.len() < 2 {
        return;
    }
    let neighbours = neighbours_of(edges);
    for round in 0..SWEEPS {
        if round % 2 == 0 {
            sweep_down(layers, &neighbours);
        } else {
            sweep_up(layers, &neighbours);
        }
    }
    transpose(layers, &neighbours);
}

/// The blocks every block is joined to.
fn neighbours_of(edges: &[(u32, u32)]) -> BTreeMap<u32, Vec<u32>> {
    let mut neighbours: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (from, to) in edges {
        neighbours.entry(*from).or_default().push(*to);
        neighbours.entry(*to).or_default().push(*from);
    }
    for list in neighbours.values_mut() {
        list.sort_unstable();
        list.dedup();
    }
    neighbours
}

/// The order of the layers, from the first to the last.
fn sweep_down(layers: &mut [Vec<u32>], neighbours: &BTreeMap<u32, Vec<u32>>) {
    for index in 1..layers.len() {
        let previous = places(&layers[index - 1]);
        sort_by_barycenter(&mut layers[index], &previous, neighbours);
    }
}

/// The order of the layers, from the last to the first.
fn sweep_up(layers: &mut [Vec<u32>], neighbours: &BTreeMap<u32, Vec<u32>>) {
    for index in (0..layers.len().saturating_sub(1)).rev() {
        let next = places(&layers[index + 1]);
        sort_by_barycenter(&mut layers[index], &next, neighbours);
    }
}

/// Where every block of a layer is.
fn places(layer: &[u32]) -> BTreeMap<u32, usize> {
    layer
        .iter()
        .enumerate()
        .map(|(index, block)| (*block, index))
        .collect()
}

/// Sort a layer by the place of the neighbours of each block.
///
/// A block with no neighbour in the layer next to it keeps its place: the
/// sort is stable, so two blocks with the same barycenter keep the order they
/// had.
fn sort_by_barycenter(
    layer: &mut [u32],
    others: &BTreeMap<u32, usize>,
    neighbours: &BTreeMap<u32, Vec<u32>>,
) {
    let mut keyed: Vec<(f64, usize, u32)> = layer
        .iter()
        .enumerate()
        .map(|(place, block)| (barycenter(*block, others, neighbours), place, *block))
        .collect();
    keyed.sort_by(|first, second| {
        first
            .0
            .partial_cmp(&second.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(first.1.cmp(&second.1))
    });
    for (index, (_, _, block)) in keyed.into_iter().enumerate() {
        layer[index] = block;
    }
}

/// The average of the places of the neighbours of a block.
///
/// A block with no neighbour in the layer next to it has no barycenter: it
/// keeps the place it has.
fn barycenter(
    block: u32,
    others: &BTreeMap<u32, usize>,
    neighbours: &BTreeMap<u32, Vec<u32>>,
) -> f64 {
    let Some(list) = neighbours.get(&block) else {
        return f64::MAX;
    };
    let places: Vec<usize> = list
        .iter()
        .filter_map(|neighbour| others.get(neighbour).copied())
        .collect();
    if places.is_empty() {
        return f64::MAX;
    }
    let total: usize = places.iter().sum();
    total as f64 / places.len() as f64
}

/// Swap the neighbours of a layer when the swap crosses fewer links.
fn transpose(layers: &mut [Vec<u32>], neighbours: &BTreeMap<u32, Vec<u32>>) {
    for _ in 0..TRANSPOSE_ROUNDS {
        let mut improved = false;
        for index in 0..layers.len().saturating_sub(1) {
            let mut place = 0;
            while place + 1 < layers[index].len() {
                let before = crossings(layers, index, neighbours);
                layers[index].swap(place, place + 1);
                let after = crossings(layers, index, neighbours);
                if after < before {
                    improved = true;
                    place += 1;
                } else {
                    layers[index].swap(place, place + 1);
                    break;
                }
            }
        }
        if !improved {
            break;
        }
    }
}

/// The links that cross between two layers.
fn crossings(layers: &[Vec<u32>], first: usize, neighbours: &BTreeMap<u32, Vec<u32>>) -> usize {
    let second = first + 1;
    let places = places(&layers[second]);
    let mut links: Vec<(usize, usize)> = Vec::new();
    for (place, block) in layers[first].iter().enumerate() {
        let Some(list) = neighbours.get(block) else {
            continue;
        };
        for neighbour in list {
            if let Some(other) = places.get(neighbour) {
                links.push((place, *other));
            }
        }
    }
    let mut crossings = 0;
    for (index, first_link) in links.iter().enumerate() {
        for second_link in links.iter().skip(index + 1) {
            let crossed = (first_link.0 < second_link.0 && first_link.1 > second_link.1)
                || (first_link.0 > second_link.0 && first_link.1 < second_link.1);
            if crossed {
                crossings += 1;
            }
        }
    }
    crossings
}
