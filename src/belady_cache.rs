#![allow(dead_code)]

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::path::Path;

use abstract_cache::{AccessResult, CacheSim, ObjIdTraits};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RawAccessTrace {
    ref_id: String,
    backward_ri: String,
    address: String,
    counter: usize,
}

#[derive(Debug)]
struct AccessTrace {
    ref_id: usize,
    backward_ri: usize,
    forward_ri: Option<usize>,
    address: usize,
    counter: usize,
}

fn read_csv<P: AsRef<Path>>(path: P) -> Result<Vec<AccessTrace>, Box<dyn Error>> {
    let mut reader = csv::Reader::from_reader(File::open(path)?);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: RawAccessTrace = result?;
        // record.forward_ri = None;

        // Parse hex values
        let ref_id = usize::from_str_radix(&record.ref_id, 16)?;
        let backward_ri = usize::from_str_radix(&record.backward_ri, 16)?;
        let address = usize::from_str_radix(&record.address, 16)?;

        records.push(AccessTrace {
            ref_id,
            backward_ri,
            forward_ri: None,
            address,
            counter: record.counter,
        });
    }

    calculate_forward_ri(&mut records);
    Ok(records)
}

fn calculate_forward_ri(records: &mut [AccessTrace]) {
    let mut last_seen: HashMap<usize, usize> = HashMap::new();

    for i in 0..records.len() {
        let address = records[i].address;
        if let Some(&next_index) = last_seen.get(&address) {
            let forward_ri = i - next_index;
            records[next_index].forward_ri = Some(forward_ri);
        }
        last_seen.insert(address, i);
    }
}

// #[derive(Debug, Clone, Hash, Eq, PartialEq)]
// struct AccessId {
//     distance: u64,
//     counter: u64,
// }
//
// impl std::fmt::Display for AccessId {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "({}, {})", self.ref_id, self.counter)
//     }
// }
//
// impl ObjIdTraits for AccessId {}

// A struct to store cache entries with their priority (distance)
#[derive(Eq, Hash, Clone, Debug)]
struct CacheEntry<ObjId: ObjIdTraits> {
    distance: usize,
    item: ObjId,
}

impl<Obj: ObjIdTraits> PartialEq<Self> for CacheEntry<Obj> {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item
    }
}

impl<Obj: ObjIdTraits> Display for CacheEntry<Obj> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.distance, self.item)
    }
}

impl <Obj: ObjIdTraits> ObjIdTraits for CacheEntry<Obj> {}

// Implement Ord and PartialOrd to make the heap a min-heap based on distance
// Implement Ord and PartialOrd to make the heap a min-heap based on distance
impl<ObjId: ObjIdTraits> Ord for CacheEntry<ObjId> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.cmp(&self.distance) // Reverse the order for min-heap
    }
}

impl<ObjId: ObjIdTraits> PartialOrd for CacheEntry<ObjId> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct OptCacheSim<ObjId: ObjIdTraits> {
    cache: BinaryHeap<ObjId>,
    cache_size: usize,
}

impl<ObjId: ObjIdTraits> OptCacheSim<ObjId> {
    pub fn new(cache_size: usize) -> Self {
        Self {
            cache: BinaryHeap::with_capacity(cache_size),
            cache_size,
        }
    }

    // fn decrement_distances(&mut self) {
    //     let mut updated_cache = BinaryHeap::with_capacity(self.cache_size);
    //
    //     while let Some(mut entry) = self.cache.pop() {
    //         if entry.distance != usize::MAX {
    //             entry.distance = entry.distance.saturating_sub(1);
    //         }
    //         updated_cache.push(entry);
    //     }
    //
    //     self.cache = updated_cache;
    // }
}

impl<ObjId: ObjIdTraits> CacheSim<CacheEntry<ObjId>> for OptCacheSim<ObjId> {
    fn cache_access(&mut self, access: CacheEntry<ObjId>) -> AccessResult {
        println!(" current cache: {:?}", self.cache.len());
        self.decrement_distances(); // Decrement all distances on each access

        if let Some(_pos) = self.cache.iter().position(|e| *e == access.item) {
            self.cache.push(access.item); // Insert the updated entry
            println!("pos: {}", _pos);
            AccessResult::Hit
        } else {
            // Cache miss
            if self.cache.len() >= self.cache_size {
                // Evict the element with the largest distance (lowest priority in min-heap)
                self.cache.pop();
            }
            self.cache.push(access.item);
            AccessResult::Miss
        }
    }

    fn set_capacity(&mut self, cache_size: usize) -> &mut Self {
        self.cache_size = cache_size;
        self.cache.clear();
        self
    }
}

fn temp() -> Result<(), Box<dyn Error>> {
    let path = "../out/access_trace.csv";
    let mut records = read_csv(path)?;

    calculate_forward_ri(&mut records);

    // Use AccessId as the key for cache simulation
    let trace = records.iter().map(|rec| {
        CacheEntry {
            distance: rec.forward_ri.unwrap_or(usize::MAX),
            item: rec.address,
        }
    });

    let mut cache = OptCacheSim::new(8);
    // let miss_ratio = cache.get_mr(trace.clone());
    let (total, miss) = cache.get_total_miss(trace);

    // println!("Miss ratio: {:.2}", miss_ratio);
    println!("Total: {}, Miss: {}", total, miss);

    Ok(())
}

// fn yifanopt() {
//     let path = "../out/access_trace.csv";
//     let mut records = read_csv(path).unwrap();
//
//     // calculate_forward_ri(&mut records);
//     let trace = records.iter().map(|rec| rec.address).collect();
//
//     let mr = opt_miss_ratio(trace, 90);
// }

#[cfg(test)]
mod tests {
    use dace_tests::polybench_simplify;
    use ri::tracing_ri_with_trace;

    use super::*;

    #[test]
    fn it_works() {
        let mut node = polybench_simplify::mvt(512);
        let _hist = tracing_ri_with_trace(&mut node, 512, 64);
    }

    #[test]
    fn read_csv_test() {
        let path = "../out/access_trace.csv";
        let records = read_csv(path).unwrap();

        for record in records {
            println!("{:?}", record);
        }
    }

    #[test]
    fn temp_test() {
        temp().unwrap();
    }
}