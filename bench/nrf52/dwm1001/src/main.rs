#![no_std]
#![no_main]
#![feature(asm)]

// Used to define panic behavior
#[allow(unused_imports)]
use panic_ramdump;
use core::fmt::Write;
use postcard;

// Provides definitions for our development board
use dwm1001::{
    nrf52832_hal::{
        prelude::*,
        Timer,
        nrf52832_pac::{
            TIMER0,
        },
    },
    cortex_m::asm::nop,
    DWM1001,
};

use heapless::{String, Vec, consts::*};
use serde::{Serialize, Deserialize};

// Used to set the program entry point
use cortex_m_rt::entry;
use core::sync::atomic::{compiler_fence, Ordering};

use serde_json_core;

// Note: nrf52832 @ 64MHz

// opt-level = 'z'          Loop Iters/sec   Time compared  Clock cycles
//                                            to counter     per loop iter
// baseline               - ops/s: 03999999, rel: 00000001, cyc/op: 00016
// u8_slice_32_slice      - ops/s: 00035956, rel: 00000111, cyc/op: 01779
// u8_slice_32_vec        - ops/s: 00034335, rel: 00000116, cyc/op: 01863
// u8_slice_32_cobvec     - ops/s: 00025519, rel: 00000156, cyc/op: 02507
// miscdata_slice         - ops/s: 00048485, rel: 00000082, cyc/op: 01319
// miscdata_vec           - ops/s: 00038370, rel: 00000104, cyc/op: 01667
// miscdata_cobvec        - ops/s: 00010390, rel: 00000384, cyc/op: 06159
// u8_slice_32_vec_sjc    - ops/s: 00009593, rel: 00000416, cyc/op: 06671
// miscdata_vec_sjc       - ops/s: 00003813, rel: 00001049, cyc/op: 16784
// u8_slice_32_slice_ssm  - ops/s: 00421053, rel: 00000009, cyc/op: 00151

// opt-level = 3            Loop Iters/sec   Time compared  Clock cycles
//                                            to counter     per loop iter
// baseline               - ops/s: 04571425, rel: 00000001, cyc/op: 00014
// u8_slice_32_slice      - ops/s: 00207793, rel: 00000021, cyc/op: 00307
// u8_slice_32_vec        - ops/s: 01999999, rel: 00000002, cyc/op: 00032
// u8_slice_32_cobvec     - ops/s: 00036200, rel: 00000126, cyc/op: 01767
// miscdata_slice         - ops/s: 00615384, rel: 00000007, cyc/op: 00104
// miscdata_vec           - ops/s: 00098766, rel: 00000046, cyc/op: 00647
// miscdata_cobvec        - ops/s: 00017506, rel: 00000261, cyc/op: 03655
// u8_slice_32_vec_sjc    - ops/s: 00015656, rel: 00000291, cyc/op: 04087
// miscdata_vec_sjc       - ops/s: 00030535, rel: 00000149, cyc/op: 02095
// u8_slice_32_slice_ssm  - ops/s: 03200000, rel: 00000001, cyc/op: 00020

#[entry]
fn main() -> ! {
    // Access the device hardware
    let mut board = DWM1001::take().unwrap();
    let mut timer = board.TIMER0.constrain();
    let mut sbuf: String<U1024> = String::new();

    let benches: &[(&str, fn(&mut Timer<TIMER0>) -> u32)] = &[
        ("baseline              ", baseline),

        ("u8_slice_32_slice     ", u8_slice_32_slice),
        ("u8_slice_32_vec       ", u8_slice_32_vec),
        ("u8_slice_32_cobvec    ", u8_slice_32_cobvec),
        ("miscdata_slice        ", miscdata_slice),
        ("miscdata_vec          ", miscdata_vec),
        ("miscdata_cobvec       ", miscdata_cobvec),

        ("u8_slice_32_vec_sjc   ", u8_slice_32_vec_sjc),
        ("miscdata_vec_sjc      ", miscdata_vec_sjc),

        ("u8_slice_32_slice_ssm ", u8_slice_32_slice_ssm),
        // ("miscdata_slice_ssm    ", miscdata_slice_ssm),

    ];

    let mut last_baseline = 0xFFFFFFFF;

    loop {
        for b in benches {
            sbuf.clear();

            compiler_fence(Ordering::SeqCst);

            let ops_per_sec = (b.1)(&mut timer);

            if b.0 == benches[0].0 {
                last_baseline = ops_per_sec;
            }

            compiler_fence(Ordering::SeqCst);

            write!(
                &mut sbuf,
                "{} - ops/s: {:08}, rel: {:08}, cyc/op: {:05}\r\n",
                b.0,
                ops_per_sec,
                last_baseline / ops_per_sec,
                64_000_000 / ops_per_sec,
            ).unwrap();
            board.uart.write(sbuf.as_bytes()).unwrap();
        }

        let mut data = [b'\r', b'\n'];
        board.uart.write(&data).unwrap();
        timer.start(3_000_000u32);
        while timer.wait().is_err() {
            nop();
        }

    }
}

#[derive(Serialize, Deserialize)]
struct MiscData<'a> {
    a: u32,
    b: u8,
    c: &'a str,
    d: &'a [u8],
    e: i64,
    f: bool,
}

fn baseline(timer: &mut Timer<TIMER0>) -> u32 {
    let mut ops_per_sec = 0;

    timer.start(1_000_000u32);
    while timer.wait().is_err() {
        ops_per_sec += 1;
    }

    ops_per_sec
}

