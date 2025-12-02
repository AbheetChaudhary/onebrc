#![feature(f16)]
#![allow(unused)]

use std::collections::BTreeMap;
use std::io::BufRead;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        eprintln!("usage: {} <measurements.txt>", &args[0]);
        std::process::exit(-1);
    }

    let filename = &args[1];

    let file = std::fs::File::open(filename).unwrap();

    let mut reader = std::io::BufReader::new(file);

    let mut data: BTreeMap<String, Vec<f16>> = BTreeMap::new();

    let mut line = String::new();

    loop {
        match reader.read_line(&mut line) {
            Ok(0) => {
                break;
            }

            Ok(_) => {
                if line.is_empty() {
                    continue;
                }

                let newline_idx = line.len() - 1;
                let semicolon_idx = line.find(';').unwrap();

                let city = &line[..semicolon_idx];
                let reading: &str = &line[semicolon_idx + 1..newline_idx];
                let reading: f16 = reading.parse().unwrap();

                data.entry(city.to_owned())
                    .and_modify(|v| v.push(reading))
                    .or_insert(vec![reading]);
            }

            Err(e) => panic!("{e}")
        }

        line.clear();
    }

    for (city, readings) in data {
        let min = readings.iter().reduce(|acc, e| {
            if acc < e {
                acc
            } else {
                e
            }
        }).unwrap();
        let max = readings.iter().reduce(|acc, e| {
            if acc > e {
                acc
            } else {
                e
            }
        }).unwrap();
        let sum = readings.iter().fold(0.0f32, |acc, e| acc + *e as f32);
        let avg = sum / readings.len() as f32;

        println!("{city}: {min}, {max}, {avg}");
    }
}
