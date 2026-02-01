//Reluctation points

use crate::shared;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
#[derive(Debug, Clone, Copy)]
pub struct InsertPointResult {
    pub lda: f32,
    pub best_a: shared::Point,
    pub best_c: shared::Point,
}

#[derive(Debug, Clone, Copy)]
struct LdaEntry {
    lda: f32,
    point: shared::Point, // inserted point
}

impl PartialEq for LdaEntry {
    fn eq(&self, other: &Self) -> bool {
        self.lda == other.lda
    }
}

impl Eq for LdaEntry {}

impl PartialOrd for LdaEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.lda.partial_cmp(&other.lda)
    }
}

impl Ord for LdaEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub fn find_lowest_lda_points(insert_log: &[InsertPointResult], k: usize) -> Vec<shared::Point> {
    let mut heap = BinaryHeap::with_capacity(k);

    for result in insert_log {
        let entry = LdaEntry {
            lda: result.lda,
            point: result.best_c,
        };

        if heap.len() < k {
            heap.push(entry);
        } else if let Some(&top) = heap.peek() {
            if entry.lda < top.lda {
                heap.pop();
                heap.push(entry);
            }
        }
    }

    // Convert heap into sorted vec of Points (lowest to highest LDA)
    heap.into_sorted_vec()
        .into_iter()
        .map(|entry| entry.point)
        .collect()
}

pub fn remove_points_from_hull(hull: &mut Vec<shared::Point>, to_remove: &[shared::Point]) {
    const EPS: f32 = 1e-5;

    fn eq(a: shared::Point, b: shared::Point) -> bool {
        (a.x - b.x).abs() < EPS && (a.y - b.y).abs() < EPS
    }

    hull.retain(|p| !to_remove.iter().any(|r| eq(*p, *r)));
}
