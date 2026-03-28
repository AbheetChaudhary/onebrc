#![feature(allocator_api)]

use std::alloc::{Allocator, Layout};
use std::cell::LazyCell;
use std::collections::BTreeMap;
use std::io::{BufWriter, Write};
use std::mem::MaybeUninit;
use std::hash::{Hasher, Hash};
use fxhash::FxHashMap;

mod alloc {
    use std::alloc::{Allocator, Layout, AllocError};
    use std::ptr::NonNull;
    use std::cell::Cell;

    pub struct CityAllocator {
        arena: NonNull<u8>,
        next: Cell<NonNull<u8>>,
    }

    // Max size (in bytes) to store all the city names.
    const MAX_SIZE: usize = 100 * 10_000;

    impl CityAllocator {
        pub fn new() -> Self {
            let buffer = Box::into_raw(Box::new([0u8; MAX_SIZE]));

            let ptr = NonNull::new(buffer.cast::<u8>()).unwrap();

            Self {
                arena: ptr,
                next: Cell::new(ptr),
            }
        }
    }

    impl Drop for CityAllocator {
        fn drop(&mut self) {
            let mut ptr = self.arena.cast::<[u8; MAX_SIZE]>();

            // Deallocate the box.
            _ = unsafe { Box::from_raw(ptr.as_mut()) };
        }
    }

    unsafe impl Allocator for CityAllocator {
        fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
            if layout.size() > 100 {
                return Err(AllocError);
            }

            if self.next.get().addr().get() + layout.size() >
                self.arena.addr().get() + MAX_SIZE {
                return Err(AllocError);
            }

            let ptr = self.next.get();

            // bump the next pointer.
            self.next.set(unsafe { ptr.add(layout.size()) });

            Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
        }

        unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
            // nop
        }
    }
}

use alloc::CityAllocator;

thread_local! {
    static CITY_ALLOCATOR: LazyCell<CityAllocator> = LazyCell::new(|| {
        CityAllocator::new()
    });
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

/*
struct HashBuilder;

impl std::hash::BuildHasher for HashBuilder {
    type Hasher = FNV1Hasher;

    fn build_hasher(&self) -> Self::Hasher {
        FNV1Hasher {
            state: 0xcbf29ce484222325,
        }
    }
}

struct FNV1Hasher {
    state: u64,
}

impl std::hash::Hasher for FNV1Hasher {
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
*/

/*
union City {
    inline: CityInline,
    slice: CitySlice,
}

impl PartialEq for City {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for City {}

impl City {
    fn from_inline(ptr: *const u8, len: usize) -> Self {
        City {
            inline: CityInline::from(ptr, len),
        }
    }

    fn from_slice(slice: &[u8]) -> Self {
        City {
            slice: CitySlice::from(slice),
        }
    }

    fn is_inline(&self) -> bool {
        let len_ptr = unsafe { ((&raw const self.inline) as *const u8).add(15) };

        unsafe { len_ptr.read() != 0 }
    }

    fn as_slice(&self) -> &[u8] {
        if self.is_inline() {
            unsafe { self.inline.as_slice() }
        } else {
            unsafe { self.slice.as_slice() }
        }
    }
}

impl Hash for City {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.is_inline() {
            (unsafe { self.inline }).hash(state);
        } else {
            (unsafe { self.slice }).hash(state);
        }
    }
}
*/

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
struct CityInline {
    inlined: [u8; 16],
}

impl Hash for CityInline {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inlined.hash(state);
    }
}

impl CityInline {
    fn from(ptr: *const u8, len: usize) -> Self {
        // assert!(len < 16 && len > 0);
        let mut uninit: MaybeUninit<[u8; 16]> = MaybeUninit::zeroed();
        let ptr_to_bytes = uninit.as_mut_ptr().cast::<u8>();
        let ptr_to_len = unsafe { ptr_to_bytes.add(15) };

        unsafe {
            ptr_to_bytes.copy_from_nonoverlapping(ptr, len);
            ptr_to_len.write(len as u8);
        }

        CityInline {
            inlined: unsafe {
                uninit.assume_init()
            }
        }
    }

