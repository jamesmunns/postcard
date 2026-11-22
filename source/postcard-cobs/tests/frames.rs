//! Messages and streams of raw bytes, through the public API.

use std::fmt;

use postcard_cobs::{
    CobsAccumulator, DeserializeError, FeedResult, TakeFromBytesCobsError, max_frame_len,
    take_from_bytes_cobs, to_slice_cobs,
};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer, de::SeqAccess, de::Visitor,
    ser::SerializeTuple,
};

/// This helper type lets the tests send arbitrary bytes through the crate's
/// public API, which otherwise only takes T: Serialize/Deserialize things.
///
/// It essentially pretends to be an arbitrarily sized array (e.g. not length
/// prefixed), and collects whatever bytes are remaining.
#[derive(Debug, PartialEq)]
struct Raw(Vec<u8>);

impl Serialize for Raw {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut tuple = s.serialize_tuple(self.0.len())?;
        self.0.iter().try_for_each(|b| tuple.serialize_element(b))?;
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for Raw {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct RawVisitor;

        impl<'de> Visitor<'de> for RawVisitor {
            type Value = Raw;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("bytes")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Raw, A::Error> {
                let mut bytes = vec![];
                while let Ok(Some(byte)) = seq.next_element() {
                    bytes.push(byte);
                }
                Ok(Raw(bytes))
            }
        }

        d.deserialize_tuple(usize::MAX, RawVisitor)
    }
}

/// A message and its frame, each as parts to concatenate.
type Vector = (&'static [&'static [u8]], &'static [&'static [u8]]);

const R253: &[u8] = &[0xAA; 253];
const R254: &[u8] = &[0xAA; 254];

// These test vectors are a little goofy so we can splat together multiple
// fragments into a single array.
const VECTORS: &[Vector] = &[
    // Short messages, with zeros in each position
    (&[], &[&[0x01, 0x00]]),
    (&[&[0x00]], &[&[0x01, 0x01, 0x00]]),
    (&[&[0x00, 0x00]], &[&[0x01, 0x01, 0x01, 0x00]]),
    (&[&[0x11]], &[&[0x02, 0x11, 0x00]]),
    (&[&[0x11, 0x00]], &[&[0x02, 0x11, 0x01, 0x00]]),
    (&[&[0x00, 0x11]], &[&[0x01, 0x02, 0x11, 0x00]]),
    (
        &[&[0x11, 0x22, 0x00, 0x33]],
        &[&[0x03, 0x11, 0x22, 0x02, 0x33, 0x00]],
    ),
    (
        &[&[0x11, 0x00, 0x00, 0x22]],
        &[&[0x02, 0x11, 0x01, 0x02, 0x22, 0x00]],
    ),
    // Data that looks like code bytes
    (
        &[&[0xFF, 0x01, 0x00, 0xFE]],
        &[&[0x03, 0xFF, 0x01, 0x02, 0xFE, 0x00]],
    ),
    // A run one byte short of a full block
    (&[R253], &[&[0xFE], R253, &[0x00]]),
    (&[R253, &[0x00]], &[&[0xFE], R253, &[0x01, 0x00]]),
    (
        &[R253, &[0x00, 0x11]],
        &[&[0xFE], R253, &[0x02, 0x11, 0x00]],
    ),
    // A full block, which does not stand for a zero
    (&[R254], &[&[0xFF], R254, &[0x00]]),
    (&[R254, &[0x00]], &[&[0xFF], R254, &[0x01, 0x01, 0x00]]),
    (
        &[R254, &[0x00, 0x11]],
        &[&[0xFF], R254, &[0x01, 0x02, 0x11, 0x00]],
    ),
    (&[&[0x00], R254], &[&[0x01, 0xFF], R254, &[0x00]]),
    // Runs longer than a full block
    (&[R254, &[0x11]], &[&[0xFF], R254, &[0x02, 0x11, 0x00]]),
    (
        &[R254, &[0x11, 0x00]],
        &[&[0xFF], R254, &[0x02, 0x11, 0x01, 0x00]],
    ),
    (&[R254, R254], &[&[0xFF], R254, &[0xFF], R254, &[0x00]]),
    (
        &[R254, R254, &[0x11]],
        &[&[0xFF], R254, &[0xFF], R254, &[0x02, 0x11, 0x00]],
    ),
    // Full blocks on both sides of a zero
    (
        &[R254, &[0x00], R254],
        &[&[0xFF], R254, &[0x01, 0xFF], R254, &[0x00]],
    ),
];

