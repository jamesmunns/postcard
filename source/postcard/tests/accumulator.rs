use postcard::accumulator::{CobsAccumulator, FeedResult};
use serde::{Deserialize, Serialize};
use std::io::Read;

// Read a "huge" serialized struct in 32 byte chunks into a 256 byte buffer and deserialize it.
#[test]
fn reader() {
    let mut raw_buf = [0u8; 32];
    let mut input_buf = [0u8; 256];
    let mut cobs_buf: CobsAccumulator<256> = CobsAccumulator::new();

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct Huge {
        a: u32,
        b: [u32; 32],
        d: u64,
    }

    let expected = Huge {
        a: 0xabcdef00,
        b: [0x01234567; 32],
        d: u64::MAX,
    };

    let input = postcard::to_slice_cobs(&expected, &mut input_buf).unwrap();
    // TODO(https://github.com/rust-lang/rust-clippy/issues/12751): Remove once fixed.
    #[allow(clippy::redundant_slicing)]
    let mut input = &input[..];

    // Magic number from serializing struct and printing length
    assert_eq!(input.len(), 145);

    let mut output = None;

    while let Ok(ct) = input.read(&mut raw_buf) {
        // Finished reading input
        if ct == 0 {
            break;
        }

        let buf = &raw_buf[..ct];
        let mut window = buf;

        'cobs: while !window.is_empty() {
            window = match cobs_buf.feed::<Huge>(window) {
                FeedResult::Consumed => break 'cobs,
                FeedResult::OverFull(new_wind) => new_wind,
                FeedResult::DeserError(new_wind) => new_wind,
                FeedResult::Success { data, remaining } => {
                    output = Some(data);

                    remaining
                }
            };
        }
    }

    assert_eq!(output.unwrap(), expected);
}

#[test]
fn oversized_frame_is_discarded_until_its_delimiter() {
    let mut encoded = [0; 4];
    let tail = postcard::to_slice_cobs(&7u8, &mut encoded)
        .unwrap()
        .to_vec();
    let next = postcard::to_slice_cobs(&9u8, &mut encoded).unwrap();
    let mut input = vec![0x55; 64];
    input.extend_from_slice(&tail);
    input.extend_from_slice(next);

    // Use the public reader loop, including chunks that fill the buffer exactly,
    // split the overflow from its delimiter, or contain the following frame.
    for chunk_size in 1..=input.len() {
        let mut reader = input.as_slice();
        let mut read_buf = vec![0; chunk_size];
        let mut accumulator = CobsAccumulator::<64>::new();
        let mut values = Vec::new();
        let mut overflows = 0;
        let mut errors = 0;
        loop {
            let count = reader.read(&mut read_buf).unwrap();
            if count == 0 {
                break;
            }
            let mut window = &read_buf[..count];
            while !window.is_empty() {
                window = match accumulator.feed::<u8>(window) {
                    FeedResult::Consumed => break,
                    FeedResult::OverFull(rest) => {
                        overflows += 1;
                        rest
                    }
                    FeedResult::DeserError(rest) => {
                        errors += 1;
                        rest
                    }
                    FeedResult::Success { data, remaining } => {
                        values.push(data);
                        remaining
                    }
                };
            }
        }
        assert_eq!(values, [9], "chunk size {chunk_size}");
        assert_eq!(overflows, 1, "chunk size {chunk_size}");
        assert_eq!(errors, 0, "chunk size {chunk_size}");
    }
}

#[test]
fn overflow_recovery_preserves_borrowed_messages() {
    let mut accumulator = CobsAccumulator::<8>::new();
    assert!(matches!(
        accumulator.feed_ref::<&str>(&[0x55; 9]),
        FeedResult::OverFull(_)
    ));
    assert!(matches!(
        accumulator.feed_ref::<&str>(&[]),
        FeedResult::Consumed
    ));
    assert!(matches!(
        accumulator.feed_ref::<&str>(&[0x55; 24]),
        FeedResult::Consumed
    ));

    let mut raw_buf = [0; 16];
    let message = postcard::to_slice_cobs("next", &mut raw_buf).unwrap();
    let mut input = vec![0x55, 0];
    input.extend_from_slice(message);
    match accumulator.feed_ref::<&str>(&input) {
        FeedResult::Success { data, remaining } => {
            assert_eq!(data, "next");
            assert!(remaining.is_empty());
        }
        _ => panic!("expected the first complete frame after the discarded one"),
    }
}

#[test]
fn oversized_frame_ending_in_the_current_chunk_preserves_remaining_input() {
    let mut accumulator = CobsAccumulator::<8>::new();
    let mut input = vec![0x55; 8];
    input.push(0);
    let mut raw_buf = [0; 16];
    let message = postcard::to_slice_cobs("next", &mut raw_buf).unwrap();
    input.extend_from_slice(message);
    let rest = match accumulator.feed_ref::<&str>(&input) {
        FeedResult::OverFull(rest) => rest,
        _ => panic!("expected an oversized frame"),
    };
    assert_eq!(rest, message);
    match accumulator.feed_ref::<&str>(rest) {
        FeedResult::Success { data, remaining } => {
            assert_eq!(data, "next");
            assert!(remaining.is_empty());
        }
        _ => panic!("expected the next frame"),
    }
}

#[test]
fn frames_that_exactly_fill_the_accumulator_are_preserved() {
    let expected = [0x55u8; 6];
    let mut raw_buf = [0; 8];
    let message = postcard::to_slice_cobs(&expected, &mut raw_buf).unwrap();
    assert_eq!(message.len(), 8);
    for chunk_size in 1..=message.len() {
        let mut accumulator = CobsAccumulator::<8>::new();
        let mut output = None;
        for chunk in message.chunks(chunk_size) {
            match accumulator.feed::<[u8; 6]>(chunk) {
                FeedResult::Consumed => {}
                FeedResult::Success { data, remaining } => {
                    assert!(remaining.is_empty());
                    output = Some(data);
                }
                _ => panic!("a fitting frame must not overflow"),
            }
        }
        assert_eq!(output, Some(expected));
    }
}

#[test]
fn zero_capacity_discards_a_frame_without_repeated_overflow() {
    let mut accumulator = CobsAccumulator::<0>::new();
    assert!(matches!(
        accumulator.feed::<()>(&[1]),
        FeedResult::OverFull([])
    ));
    assert!(matches!(accumulator.feed::<()>(&[1]), FeedResult::Consumed));
    assert!(matches!(accumulator.feed::<()>(&[0]), FeedResult::Consumed));
    assert!(matches!(
        accumulator.feed::<()>(&[0]),
        FeedResult::OverFull([])
    ));
}
