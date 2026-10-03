//! Container behaviour: round trips, refusal of every kind of damage, and crash-safe publication.

use std::io::Cursor;

use commons_persist::{
    Codec, Compression, FOOTER_LEN, HEADER_LEN, Limits, MANIFEST_ENTRY_LEN, MANIFEST_PREFIX_LEN,
    PersistError, SaveDir, SaveKind, SectionData, SectionTag, SnapshotInfo, SnapshotReader,
    SnapshotWriter, publish_file,
};
use proptest::prelude::*;

const TERRAIN: SectionTag = SectionTag::new("terrain");
const CLOCK: SectionTag = SectionTag::new("clock");
const NOISE: SectionTag = SectionTag::new("noise");

fn info() -> SnapshotInfo {
    let mut info = SnapshotInfo::new(*b"TESTENG\0", 2, [9; 16]);
    info.snapshot_id = [1; 16];
    info.parent_id = Some([2; 16]);
    info.generation = 4;
    info.created_unix_ms = 1_700_000_000_000;
    info.sim_time = 525_600;
    info.content_fingerprint = [5; 32];
    info.label = "test".into();
    info
}

/// Pseudo-random bytes that zstd cannot shrink.
fn noise(len: usize, mut state: u64) -> Vec<u8> {
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 24) as u8
        })
        .collect()
}

fn sections() -> Vec<SectionData> {
    vec![
        SectionData::new(TERRAIN, 0, 1, vec![7u8; 4096], Compression::Zstd),
        SectionData::new(
            TERRAIN,
            1,
            1,
            (0..=255u8).cycle().take(3000).collect(),
            Compression::Zstd,
        ),
        SectionData::new(CLOCK, 0, 3, 42u64.to_le_bytes().to_vec(), Compression::Raw),
        SectionData::new(
            NOISE,
            0,
            1,
            noise(2048, 0x9e37_79b9_7f4a_7c15),
            Compression::Zstd,
        ),
    ]
}

fn build(info: &SnapshotInfo, sections: &[SectionData]) -> Vec<u8> {
    let mut writer = SnapshotWriter::new(Vec::new(), info).expect("header");
    for s in sections {
        writer
            .add_chunk(s.tag, s.index, s.version, &s.bytes, s.compression)
            .expect("chunk");
    }
    writer.finish().expect("finish").out
}

fn open(bytes: Vec<u8>) -> Result<SnapshotReader<Cursor<Vec<u8>>>, PersistError> {
    SnapshotReader::new(Cursor::new(bytes), Limits::default())
}

#[test]
fn round_trip_preserves_info_and_every_section() {
    let secs = sections();
    let mut reader = open(build(&info(), &secs)).expect("opens");
    reader.verify_all().expect("verifies");
    assert_eq!(reader.info(), &info());

    let terrain = reader.require_section(TERRAIN).expect("terrain");
    assert_eq!(terrain.len(), 2);
    assert_eq!(terrain[0].bytes, secs[0].bytes);
    assert_eq!(terrain[1].bytes, secs[1].bytes);
    let clock = reader.require_section(CLOCK).expect("clock");
    assert_eq!(clock[0].version, 3);
    assert_eq!(clock[0].bytes, 42u64.to_le_bytes());
    assert_eq!(
        reader.read_section(NOISE).expect("noise")[0].bytes,
        secs[3].bytes
    );
    assert!(
        reader
            .read_section(SectionTag::new("absent"))
            .expect("ok")
            .is_empty()
    );
    assert!(matches!(
        reader.require_section(SectionTag::new("absent")),
        Err(PersistError::MissingSection { .. })
    ));
}

#[test]
fn compressible_chunks_shrink_and_incompressible_ones_are_stored_raw() {
    let reader = open(build(&info(), &sections())).expect("opens");
    let by_tag = |tag, index| {
        reader
            .chunks()
            .iter()
            .find(|c| c.tag == tag && c.index == index)
            .cloned()
            .expect("present")
    };
    let flat = by_tag(TERRAIN, 0);
    assert_eq!(flat.codec, Codec::Zstd);
    assert!(flat.stored_len < flat.raw_len / 10);
    let random = by_tag(NOISE, 0);
    assert_eq!(random.codec, Codec::Raw, "zstd output was not smaller");
    assert_eq!(random.stored_len, random.raw_len);
}