/// Call `f` with each message and its frame.
fn for_each_vector(mut f: impl FnMut(&[u8], &[u8])) {
    for (message, frame) in VECTORS {
        f(&message.concat(), &frame.concat());
    }
}

#[test]
fn encode() {
    for_each_vector(|message, frame| {
        assert!(frame.len() <= max_frame_len(message.len()));
        let message = Raw(message.to_vec());
        for cap in 0..=frame.len() + 1 {
            let mut buf = vec![0xEE; cap + 1];
            let got = to_slice_cobs(&message, &mut buf[..cap]);
            if cap < frame.len() {
                assert!(got.is_err(), "{cap} {frame:02x?}");
            } else {
                assert_eq!(got.unwrap(), frame, "{cap}");
            }
            assert!(buf[cap.min(frame.len())..].iter().all(|b| *b == 0xEE));
        }
    });
}

/// Attempt to decode the first frame
fn decode(stream: &[u8]) -> (Result<Vec<u8>, DeserializeError>, usize) {
    let mut buf = stream.to_vec();
    let (result, remaining) = match take_from_bytes_cobs::<Raw>(&mut buf) {
        Ok((message, remaining)) => (Ok(message.0), remaining),
        Err(TakeFromBytesCobsError { error, remaining }) => (Err(error), remaining),
    };
    let len = remaining.len();
    assert!(stream.ends_with(remaining), "{stream:02x?}");
    (result, len)
}

#[test]
fn decode_messages() {
    for_each_vector(|message, frame| {
        // Inside a frame sandwich
        let stream = [&[0, 0], frame, &[7, 7]].concat();
        assert_eq!(decode(&stream), (Ok(message.to_vec()), 2));

        // Iterate over every way to cut off the frame
        for len in 1..frame.len() {
            let stream = &stream[..2 + len];
            assert_eq!(
                decode(stream),
                (Err(DeserializeError::Truncated), stream.len()),
                "{stream:02x?}",
            );
        }
    });
}

#[test]
fn decode_no_frame() {
    for stream in [&[][..], &[0], &[0, 0, 0]] {
        assert_eq!(decode(stream), (Err(DeserializeError::NoFrame), 0));
    }
}

#[test]
fn decode_invalid() {
    // [(input, remaining)]
    let cases: &[(&[u8], usize)] = &[
        // A block with no data, or too little
        (&[2, 0][..], 0),
        (&[3, 0x11, 0], 0),
        (&[0xFE, 0x11, 0], 0),
        (&[0xFF, 0x11, 0], 0),
        // A later block
        (&[2, 0x11, 3, 0x22, 0], 0),
        (&[1, 2, 0], 0),
        // A block that would end inside the next frame
        (&[3, 0x11, 0, 2, 0x22, 0], 3),
        // After extra zeros
        (&[0, 0, 3, 0x11, 0, 7], 1),
    ];

    for (stream, remaining) in cases {
        assert_eq!(
            decode(stream),
            (Err(DeserializeError::InvalidFrame), *remaining),
            "{stream:02x?}"
        );
    }

    // A full block with one byte missing
    let mut stream = vec![];
    stream.push(0xFF);
    stream.extend_from_slice(&[0xAA; 253]);
    stream.push(0);

    assert_eq!(decode(&stream), (Err(DeserializeError::InvalidFrame), 0));
}

/// postcard 1.x ends a message that ends on a full run with an extra empty block.
#[test]
fn decode_extra_empty_block() {
    let mut stream = vec![];
    stream.push(0xFF);
    stream.extend_from_slice(&[0xAA; 254]);
    stream.extend_from_slice(&[0x01, 0x00]);
    assert_eq!(decode(&stream), (Ok(vec![0xAA; 254]), 0));
}

/// Same postcard 1.x check with the accumulator
#[test]
fn accumulate_extra_empty_block() {
    const N: usize = max_frame_len(254);
    let mut stream = vec![];
    stream.push(0xFF);
    stream.extend_from_slice(&[0xAA; 254]);
    stream.extend_from_slice(&[0x01, 0x00]);
    assert_eq!(accumulate::<N>(&stream, 100), [Some(Ok(vec![0xAA; 254]))]);
}

