#![allow(unused)]
#![feature(allocator_api)]

use std::alloc::{Allocator, Layout};
use std::cell::LazyCell;
use std::collections::{BTreeMap, HashMap};
use std::io::{BufWriter, Write};
use std::hash::{Hasher, Hash};
use std::time;

/// Idiomatic rust prasing.
fn parse_base(read_buffer: &[u8]) {
    for line in read_buffer.split(|b| *b == b'\n') {
        if line.len() == 0 {
            break;
        }

        let semi_index = line.iter().position(|b| *b == b';').unwrap();

        let (city_name, temperature) = line.split_at_checked(semi_index).unwrap();

        std::hint::black_box((city_name, temperature));
    }
}

/// Manually indexing through the slice with a loop.
fn parse_manual_for(read_buffer: &[u8]) {
    let mut idx = 0;

    while idx < read_buffer.len() {
        while read_buffer[idx] != b';' {
            idx += 1;
        }

        let semi_idx = idx;

        let newline_idx = if read_buffer[idx + 4] == b'\n' {
            idx + 4
        } else if read_buffer[idx + 5] == b'\n' {
            idx + 5
        } else if read_buffer[idx + 6] == b'\n' {
            idx + 6
        } else {
            unsafe { std::hint::unreachable_unchecked(); }
        };

        idx = newline_idx + 1;

        std::hint::black_box(semi_idx);
        std::hint::black_box(newline_idx);
    }
}

/// `memchr` everywhere.
fn parse_memchr(read_buffer: &[u8]) {
    let read_buffer_ptr: *const u8 = read_buffer.as_ptr();
    let read_buffer_size = read_buffer.len();

    // Index of the next byte to process in the mmap'ed file.
    let mut begin_idx = 0;

    loop {
        let begin_ptr = unsafe { read_buffer_ptr.add(begin_idx) };
        let semicolon_ptr = unsafe {
            libc::memchr(
                begin_ptr as *const _,
                b';' as _,
                read_buffer_size - begin_idx,
            )
        } as *const u8;

        let city_name_len: usize = semicolon_ptr.addr() - begin_ptr.addr();

        let newline_ptr = unsafe {
            libc::memchr(
                semicolon_ptr.add(1) as *const _,
                b'\n' as _,
                read_buffer_size - begin_idx - city_name_len,
            )
        } as *const u8;

        let temperature_bytes_len =
            newline_ptr.addr() - semicolon_ptr.addr() - 1;

        begin_idx = newline_ptr.addr() + 1 - read_buffer_ptr.addr();

        if begin_idx == read_buffer_size {
            break;
        }

        std::hint::black_box(semicolon_ptr);
        std::hint::black_box(newline_ptr);
    }
}

/// `memchr` is only needed to parse the city name, once that is done the
/// temperature could be parsed with a single branch or maybe that branch could
/// also be avoided.
fn parse_better_memchr(read_buffer: &[u8]) {
    let buf_ptr = read_buffer.as_ptr();
    let buf_len = read_buffer.len();

    let mut idx = 0;

    loop {
        let begin_ptr = unsafe { buf_ptr.add(idx) };
        let semi_ptr = unsafe {
            libc::memchr(
                begin_ptr as *const _,
                b';' as _,
                // 64,
                104,
            )
        } as *const u8;

        let city_len = semi_ptr.addr() - begin_ptr.addr();
        let city_name = unsafe {
            std::slice::from_raw_parts(begin_ptr, city_len)
        };

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

        let temperature = unsafe {
            std::slice::from_raw_parts(semi_ptr.add(1), temp_len)
        };

        idx += city_len + 1 + temp_len + 1;

        if idx == buf_len {
            break;
        }

        std::hint::black_box(city_name);
        std::hint::black_box(temperature);
    }
}

// /// Manually unrolled loop. 2 unrolling. Gives 2x performance.
// fn parse_better_memchr_unrolled(read_buffer: &[u8]) {
//     let buf_ptr_1 = read_buffer.as_ptr();

//     let x = (&read_buffer[read_buffer.len() / 2..])
//         .iter().position(|b| *b == b'\n').unwrap();
//     let buf_len_1 = read_buffer.len() / 2 + x + 1;

//     let buf_len_2 = read_buffer.len() - buf_len_1;
//     let buf_ptr_2 = (&read_buffer[buf_len_1]) as *const u8;

//     let mut idx_1 = 0;
//     let mut idx_2 = 0;

//     while idx_1 < buf_len_1 && idx_2 < buf_len_2 {
//         let begin_ptr_1 = unsafe { buf_ptr_1.add(idx_1) };
//         let begin_ptr_2 = unsafe { buf_ptr_2.add(idx_2) };
//         let semi_ptr_1 = unsafe {
//             libc::strchr(
//                 begin_ptr_1 as *const _,
//                 b';' as _,
//                 // 64,
//                 // 104,
//             )
//         } as *const u8;
//         let semi_ptr_2 = unsafe {
//             libc::strchr(
//                 begin_ptr_2 as *const _,
//                 b';' as _,
//                 // 64,
//                 // 104,
//             )
//         } as *const u8;

