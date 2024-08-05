#![allow(dead_code)]
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AccessTrace {
    pub ref_id: u64,
    backward_ri: u64,
    forward_ri: Option<u64>,
    address: u64,
    counter: u64,
}

fn read_csv<P: AsRef<Path>>(path: P) -> Result<Vec<AccessTrace>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let mut record: AccessTrace = result?;
        record.forward_ri = None;
        records.push(record);
    }

    Ok(records)
}

fn calculate_forward_ri(records: &mut [AccessTrace]) {
    let mut last_seen: HashMap<u64, usize> = HashMap::new();

    for i in 0..records.len() {
        let address = records[i].address;
        if let Some(&next_index) = last_seen.get(&address) {
            let forward_ri = (i - next_index) as u64;
            records[next_index].forward_ri = Some(forward_ri);
        }
        last_seen.insert(address, i);
    }
}

// fn create_mock_csv<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error>> {
//     let mut file = OpenOptions::new().write(true).create(true).open(path)?;
//     writeln!(file, "ref_id,froward_ri,address,counter")?;
//     writeln!(file, "1,10,1000,1")?;
//     writeln!(file, "2,20,2000,2")?;
//     Ok(())
// }

fn main() -> Result<(), Box<dyn Error>> {
    let path = "../out/access_trace.csv";
    let mut records = read_csv(path)?;

    calculate_forward_ri(&mut records);

    for record in records {
        println!("{:?}", record);
    }

    Ok(())
}

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
        main().unwrap();
        // let path = "../out/access_trace.csv";
        // if !Path::new(path).exists() {
        //     create_mock_csv(path).unwrap();
        // }
        // let records = read_csv(path).unwrap();
        // println!("{:?}", records);
    }
}