#[test]
fn duplicate_chunks_are_refused_by_the_writer() {
    let mut writer = SnapshotWriter::new(Vec::new(), &info()).expect("header");
    writer
        .add_chunk(CLOCK, 0, 1, b"a", Compression::Raw)
        .expect("first");
    assert!(matches!(
        writer.add_chunk(CLOCK, 0, 1, b"b", Compression::Raw),
        Err(PersistError::DuplicateChunk { .. })
    ));
}

#[test]
fn gaps_in_a_section_are_refused() {
    let secs = vec![
        SectionData::new(TERRAIN, 0, 1, vec![1], Compression::Raw),
        SectionData::new(TERRAIN, 2, 1, vec![3], Compression::Raw),
    ];
    let mut reader = open(build(&info(), &secs)).expect("opens");
    assert!(matches!(
        reader.read_section(TERRAIN),
        Err(PersistError::MissingChunk { index: 1, .. })
    ));
}

#[test]
fn damage_in_each_region_is_reported_precisely() {
    let good = build(&info(), &sections());
    let manifest_offset = good.len() - FOOTER_LEN - (MANIFEST_PREFIX_LEN + 4 * MANIFEST_ENTRY_LEN);

    // Header: refused at open.
    let mut bad = good.clone();
    bad[100] ^= 0x40;
    assert!(matches!(open(bad), Err(PersistError::HeaderCorrupt { .. })));

    // Chunk body: open succeeds (bodies are read lazily), the chunk read fails, and so does the
    // body digest.
    let mut bad = good.clone();
    bad[HEADER_LEN + 3] ^= 0x01;
    let mut reader = open(bad).expect("structure is intact");
    assert!(matches!(
        reader.read_section(TERRAIN),
        Err(PersistError::ChunkCorrupt {
            stage: "stored",
            ..
        })
    ));
    assert!(matches!(
        reader.verify_body(),
        Err(PersistError::BodyCorrupt { .. })
    ));

    // Manifest: refused at open.
    let mut bad = good.clone();
    bad[manifest_offset + MANIFEST_PREFIX_LEN + 30] ^= 0x01;
    assert!(matches!(
        open(bad),
        Err(PersistError::ManifestCorrupt { .. })
    ));

    // Footer magic: the save looks interrupted.
    let mut bad = good.clone();
    let last = bad.len() - 1;
    bad[last] ^= 0x01;
    assert!(matches!(open(bad), Err(PersistError::Incomplete { .. })));

    // Not a snapshot at all.
    assert!(matches!(
        open(b"hello world, this is not a snapshot".repeat(20)),
        Err(PersistError::NotASnapshot)
    ));
}

#[test]
fn every_truncation_is_refused_without_panicking() {
    let good = build(&info(), &sections());
    for len in 0..good.len() {
        let result = open(good[..len].to_vec()).and_then(|mut r| r.verify_all());
        assert!(result.is_err(), "truncated to {len} bytes was accepted");
    }
}

#[test]
fn a_declared_raw_length_caps_decompression() {
    // Rewrite the manifest so a compressed chunk claims to decode to fewer bytes than it does,
    // then refresh the manifest and body digests so only the size lie remains.
    let good = build(&info(), &sections());
    let manifest_len = MANIFEST_PREFIX_LEN + 4 * MANIFEST_ENTRY_LEN;
    let manifest_offset = good.len() - FOOTER_LEN - manifest_len;
    let mut bad = good.clone();
    let entry = manifest_offset + MANIFEST_PREFIX_LEN; // first chunk: 4096 bytes of 7s, zstd
    bad[entry + 40..entry + 48].copy_from_slice(&100u64.to_le_bytes());
    let manifest_digest = xxhash_rust_digest(&bad[manifest_offset..manifest_offset + manifest_len]);
    let body_digest = xxhash_rust_digest(&bad[..manifest_offset + manifest_len]);
    let footer = bad.len() - FOOTER_LEN;
    bad[footer + 24..footer + 32].copy_from_slice(&manifest_digest.to_le_bytes());
    bad[footer + 32..footer + 40].copy_from_slice(&body_digest.to_le_bytes());

    let mut reader = open(bad).expect("structure is consistent");
    reader.verify_body().expect("digests were refreshed");
    assert!(matches!(
        reader.read_section(TERRAIN),
        Err(PersistError::Decompress { .. })
    ));
}