fn u8_slice_32_slice(timer: &mut Timer<TIMER0>) -> u32 {
    let mut ops_per_sec = 0;

    let mut data = [0u8; 32];
    for (i, d) in data.iter_mut().enumerate() {
        *d = i as u8;
    }
    let mut buf = [0u8; 128];

    timer.start(1_000_000u32);
    while timer.wait().is_err() {
        black_box(postcard::to_slice(&data, &mut buf).unwrap());
        ops_per_sec += 1;
    }

    ops_per_sec
}

fn u8_slice_32_vec(timer: &mut Timer<TIMER0>) -> u32 {
    let mut ops_per_sec = 0;

    let mut data = [0u8; 32];
    for (i, d) in data.iter_mut().enumerate() {
        *d = i as u8;
    }

    timer.start(1_000_000u32);
    while timer.wait().is_err() {
        black_box::<Vec<u8, U33>>(postcard::to_vec(&data).unwrap());
        ops_per_sec += 1;
    }

    ops_per_sec
}

fn u8_slice_32_cobvec(timer: &mut Timer<TIMER0>) -> u32 {
    let mut ops_per_sec = 0;

    let mut data = [0u8; 32];
    for (i, d) in data.iter_mut().enumerate() {
        *d = i as u8;
    }

    timer.start(1_000_000u32);
    while timer.wait().is_err() {
        black_box::<Vec<u8, U40>>(postcard::to_vec_cobs(&data).unwrap());
        ops_per_sec += 1;
    }

    ops_per_sec
}

fn miscdata_slice(timer: &mut Timer<TIMER0>) -> u32 {
    let mut ops_per_sec = 0;

    let data = MiscData {
        a: 1023,
        b: 27,
        c: "Hi!",
        d: &[0, 10, 50, 20, 3],
        e: 0x0F00_0000_000F_A000,
        f: false,
    };

    let mut buf = [0u8; 128];

    timer.start(1_000_000u32);
    while timer.wait().is_err() {
        black_box(postcard::to_slice(&data, &mut buf).unwrap());
        ops_per_sec += 1;
    }

    ops_per_sec
}

fn miscdata_vec(timer: &mut Timer<TIMER0>) -> u32 {
    let mut ops_per_sec = 0;

    let data = MiscData {
        a: 1023,
        b: 27,
        c: "Hi!",
        d: &[0, 10, 50, 20, 3],
        e: 0x0F00_0000_000F_A000,
        f: false,
    };


    timer.start(1_000_000u32);
    while timer.wait().is_err() {
        black_box::<Vec<u8, U33>>(postcard::to_vec(&data).unwrap());
        ops_per_sec += 1;
    }

    ops_per_sec
}

fn miscdata_cobvec(timer: &mut Timer<TIMER0>) -> u32 {
    let mut ops_per_sec = 0;

    let data = MiscData {
        a: 1023,
        b: 27,
        c: "Hi!",
        d: &[0, 10, 50, 20, 3],
        e: 0x0F00_0000_000F_A000,
        f: false,
    };

    timer.start(1_000_000u32);
    while timer.wait().is_err() {
        black_box::<Vec<u8, U128>>(postcard::to_vec_cobs(&data).unwrap());
        ops_per_sec += 1;
    }

    ops_per_sec
}

pub fn black_box<T>(dummy: T) -> T {
    // we need to "use" the argument in some way LLVM can't
    // introspect.
    unsafe {asm!("" : : "r"(&dummy))}
    dummy
}

fn u8_slice_32_vec_sjc(timer: &mut Timer<TIMER0>) -> u32 {
    let mut ops_per_sec = 0;

    let mut data = [0u8; 32];
    for (i, d) in data.iter_mut().enumerate() {
        *d = i as u8;
    }

    timer.start(1_000_000u32);
    while timer.wait().is_err() {
        black_box::<Vec<u8, U128>>(serde_json_core::to_vec(&data).unwrap());
        ops_per_sec += 1;
    }

    ops_per_sec
}

fn miscdata_vec_sjc(timer: &mut Timer<TIMER0>) -> u32 {
    let mut ops_per_sec = 0;

    let data = MiscData {
        a: 1023,
        b: 27,
        c: "Hi!",
        d: &[0, 10, 50, 20, 3],
        e: 0x0F00_0000_000F_A000,
        f: false,
    };


    timer.start(1_000_000u32);
    while timer.wait().is_err() {
        black_box::<Vec<u8, U128>>(serde_json_core::to_vec(&data).unwrap());
        ops_per_sec += 1;
    }

    ops_per_sec
}

fn u8_slice_32_slice_ssm(timer: &mut Timer<TIMER0>) -> u32 {
    let mut ops_per_sec = 0;

    let mut data = [0u8; 32];
    for (i, d) in data.iter_mut().enumerate() {
        *d = i as u8;
    }
    let mut buf = [0u8; 128];

    timer.start(1_000_000u32);
    while timer.wait().is_err() {
        black_box(ssmarshal::serialize(&mut buf, &data).unwrap());
        ops_per_sec += 1;
    }

    ops_per_sec
}

fn miscdata_slice_ssm(timer: &mut Timer<TIMER0>) -> u32 {
    let mut ops_per_sec = 0;

    let data = MiscData {
        a: 1023,
        b: 27,
        c: "Hi!",
        d: &[0, 10, 50, 20, 3],
        e: 0x0F00_0000_000F_A000,
        f: false,
    };

    let mut buf = [0u8; 128];

    timer.start(1_000_000u32);
    while timer.wait().is_err() {
        black_box(ssmarshal::serialize(&mut buf, &data).unwrap());
        ops_per_sec += 1;
    }

    ops_per_sec
}