//         let city_len_1 = semi_ptr_1.addr() - begin_ptr_1.addr();
//         let city_len_2 = semi_ptr_2.addr() - begin_ptr_2.addr();
//         let city_name_1 = unsafe {
//             std::slice::from_raw_parts(begin_ptr_1, city_len_1)
//         };

//         let city_name_2 = unsafe {
//             std::slice::from_raw_parts(begin_ptr_2, city_len_2)
//         };

//         let (x_1, y_1) = unsafe {
//             (semi_ptr_1.add(4).read(), semi_ptr_1.add(5).read())
//         };

//         let (x_2, y_2) = unsafe {
//             (semi_ptr_2.add(4).read(), semi_ptr_2.add(5).read())
//         };

//         let temp_len_1 = if x_1 == b'\n' {
//             3
//         } else if y_1 == b'\n' {
//             4
//         } else {
//             5
//         };

//         let temp_len_2 = if x_2 == b'\n' {
//             3
//         } else if y_2 == b'\n' {
//             4
//         } else {
//             5
//         };

//         let temperature_1 = unsafe {
//             std::slice::from_raw_parts(semi_ptr_1.add(1), temp_len_1)
//         };

//         let temperature_2 = unsafe {
//             std::slice::from_raw_parts(semi_ptr_2.add(1), temp_len_2)
//         };

//         idx_1 += city_len_1 + 1 + temp_len_1 + 1;
//         idx_2 += city_len_2 + 1 + temp_len_2 + 1;

//         std::hint::black_box((city_name_1, temperature_1));
//         std::hint::black_box((city_name_2, temperature_2));
//     }

//     while idx_1 < buf_len_1 {
//         let begin_ptr_1 = unsafe { buf_ptr_1.add(idx_1) };
//         let semi_ptr_1 = unsafe {
//             libc::strchr(
//                 begin_ptr_1 as *const _,
//                 b';' as _,
//                 // 64,
//                 // 104,
//             )
//         } as *const u8;

//         let city_len_1 = semi_ptr_1.addr() - begin_ptr_1.addr();
//         let city_name_1 = unsafe {
//             std::slice::from_raw_parts(begin_ptr_1, city_len_1)
//         };

//         let (x_1, y_1) = unsafe {
//             (semi_ptr_1.add(4).read(), semi_ptr_1.add(5).read())
//         };

//         let temp_len_1 = if x_1 == b'\n' {
//             3
//         } else if y_1 == b'\n' {
//             4
//         } else {
//             5
//         };

//         let temperature_1 = unsafe {
//             std::slice::from_raw_parts(semi_ptr_1.add(1), temp_len_1)
//         };

//         idx_1 += city_len_1 + 1 + temp_len_1 + 1;

//         std::hint::black_box((city_name_1, temperature_1));
//     }

//     while idx_2 < buf_len_2 {
//         let begin_ptr_2 = unsafe { buf_ptr_2.add(idx_2) };
//         let semi_ptr_2 = unsafe {
//             libc::strchr(
//                 begin_ptr_2 as *const _,
//                 b';' as _,
//                 // 64,
//                 // 104,
//             )
//         } as *const u8;

//         let city_len_2 = semi_ptr_2.addr() - begin_ptr_2.addr();

//         let city_name_2 = unsafe {
//             std::slice::from_raw_parts(begin_ptr_2, city_len_2)
//         };

//         let (x_2, y_2) = unsafe {
//             (semi_ptr_2.add(4).read(), semi_ptr_2.add(5).read())
//         };

//         let temp_len_2 = if x_2 == b'\n' {
//             3
//         } else if y_2 == b'\n' {
//             4
//         } else {
//             5
//         };

//         let temperature_2 = unsafe {
//             std::slice::from_raw_parts(semi_ptr_2.add(1), temp_len_2)
//         };

//         idx_2 += city_len_2 + 1 + temp_len_2 + 1;

//         std::hint::black_box((city_name_2, temperature_2));
//     }
// }

macro_rules! process {
    ($buf_ptr:expr, $idx:ident) => {{
        let begin_ptr = unsafe { $buf_ptr.add($idx) };
        let semi_ptr =
            unsafe { libc::strchr(begin_ptr as _, b';' as _) } as *const u8;

        let city_len = semi_ptr.addr() - begin_ptr.addr();

        let (x, y) =
            unsafe { (semi_ptr.add(4).read(), semi_ptr.add(5).read()) };

        let temp_len =
            if x == b'\n' { 3 } else if y == b'\n' { 4 } else { 5 };

        let city =
            unsafe { std::slice::from_raw_parts(begin_ptr, city_len) };

        let temp =
            unsafe { std::slice::from_raw_parts(semi_ptr.add(1), temp_len) };

        $idx += city_len + 1 + temp_len + 1;

        std::hint::black_box((city, temp));
    }};
}

