#![allow(unused)]
#![feature(cold_path)]

use std::collections::{
    BTreeMap,
    HashMap,
};
use std::io::{
    BufWriter,
    Write,
    Read,
};

use std::time;

use core::arch::x86_64::*;

#[inline(never)]
#[unsafe(no_mangle)]
unsafe extern "C" fn bcmp(s1: *const u8, s2: *const u8, size: usize) -> i32 {
    unsafe { libc::memcmp(s1 as *const _, s2 as *const _, size) }
}

fn reading_from_str(bytes: &[u8]) -> i16 {
    let len = bytes.len();

    match len {
        3 => {
            std::hint::cold_path();
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

struct CityHashBuilder;

impl std::hash::BuildHasher for CityHashBuilder {
    type Hasher = CityHasher;

    fn build_hasher(&self) -> Self::Hasher {
        CityHasher {
            state: 0xcbf29ce484222325,
        }
    }
}

struct CityHasher {
    state: u64,
}

impl std::hash::Hasher for CityHasher {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.state ^= byte as u64;
            self.state = self.state.wrapping_mul(0x100000001b3);
        }
    }
}

#[repr(transparent)]
struct CityName<'a> {
    name: &'a [u8],
}

impl<'a> PartialEq for CityName<'a> {
    fn eq(&self, other: &CityName) -> bool {
        if self.name.len() != other.name.len() {
            return false;
        }

        let len = self.name.len();

        unsafe {
            libc::memcmp(self.name.as_ptr() as _, other.name.as_ptr() as _, len) == 0
        }
    }
}

impl<'a> Eq for CityName<'a> {}

impl<'a> std::hash::Hash for CityName<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let ptr = self.name.as_ptr();

        match self.name.len() {
            0..16 => {
                // Less than 16 bytes
                state.write(&self.name);
            }

            16..32 => {
                // First 16 bytes
                state.write_u128(unsafe { *ptr.cast() });
                state.write(&self.name[16..]);
            }

            32..48 => {
                // First 16 bytes
                state.write_u128(unsafe { *ptr.cast() });
                // Next 16 bytes
                state.write_u128(unsafe { *ptr.add(16).cast() });
                state.write(&self.name[32..]);
            }

            48..64 => {
                // First 16 bytes
                state.write_u128(unsafe { *ptr.cast() });
                // Next 16 bytes
                state.write_u128(unsafe { *ptr.add(16).cast() });
                // Next 16 bytes
                state.write_u128(unsafe { *ptr.add(32).cast() });
                state.write(&self.name[48..]);
            }

            64..80 => {
                // First 16 bytes
                state.write_u128(unsafe { *ptr.cast() });
                // Next 16 bytes
                state.write_u128(unsafe { *ptr.add(16).cast() });
                // Next 16 bytes
                state.write_u128(unsafe { *ptr.add(32).cast() });
                // Next 16 bytes
                state.write_u128(unsafe { *ptr.add(48).cast() });
                state.write(&self.name[64..]);
            }

            80..96 => {
                // First 16 bytes
                state.write_u128(unsafe { *ptr.cast() });
                // Next 16 bytes
                state.write_u128(unsafe { *ptr.add(16).cast() });
                // Next 16 bytes
                state.write_u128(unsafe { *ptr.add(32).cast() });
                // Next 16 bytes
                state.write_u128(unsafe { *ptr.add(48).cast() });
                // Next 16 bytes
                state.write_u128(unsafe { *ptr.add(64).cast() });
                state.write(&self.name[80..]);
            }

            96..=100 => {
                // First 16 bytes
                state.write_u128(unsafe { *ptr.cast() });
                // Next 16 bytes
                state.write_u128(unsafe { *ptr.add(16).cast() });
                // Next 16 bytes
                state.write_u128(unsafe { *ptr.add(32).cast() });
                // Next 16 bytes
                state.write_u128(unsafe { *ptr.add(48).cast() });
                // Next 16 bytes
                state.write_u128(unsafe { *ptr.add(64).cast() });
                // Next 16 bytes
                state.write_u128(unsafe { *ptr.add(80).cast() });
                state.write(&self.name[96..]);
            }

            _ => unreachable!(),
        }
    }
}