/// The container's documented digest: xxh3-64. Recomputed here independently of the crate's
/// internals, through the same public algorithm.
fn xxhash_rust_digest(bytes: &[u8]) -> u64 {
    xxhash_rust::xxh3::xxh3_64(bytes)
}

#[test]
fn limits_are_enforced_before_reading_bodies() {
    let good = build(&info(), &sections());
    let tight = Limits {
        max_chunks: 3,
        ..Limits::default()
    };
    assert!(matches!(
        SnapshotReader::new(Cursor::new(good.clone()), tight),
        Err(PersistError::TooManyChunks { count: 4, max: 3 })
    ));
    let tight = Limits {
        max_chunk_raw_len: 1000,
        ..Limits::default()
    };
    assert!(matches!(
        SnapshotReader::new(Cursor::new(good), tight),
        Err(PersistError::ChunkTooLarge { .. })
    ));
}

#[test]
fn publication_never_overwrites_and_leaves_no_temp_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let published =
        publish_file(dir.path(), "a.save", &info(), &sections(), 1).expect("first publish");
    let original = std::fs::read(&published.path).expect("read back");
    assert_eq!(original.len() as u64, published.file_len);

    let mut other = info();
    other.label = "different".into();
    assert!(matches!(
        publish_file(dir.path(), "a.save", &other, &sections(), 1),
        Err(PersistError::AlreadyExists { .. })
    ));
    assert_eq!(
        std::fs::read(&published.path).expect("read again"),
        original
    );

    let save_dir = SaveDir::create(dir.path(), "save").expect("save dir");
    assert!(save_dir.stray_temp_files().expect("scan").is_empty());
    let mut reader = SnapshotReader::open_file(&published.path, Limits::default()).expect("opens");
    reader.verify_all().expect("valid");
}

#[test]
fn save_dirs_number_generations_and_prune_only_autosaves() {
    let dir = tempfile::tempdir().expect("tempdir");
    let saves = SaveDir::create(dir.path().join("world"), "save").expect("save dir");
    let mut info = info();
    let manual = saves
        .publish(SaveKind::Manual, &mut info, &sections())
        .expect("manual");
    assert_eq!(info.generation, 1);
    for _ in 0..4 {
        saves
            .publish(SaveKind::Autosave, &mut info, &sections())
            .expect("auto");
    }
    saves
        .publish(SaveKind::Crash, &mut info, &sections())
        .expect("crash");
    assert_eq!(info.generation, 6);

    // A stray temp file and an unrelated file are ignored by the scan.
    std::fs::write(saves.path().join(".tmp-abc.part"), b"partial").expect("write temp");
    std::fs::write(saves.path().join("notes.txt"), b"hello").expect("write other");

    let removed = saves.prune(SaveKind::Autosave, 2).expect("prune");
    assert_eq!(removed.len(), 2);
    let kinds: Vec<(u64, SaveKind)> = saves
        .scan()
        .expect("scan")
        .into_iter()
        .map(|g| (g.generation, g.kind))
        .collect();
    assert_eq!(
        kinds,
        vec![
            (1, SaveKind::Manual),
            (4, SaveKind::Autosave),
            (5, SaveKind::Autosave),
            (6, SaveKind::Crash),
        ]
    );
    assert!(manual.path.exists());
    assert_eq!(saves.stray_temp_files().expect("temps").len(), 1);

    // The generation in the header matches the file name.
    let reader = SnapshotReader::open_file(&manual.path, Limits::default()).expect("opens");
    assert_eq!(reader.info().generation, 1);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(192))]

    /// Every byte of a snapshot is covered by some digest or structural check: a single flipped
    /// byte anywhere must be refused by open + full verification.
    #[test]
    fn any_single_byte_flip_is_refused(position in any::<prop::sample::Index>(), flip in 1u8..=255) {
        let mut bytes = build(&info(), &sections());
        let at = position.index(bytes.len());
        bytes[at] ^= flip;
        let result = open(bytes).and_then(|mut r| r.verify_all());
        prop_assert!(result.is_err(), "flip at byte {} was accepted", at);
    }

    /// Arbitrary bytes never panic the reader.
    #[test]
    fn arbitrary_bytes_never_panic(bytes in proptest::collection::vec(any::<u8>(), 0..600)) {
        let _ = open(bytes).and_then(|mut r| r.verify_all());
    }
}