/// Handy test tool type alias.
///
/// None: if the frame doesn't fit
/// Some: The result from feed/decode
type Item = Option<Result<Vec<u8>, DeserializeError>>;

/// Decode all of `stream` with [`take_from_bytes_cobs`].
fn walk(mut stream: &[u8], cap: usize) -> Vec<Item> {
    let mut items = vec![];
    loop {
        let (res, len) = decode(stream);
        if let Err(DeserializeError::NoFrame | DeserializeError::Truncated) = res {
            return items;
        }

        let zeros = stream.iter().take_while(|b| **b == 0).count();
        let frame_len = stream.len() - len - zeros - 1;
        items.push((frame_len <= cap).then_some(res));
        stream = &stream[stream.len() - len..];
    }
}

/// Decode all of `stream` with a [`CobsAccumulator`], in chunks of `chunk` bytes.
fn accumulate<const N: usize>(stream: &[u8], chunk: usize) -> Vec<Item> {
    let mut acc = CobsAccumulator::<N>::new();
    let mut items = vec![];
    for mut input in stream.chunks(chunk) {
        while !input.is_empty() {
            let (item, remaining) = match acc.feed::<Raw>(input) {
                FeedResult::Consumed => break,
                FeedResult::OverFull(remaining) => (None, remaining),
                FeedResult::DeserError { error, remaining } => (Some(Err(error)), remaining),
                FeedResult::Success { data, remaining } => (Some(Ok(data.0)), remaining),
            };
            items.push(item);
            input = remaining;
        }
    }
    items
}

/// Feeding a stream in chunks of any one size gives what [`take_from_bytes_cobs`] does.
fn check<const N: usize>(stream: &[u8]) {
    let want = walk(stream, N);
    for chunk in 1..=stream.len().max(1) {
        let got = accumulate::<N>(stream, chunk);
        assert_eq!(got, want, "{stream:02x?} {N} {chunk}");
    }
}

#[test]
fn accumulate_messages() {
    for_each_vector(|_, frame| {
        let mut stream = vec![];
        stream.push(0);
        stream.extend_from_slice(frame);
        stream.extend_from_slice(&[0, 0, 2, 42, 0]);

        check::<253>(&stream);
        check::<254>(&stream);
        check::<255>(&stream);
        check::<512>(&stream);
    });
}

#[test]
fn accumulate_edge_cases() {
    let cases: &[&[u8]] = &[
        // No frames
        &[],
        // Oops all sentinels
        &[0, 0, 0],
        // Several messages, empty messages, and extra zeros
        &[2, 0x11, 0, 0, 0, 1, 0, 1, 0, 3, 0x22, 0x33, 1, 0],
        // Malformed, then good
        &[3, 0x11, 0, 2, 0x22, 0],
        &[2, 0, 2, 0x22, 0],
        &[0xFF, 0x11, 0, 0, 2, 0x22, 0],
        &[2, 0x11, 3, 0x22, 0, 1, 0],
        // Malformed, twice in a row
        &[2, 0, 5, 0x11, 0, 2, 0x22, 0],
        // Too long for the smaller buffers, then one that fits
        &[4, 0x11, 0x22, 0x33, 0, 1, 0],
        &[1, 1, 1, 1, 0, 1, 0],
        &[2, 0x11, 4, 0x22, 0x33, 0x44, 0, 0, 1, 0],
        // The tail of a frame that doesn't fit is not decoded as a message, see #311
        &[9, 1, 1, 1, 1, 1, 1, 2, 0x11, 0, 2, 0x22, 0],
        // Too long for the smaller buffers, and malformed
        &[4, 0x11, 0x22, 5, 0x33, 0, 1, 0],
        // A partial frame at the end
        &[2, 0x11, 0, 3, 0x22],
        &[2, 0x11, 0, 4, 0x22, 0x33, 0x44],
        &[2, 0x11, 0, 1],
    ];

    for stream in cases {
        check::<0>(stream);
        check::<1>(stream);
        check::<2>(stream);
        check::<8>(stream);
    }
}
