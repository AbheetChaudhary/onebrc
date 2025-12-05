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

struct Record {
    min:   i16, // current minimum scaled temperature
    max:   i16, // current maximum scaled temperature
    count: u32, // number of readings for this city
    sum:   i64, // sum of all the scaled temperature readings
}

impl Record {
    fn new(val: i16) -> Self {
        Record {
            min: val,
            max: val,
            count: 1,
            sum: val as _,
        }
    }

    fn update(&mut self, val: i16) {
        self.min = self.min.min(val);
        self.max = self.max.max(val);
        self.count += 1;
        self.sum += val as i64;
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

    let mut map: HashMap<&[u8], Record> = HashMap::with_capacity(10000);

    for line in read_buffer.split(|x| *x == b'\n') {
        if line.is_empty() { continue; }

        let semicolon_idx = line.iter().position(|x| x == &b';').unwrap();

        let city = &line[..semicolon_idx];
        let temperature_bytes: &[u8] = &line[semicolon_idx + 1..line.len()];

        let temperature = reading_from_str(temperature_bytes);
        // let reading_scaled = reading_from_str_unchecked(reading.as_bytes());

        match map.get_mut(city) {
            Some(v) => v.update(temperature),
            None => {
                map.insert(city, Record::new(temperature));
            }
        }
    }

    let mut writer = BufWriter::with_capacity(512 * 1024 * 1024, std::io::stdout());

    let data_btree: BTreeMap<&[u8], &Record> =
        BTreeMap::from_iter(map.iter().map(|(city, record)| {
            (*city, record)
        }));

    for (city, entry) in &data_btree {
        _ = writer.write(
            format!(
                "{}: {:.1}, {:.1}, {:.1}\n",
                unsafe { str::from_utf8_unchecked(city) },
                entry.min as f32 / 10.0,
                entry.max as f32 / 10.0,
                entry.sum as f64 / entry.count as f64 / 10.0,
            ).as_bytes()
        );
    }

    _ = writer.flush();
}