// /// Better unrolling with a macro.
// fn parse_better_memchr_unrolled(read_buffer: &[u8]) {
//     let buf_ptr_1 = read_buffer.as_ptr();

//     let split1_base = read_buffer.len() / 2;
//     let x = (&read_buffer[split1_base..])
//         .iter().position(|b| *b == b'\n').unwrap();
//     let buf_len_1 = split1_base + x + 1;

//     let buf_ptr_2 = unsafe { buf_ptr_1.add(buf_len_1) };
//     let buf_len_2 = read_buffer.len() - buf_len_1;

//     let len_1 = buf_len_1;
//     let len_2 = buf_len_2;

//     let mut idx_1 = 0;
//     let mut idx_2 = 0;

//     while idx_1 < len_1 && idx_2 < len_2 {
//         process!(buf_ptr_1, idx_1);
//         process!(buf_ptr_2, idx_2);
//     }

//     while idx_1 < len_1 {
//         process!(buf_ptr_1, idx_1);
//     }

//     while idx_2 < len_2 {
//         process!(buf_ptr_2, idx_2);
//     }
// }

/// Manually unrolling 3 loops. Still scales! Works with 128million lines, does
/// not works with 1 billion lines, better paging could be used there.
fn parse_better_memchr_unrolled(read_buffer: &[u8]) {
    let buf_ptr_1 = read_buffer.as_ptr();

    // first split (1/3)
    let split1_base = read_buffer.len() / 3;
    let x1 = (&read_buffer[split1_base..])
        .iter().position(|b| *b == b'\n').unwrap();
    let buf_len_1 = split1_base + x1 + 1;

    // second split (2/3)
    let split2_base = 2 * read_buffer.len() / 3;
    let x2 = (&read_buffer[split2_base..])
        .iter().position(|b| *b == b'\n').unwrap();
    let end_2 = split2_base + x2 + 1;

    let buf_ptr_2 = unsafe { buf_ptr_1.add(buf_len_1) };
    let buf_ptr_3 = unsafe { buf_ptr_1.add(end_2) };

    let len_1 = buf_len_1;
    let len_2 = end_2 - buf_len_1;
    let len_3 = read_buffer.len() - end_2;

    let mut idx_1 = 0;
    let mut idx_2 = 0;
    let mut idx_3 = 0;

    while idx_1 < len_1 && idx_2 < len_2 && idx_3 < len_3 {
        process!(buf_ptr_1, idx_1);
        process!(buf_ptr_2, idx_2);
        process!(buf_ptr_3, idx_3);
    }

    while idx_1 < len_1 && idx_2 < len_2 {
        process!(buf_ptr_1, idx_1);
        process!(buf_ptr_2, idx_2);
    }

    while idx_1 < len_1 && idx_3 < len_3 {
        process!(buf_ptr_1, idx_1);
        process!(buf_ptr_3, idx_3);

    }

    while idx_2 < len_2 && idx_3 < len_3 {
        process!(buf_ptr_2, idx_2);
        process!(buf_ptr_3, idx_3);
    }

    while idx_1 < len_1 {
        process!(buf_ptr_1, idx_1);

    }

    while idx_3 < len_3 {
        process!(buf_ptr_3, idx_3);
    }

    while idx_2 < len_2 {
        process!(buf_ptr_2, idx_2);
    }
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
            libc::MAP_PRIVATE | libc::MAP_HUGE_2MB | libc::MAP_POPULATE,
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

    let begin = time::Instant::now();
    let file_buffer = mmap_read(&file);
    let elapsed_mmap = begin.elapsed();
    println!("mmap_populate: {}", elapsed_mmap.as_millis());

    // let begin = time::Instant::now();
    // parse_memchr(file_buffer.buffer);
    // let elapsed = begin.elapsed();
    // println!("parsing time(memchr): {}", elapsed.as_millis());

    // let begin = time::Instant::now();
    // parse_better_memchr(file_buffer.buffer);
    // let elapsed = begin.elapsed();
    // println!("parsing time(better_memchr): {}", elapsed.as_millis());

    let begin = time::Instant::now();
    parse_better_memchr_unrolled(file_buffer.buffer);
    let elapsed = begin.elapsed();
    println!("parsing time(better_memchr_unrolled): {}", elapsed.as_millis());

    /*
    let begin = time::Instant::now();
    parse_base(file_buffer.buffer);
    let elapsed = begin.elapsed();
    println!("parsing time(base): {}", elapsed.as_millis());

    let begin = time::Instant::now();
    parse_manual_for(file_buffer.buffer);
    let elapsed = begin.elapsed();
    println!("parsing time(manual_for): {}", elapsed.as_millis());
    */

}