    fn as_slice(&self) -> &[u8] {
        let len = self.inlined[15];
        unsafe {
            std::slice::from_raw_parts(&self.inlined as *const _, len as usize)
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CitySlice {
    ptr: *const u8,
    len: usize,
}

impl CitySlice {
    // Create a new CitySlice from a given byte slice. The slice will be coming from
    // mmap'ed file, whose owned copy will be stored. The underlying memory
    // comes from an allocator designed to keep all the city names memory together.
    fn from(name: &[u8]) -> Self {
        let len = name.len();
        // assert!(len >= 16);

        let layout = Layout::array::<u8>(len).unwrap();

        // Allocate memory to hold the city name.
        let ptr = CITY_ALLOCATOR.with(|city_allocator| {
            city_allocator.allocate(layout)
        }).unwrap().cast::<u8>();

        // Copy from disk to newly allocated memory.
        unsafe { ptr.as_ptr().copy_from_nonoverlapping(name.as_ptr(), len); }

        Self {
            ptr: ptr.as_ptr() as *const _,
            len,
        }
    }

    fn as_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.ptr, self.len)
        }
    }
}

impl Hash for CitySlice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        self.as_slice()[..15].hash(state);
    }
}

impl PartialEq for CitySlice {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for CitySlice {}

fn parse(read_buffer: &[u8]) -> (FxHashMap<CityInline, Record>, FxHashMap<CitySlice, Record>) {
    let read_buffer_ptr: *const u8 = read_buffer.as_ptr();
    let read_buffer_size = read_buffer.len();

    let mut map_inlined: FxHashMap<CityInline, Record> = 
        FxHashMap::with_capacity_and_hasher(10000, fxhash::FxBuildHasher::new());

    let mut map_sliced: FxHashMap<CitySlice, Record> = 
        FxHashMap::with_capacity_and_hasher(10000, fxhash::FxBuildHasher::new());

    // Index of the next byte to process in the mmap'ed file.
    let mut begin_idx = 0;

    loop {
        let begin_ptr = unsafe { read_buffer_ptr.add(begin_idx) };
        let semi_ptr = unsafe {
            libc::memchr(
                begin_ptr as *const _,
                b';' as _,
                104,
            )
        } as *const u8;

        let (x, y) = unsafe {
            (semi_ptr.add(4).read(), semi_ptr.add(5).read())
        };

        let temp_len = if x == b'\n' {
            3
        } else if y == b'\n' {
            4
        } else {
            5
        };

        let temperature_bytes = unsafe {
            std::slice::from_raw_parts(semi_ptr.add(1), temp_len)
        };

        let temperature = reading_from_str(temperature_bytes);

        let city_name_len: usize = semi_ptr.addr() - begin_ptr.addr();

        if city_name_len < 16 {
            let city = CityInline::from(begin_ptr, city_name_len);

            map_inlined.entry(city).and_modify(|record| record.update(temperature))
                .or_insert(Record::new(temperature));
        } else {
            let city = CitySlice {
                ptr: begin_ptr,
                len: city_name_len,
            };

            // map_sliced.entry(city).and_modify(|record| record.update(temperature))
            //     .or_insert(Record::new(temperature));

            match map_sliced.get_mut(&city) {
                Some(v) => {
                    v.update(temperature);
                }
                None => {
                    std::hint::cold_path();

                    // This allocates using CityAllocator.
                    let city_name = CitySlice::from(city.as_slice());
                    _ = map_sliced.insert(city_name, Record::new(temperature));
                }
            }
        };

        /*
        map.entry(city).and_modify(|record| record.update(temperature))
            .or_insert(Record::new(temperature));
        */

        begin_idx += city_name_len + 1 + temp_len + 1;

        if begin_idx == read_buffer_size {
            break;
        }
    }

    (map_inlined, map_sliced)
}

fn print_sorted_slice(map: &FxHashMap<CitySlice, Record>) {
    let mut writer = BufWriter::with_capacity(512 * 1024 * 1024, std::io::stdout());

    let data_btree: BTreeMap<&[u8], &Record> =
        BTreeMap::from_iter(map.iter().map(|(city, record)| {
            (city.as_slice(), record)
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

fn print_sorted_inlined(map: &FxHashMap<CityInline, Record>) {
    let mut writer = BufWriter::with_capacity(512 * 1024 * 1024, std::io::stdout());

    let data_btree: BTreeMap<&[u8], &Record> =
        BTreeMap::from_iter(map.iter().map(|(city, record)| {
            (city.as_slice(), record)
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

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        eprintln!("usage: {} <measurements.txt>", &args[0]);
        std::process::exit(-1);
    }

    let filename = &args[1];

    let file = std::fs::File::open(filename).unwrap();

    let file_buffer = mmap_read(&file);

    let (map_inlined, map_slice) = parse(file_buffer.buffer);

    print_sorted_inlined(&map_inlined);
    print_sorted_slice(&map_slice);
}

