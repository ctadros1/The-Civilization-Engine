//! Envelope tests against independently built golden vectors, malformed input, and generated
//! frames. The golden file is shared with the TypeScript decoder in `web/`.

use commons_wire::{
    FrameKind, FrameMeta, HEADER_LEN, Limits, SchemaId, WireError, decode, decode_header, encode,
    split_first,
};
use proptest::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct GoldenFile {
    vectors: Vec<Vector>,
}

#[derive(Deserialize)]
struct Vector {
    name: String,
    hex: String,
    kind: u8,
    flags: u16,
    schema_tag_hex: String,
    major: u16,
    minor: u16,
    epoch: u32,
    sequence: String,
    correlation: String,
    sim_time: String,
    payload_crc32: u32,
    payload_hex: String,
}

fn unhex(text: &str) -> Vec<u8> {
    assert!(text.len().is_multiple_of(2), "odd-length hex");
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("valid hex"))
        .collect()
}

fn golden() -> Vec<Vector> {
    let text = include_str!("golden.json");
    serde_json::from_str::<GoldenFile>(text)
        .expect("golden.json parses")
        .vectors
}

fn vector_meta(v: &Vector) -> FrameMeta {
    let tag = unhex(&v.schema_tag_hex);
    FrameMeta {
        kind: FrameKind::from_u8(v.kind).expect("known kind"),
        schema: SchemaId::new([tag[0], tag[1], tag[2], tag[3]], v.major, v.minor),
        epoch: v.epoch,
        sequence: v.sequence.parse().expect("u64"),
        correlation: v.correlation.parse().expect("u64"),
        sim_time: v.sim_time.parse().expect("i64"),
    }
}

#[test]
fn golden_vectors_decode_to_their_fields() {
    for v in golden() {
        let bytes = unhex(&v.hex);
        let frame =
            decode(&bytes, &Limits::default()).unwrap_or_else(|e| panic!("{}: {e}", v.name));
        assert_eq!(frame.header.meta, vector_meta(&v), "{}", v.name);
        assert_eq!(frame.header.flags, v.flags, "{}", v.name);
        assert_eq!(frame.header.payload_crc32, v.payload_crc32, "{}", v.name);
        assert_eq!(
            frame.payload,
            unhex(&v.payload_hex).as_slice(),
            "{}",
            v.name
        );
    }
}

#[test]
fn encoder_reproduces_golden_bytes() {
    for v in golden() {
        let with_crc = v.flags & 1 != 0;
        let bytes = encode(&vector_meta(&v), &unhex(&v.payload_hex), with_crc)
            .unwrap_or_else(|e| panic!("{}: {e}", v.name));
        assert_eq!(bytes, unhex(&v.hex), "{}", v.name);
    }
}

#[test]
fn each_header_field_is_validated() {
    let good = unhex(&golden()[1].hex);
    let limits = Limits::default();

    let mut bad = good.clone();
    bad[0] = b'X';
    assert!(matches!(
        decode(&bad, &limits),
        Err(WireError::BadMagic { .. })
    ));

    let mut bad = good.clone();
    bad[4] = 2;
    assert!(matches!(
        decode(&bad, &limits),
        Err(WireError::UnsupportedEnvelopeVersion { found: 2, .. })
    ));

    let mut bad = good.clone();
    bad[5] = 0;
    assert_eq!(
        decode(&bad, &limits),
        Err(WireError::UnknownKind { found: 0 })
    );

    let mut bad = good.clone();
    bad[6] = 0x03;
    assert_eq!(
        decode(&bad, &limits),
        Err(WireError::UnknownFlags { found: 3 })
    );

    let mut bad = good.clone();
    bad[52] = 1;
    assert_eq!(
        decode(&bad, &limits),
        Err(WireError::ReservedNotZero { found: 1 })
    );

    let mut bad = good.clone();
    bad[6] = 0; // clear the checksum flag but keep the checksum value
    assert!(matches!(
        decode(&bad, &limits),
        Err(WireError::ChecksumWithoutFlag { .. })
    ));

    let mut bad = good.clone();
    let last = bad.len() - 1;
    bad[last] ^= 0xff; // corrupt the payload
    assert!(matches!(
        decode(&bad, &limits),
        Err(WireError::ChecksumMismatch { .. })
    ));

    let mut bad = good.clone();
    bad.push(0); // one trailing byte
    assert!(matches!(
        decode(&bad, &limits),
        Err(WireError::LengthMismatch { .. })
    ));

    let short = &good[..good.len() - 1];
    assert!(matches!(
        decode(short, &limits),
        Err(WireError::LengthMismatch { .. })
    ));

    assert!(matches!(
        decode_header(&good[..HEADER_LEN - 1], &limits),
        Err(WireError::Truncated { .. })
    ));
}

#[test]
fn concatenated_frames_split_in_order() {
    let vectors = golden();
    let mut stream = Vec::new();
    for v in &vectors {
        stream.extend_from_slice(&unhex(&v.hex));
    }
    let mut rest = stream.as_slice();
    for v in &vectors {
        let (frame, tail) = split_first(rest, &Limits::default()).expect("splits");
        assert_eq!(frame.header.meta, vector_meta(v));
        rest = tail;
    }
    assert!(rest.is_empty());
}

fn any_kind() -> impl Strategy<Value = FrameKind> {
    proptest::sample::select(FrameKind::ALL.to_vec())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn generated_frames_round_trip(
        kind in any_kind(),
        tag in any::<[u8; 4]>(),
        major in any::<u16>(),
        minor in any::<u16>(),
        epoch in any::<u32>(),
        sequence in any::<u64>(),
        correlation in 1u64..,
        sim_time in any::<i64>(),
        payload in proptest::collection::vec(any::<u8>(), 0..512),
        with_crc in any::<bool>(),
    ) {
        let meta = FrameMeta {
            kind,
            schema: SchemaId::new(tag, major, minor),
            epoch,
            sequence,
            correlation,
            sim_time,
        };
        let bytes = encode(&meta, &payload, with_crc).expect("encodes");
        let frame = decode(&bytes, &Limits::default()).expect("decodes");
        prop_assert_eq!(frame.header.meta, meta);
        prop_assert_eq!(frame.payload, payload.as_slice());
    }

    #[test]
    fn arbitrary_bytes_never_panic(bytes in proptest::collection::vec(any::<u8>(), 0..256)) {
        let _ = decode(&bytes, &Limits::default());
        let _ = split_first(&bytes, &Limits { max_payload_len: 64 });
    }

    #[test]
    fn single_byte_corruption_is_detected_or_harmless(
        index in 0usize..61,
        flip in 1u8..=255,
    ) {
        // A checksummed frame: any flipped byte must either be refused or decode to a frame
        // that differs only where the flip landed (header fields that carry no invariant).
        let meta = FrameMeta {
            kind: FrameKind::Response,
            schema: SchemaId::new(*b"TCE\0", 1, 0),
            epoch: 2,
            sequence: 5,
            correlation: 11,
            sim_time: 99,
        };
        let mut bytes = encode(&meta, b"hello", true).expect("encodes");
        bytes[index] ^= flip;
        if let Ok(frame) = decode(&bytes, &Limits::default()) {
            // Only free-valued header fields (the kind byte, which can land on another valid
            // kind, and offsets 8..48) can absorb a flip and still decode. Magic, version,
            // flags, checksum, reserved and payload bytes must always be refused.
            prop_assert!(index == 5 || (8..48).contains(&index), "flip at {} decoded", index);
            prop_assert_eq!(frame.payload, b"hello".as_slice());
        }
    }
}
