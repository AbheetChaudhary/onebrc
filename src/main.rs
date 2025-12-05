use std::collections::{
    BTreeMap,
    HashMap,
};
use std::io::{
    BufWriter,
    Write,
    Read,
};

fn reading_from_str(bytes: &[u8]) -> i16 {
    let len = bytes.len();

    match len {
        3 => {
            let mut reading = (bytes[len - 1] - b'0') as i16;
            reading += (bytes[len - 3] - b'0') as i16 * 10;
            reading
        }

        4 => {
            let first = bytes[0]; // maybe - or maybe digit
            let third = bytes[2]; // always .

            // high nibble of both - and . is 4, so if both are present then
            // negative will be true.
            let negative = ((first ^ third) >> 6) == 0;

            let mut reading = (bytes[len - 1] - b'0') as i16;
            reading += (bytes[len - 3] - b'0') as i16 * 10;

            (negative as i16) * (-reading) +
                (!negative as i16) * (bytes[0] - b'0') as i16 * 100
        }

        5 => {
            let mut reading = (bytes[len - 1] - b'0') as i16;
            reading += (bytes[len - 3] - b'0') as i16 * 10;
            reading += (bytes[len - 4] - b'0') as i16 * 100;
            -reading
        }
        _ => unreachable!(),
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        eprintln!("usage: {} <measurements.txt>", &args[0]);
        std::process::exit(-1);
    }

    let filename = &args[1];

    let mut file = std::fs::File::open(filename).unwrap();

    // let reader = std::io::BufReader::new(file);

    let mut read_buffer: Vec<u8> = Vec::with_capacity(128 * 1024 * 1024);
    _ = file.read_to_end(&mut read_buffer);

    let mut map: HashMap<&[u8], Vec<i16>> = HashMap::with_capacity(10000);

    for line in read_buffer.split(|x| *x == b'\n') {
        if line.is_empty() { continue; }

        let semicolon_idx = line.iter().position(|x| x == &b';').unwrap();

        let city = &line[..semicolon_idx];
        let temperature_bytes: &[u8] = &line[semicolon_idx + 1..line.len()];

        let temperature = reading_from_str(temperature_bytes);
        // let reading_scaled = reading_from_str_unchecked(reading.as_bytes());

        match map.get_mut(city) {
            Some(v) => v.push(temperature),
            None => {
                let mut v = Vec::with_capacity(128);
                v.push(temperature);
                map.insert(city, v);
            }
        }
    }

    let mut writer = BufWriter::with_capacity(512 * 1024 * 1024, std::io::stdout());

    let data_btree: BTreeMap<&[u8], &Vec<i16>> =
        BTreeMap::from_iter(map.iter().map(|(city, temperature)| {
            (*city, temperature)
        }));

    for (city, readings) in &data_btree {
        let len = readings.len();

        let mut min = i16::MAX;
        let mut max = i16::MIN;

        let mut sum: i64 = 0;

        for r in *readings {
            min = min.min(*r);

            max = max.max(*r);

            sum += *r as i64;
        }

        let min = min as f32 / 10.0;
        let max = max as f32 / 10.0;
        let avg = sum as f64 / len as f64 / 10.0;

        _ = writer.write(
            format!(
                "{}: {:.1}, {:.1}, {:.1}\n",
                unsafe { str::from_utf8_unchecked(city) },
                min,
                max,
                avg,
            ).as_bytes()
        );
    }

    _ = writer.flush();
}