/*
impl<'a> std::borrow::Borrow<[u8]> for CityName<'a> {
    fn borrow(&self) -> &[u8] {
        self.name
    }
}
*/

impl<'a> CityName<'a> {
    fn from(name: &'a [u8]) -> Self {
        CityName { name }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        eprintln!("usage: {} <measurements.txt>", &args[0]);
        std::process::exit(-1);
    }

    let filename = &args[1];

    let file = std::fs::File::open(filename).unwrap();

    let file_buffer = mmap_read(&file);

    let read_buffer_ptr: *const u8 = file_buffer.buffer.as_ptr();
    let read_buffer_size = file_buffer.buffer.len();

    let mut map: HashMap<CityName, Record, CityHashBuilder> =
        HashMap::with_capacity_and_hasher(10000, CityHashBuilder);

    let mut begin_idx = 0;

    loop {
        let semicolon_ptr = unsafe {
            libc::memchr(
                read_buffer_ptr.add(begin_idx) as *const _,
                b';' as _,
                read_buffer_size - begin_idx,
            )
        } as *const u8;

        let city_name_len: usize = unsafe {
            semicolon_ptr.addr() - read_buffer_ptr.add(begin_idx).addr()
        };

        let newline_ptr = unsafe {
            libc::memchr(
                semicolon_ptr.add(1) as *const _,
                b'\n' as _,
                read_buffer_size - begin_idx - city_name_len,
            )
        } as *const u8;

        let temperature_bytes_len =
            newline_ptr.addr() - semicolon_ptr.addr() - 1;

        let city = unsafe {
            std::slice::from_raw_parts(
                read_buffer_ptr.add(begin_idx),
                city_name_len,
            )
        };

        let temperature_bytes = unsafe {
            std::slice::from_raw_parts(
                semicolon_ptr.add(1),
                temperature_bytes_len
            )
        };

        let temperature = reading_from_str(temperature_bytes);

        map.entry(CityName::from(city)).and_modify(|e| e.update(temperature))
            .or_insert(Record::new(temperature));

        begin_idx = unsafe {
            newline_ptr.add(1).addr() - read_buffer_ptr.addr()
        } as usize;

        if begin_idx == read_buffer_size {
            break;
        }
    }

    let mut writer = BufWriter::with_capacity(512 * 1024 * 1024, std::io::stdout());

    let data_btree: BTreeMap<&[u8], &Record> =
        BTreeMap::from_iter(map.iter().map(|(city, record)| {
            (city.name, record)
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

use std::os::fd::{AsFd, AsRawFd};

struct FileBuffer<'a> {
    buffer: &'a [u8],
}

impl<'a> Drop for FileBuffer<'a> {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(
                self.buffer.as_ptr() as *mut u8 as *mut std::ffi::c_void,
                self.buffer.len()
            );
        }
    }
}

fn mmap_read<'a>(file: &'a std::fs::File) -> FileBuffer<'a> {
    let fd = file.as_fd().as_raw_fd();
    let size = file.metadata().unwrap().len() as usize;

    let mmap_ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ,
            libc::MAP_PRIVATE | libc::MAP_HUGE_1GB, /* | libc::MAP_POPULATE, */
            fd as std::ffi::c_int,
            0
        )
    };

    if mmap_ptr == libc::MAP_FAILED {
        panic!("mmap: {:?}", std::io::Error::last_os_error());
    }

    let advise_result = unsafe {
        libc::madvise(mmap_ptr, size, libc::MADV_SEQUENTIAL)
    };

    if advise_result != 0 {
        panic!("madvise: {:?}", std::io::Error::last_os_error());
    }

    let mmap_ptr = mmap_ptr as *const _ as *const u8;

    let buffer: &[u8] = unsafe { std::slice::from_raw_parts(mmap_ptr, size) };

    FileBuffer {
        buffer,
    }
}
