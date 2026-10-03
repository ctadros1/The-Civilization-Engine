// commons-wire's frame envelope in C++: the same 56-byte little-endian header that the Rust crate
// (commons/crates/commons-wire) and the web observer (web/src/wire/envelope.ts) read and write,
// for hosts written in C++ such as the Unreal plugin. ADR-0001 of The Civilization Engine
// describes the layout and the delivery contract of each kind; golden vectors in
// commons/crates/commons-wire/tests/golden.json pin it.
//
// Header-only, C++17, no dependencies. Decoding validates every header field, and the payload
// length against a limit, before anything sized by the frame is trusted.

#pragma once

#include <cstddef>
#include <cstdint>
#include <vector>

namespace commons_wire {

/// Magic bytes at the start of every frame.
inline constexpr std::uint8_t kMagic[4] = {'C', 'W', 'I', 'R'};
/// Layout version of the envelope itself.
inline constexpr std::uint8_t kEnvelopeVersion = 1;
/// Size of the fixed header; the payload starts here.
inline constexpr std::size_t kHeaderLen = 56;
/// Flags bit 0: the header carries a CRC-32 (IEEE) of the payload.
inline constexpr std::uint16_t kFlagPayloadCrc32 = 0x0001;
/// Default upper bound on a payload.
inline constexpr std::uint32_t kDefaultMaxPayloadLen = 64u * 1024u * 1024u;

/// What a frame is for. The values are part of the wire format.
enum class FrameKind : std::uint8_t {
    Hello = 1,
    Welcome = 2,
    Snapshot = 3,
    Delta = 4,
    Events = 5,
    Command = 6,
    Query = 7,
    Response = 8,
    Error = 9,
    Heartbeat = 10,
};

/// Whether a frame of this kind must carry a non-zero correlation id.
inline bool requires_correlation(FrameKind kind) {
    return kind == FrameKind::Command || kind == FrameKind::Query || kind == FrameKind::Response;
}

/// The engine schema a payload is written in: a four-byte tag and a version. Peers can exchange
/// frames when the tag and the major version match.
struct SchemaId {
    std::uint8_t tag[4] = {0, 0, 0, 0};
    std::uint16_t major = 0;
    std::uint16_t minor = 0;

    bool compatible_with(const SchemaId& other) const {
        return tag[0] == other.tag[0] && tag[1] == other.tag[1] && tag[2] == other.tag[2] &&
               tag[3] == other.tag[3] && major == other.major;
    }
};

/// The header fields a sender chooses.
struct FrameMeta {
    FrameKind kind = FrameKind::Heartbeat;
    SchemaId schema;
    std::uint32_t epoch = 0;
    std::uint64_t sequence = 0;
    std::uint64_t correlation = 0;
    std::int64_t sim_time = 0;
};

/// A decoded header.
struct FrameHeader {
    FrameMeta meta;
    std::uint16_t flags = 0;
    std::uint32_t payload_len = 0;
    std::uint32_t payload_crc32 = 0;
};

/// A decoded frame; `payload` points into the bytes that were decoded.
struct Frame {
    FrameHeader header;
    const std::uint8_t* payload = nullptr;
    std::size_t payload_len = 0;
};

/// Why bytes are not a frame, named as in the Rust crate.
enum class Error {
    None,
    Truncated,
    BadMagic,
    UnsupportedEnvelopeVersion,
    UnknownKind,
    UnknownFlags,
    ReservedNotZero,
    ChecksumWithoutFlag,
    PayloadTooLarge,
    LengthMismatch,
    ChecksumMismatch,
    MissingCorrelation,
};

/// A stable name for an error, for logs.
inline const char* error_name(Error error) {
    switch (error) {
    case Error::None: return "none";
    case Error::Truncated: return "truncated";
    case Error::BadMagic: return "bad magic";
    case Error::UnsupportedEnvelopeVersion: return "unsupported envelope version";
    case Error::UnknownKind: return "unknown kind";
    case Error::UnknownFlags: return "unknown flags";
    case Error::ReservedNotZero: return "reserved word not zero";
    case Error::ChecksumWithoutFlag: return "checksum without flag";
    case Error::PayloadTooLarge: return "payload too large";
    case Error::LengthMismatch: return "length mismatch";
    case Error::ChecksumMismatch: return "checksum mismatch";
    case Error::MissingCorrelation: return "missing correlation";
    }
    return "unknown";
}

/// CRC-32 (IEEE 802.3, reflected, as zlib computes it) of `len` bytes.
inline std::uint32_t crc32(const std::uint8_t* bytes, std::size_t len) {
    std::uint32_t crc = 0xFFFFFFFFu;
    for (std::size_t i = 0; i < len; ++i) {
        crc ^= bytes[i];
        for (int bit = 0; bit < 8; ++bit) {
            crc = (crc >> 1) ^ (0xEDB88320u & (0u - (crc & 1u)));
        }
    }
    return ~crc;
}

namespace detail {

inline std::uint16_t read_u16(const std::uint8_t* p) {
    return static_cast<std::uint16_t>(p[0] | (p[1] << 8));
}

inline std::uint32_t read_u32(const std::uint8_t* p) {
    return static_cast<std::uint32_t>(p[0]) | (static_cast<std::uint32_t>(p[1]) << 8) |
           (static_cast<std::uint32_t>(p[2]) << 16) | (static_cast<std::uint32_t>(p[3]) << 24);
}

inline std::uint64_t read_u64(const std::uint8_t* p) {
    return static_cast<std::uint64_t>(read_u32(p)) |
           (static_cast<std::uint64_t>(read_u32(p + 4)) << 32);
}

inline void put_u16(std::vector<std::uint8_t>& out, std::uint16_t v) {
    out.push_back(static_cast<std::uint8_t>(v));
    out.push_back(static_cast<std::uint8_t>(v >> 8));
}

inline void put_u32(std::vector<std::uint8_t>& out, std::uint32_t v) {
    for (int i = 0; i < 4; ++i) out.push_back(static_cast<std::uint8_t>(v >> (8 * i)));
}

inline void put_u64(std::vector<std::uint8_t>& out, std::uint64_t v) {
    for (int i = 0; i < 8; ++i) out.push_back(static_cast<std::uint8_t>(v >> (8 * i)));
}

} // namespace detail

/// Decodes and validates the header in the first `kHeaderLen` bytes of `bytes`, including the
/// payload length against `max_payload_len`, but not the payload itself.
inline Error decode_header(const std::uint8_t* bytes, std::size_t len, FrameHeader& out,
                           std::uint32_t max_payload_len = kDefaultMaxPayloadLen) {
    using namespace detail;
    if (len < kHeaderLen) return Error::Truncated;
    for (int i = 0; i < 4; ++i) {
        if (bytes[i] != kMagic[i]) return Error::BadMagic;
    }
    if (bytes[4] != kEnvelopeVersion) return Error::UnsupportedEnvelopeVersion;
    if (bytes[5] < 1 || bytes[5] > 10) return Error::UnknownKind;
    const auto kind = static_cast<FrameKind>(bytes[5]);
    const std::uint16_t flags = read_u16(bytes + 6);
    if ((flags & ~kFlagPayloadCrc32) != 0) return Error::UnknownFlags;
    FrameHeader header;
    header.meta.kind = kind;
    for (int i = 0; i < 4; ++i) header.meta.schema.tag[i] = bytes[8 + i];
    header.meta.schema.major = read_u16(bytes + 12);
    header.meta.schema.minor = read_u16(bytes + 14);
    header.meta.epoch = read_u32(bytes + 16);
    header.payload_len = read_u32(bytes + 20);
    header.meta.sequence = read_u64(bytes + 24);
    header.meta.correlation = read_u64(bytes + 32);
    header.meta.sim_time = static_cast<std::int64_t>(read_u64(bytes + 40));
    header.payload_crc32 = read_u32(bytes + 48);
    header.flags = flags;
    if (read_u32(bytes + 52) != 0) return Error::ReservedNotZero;
    if ((flags & kFlagPayloadCrc32) == 0 && header.payload_crc32 != 0) {
        return Error::ChecksumWithoutFlag;
    }
    if (header.payload_len > max_payload_len) return Error::PayloadTooLarge;
    if (requires_correlation(kind) && header.meta.correlation == 0) {
        return Error::MissingCorrelation;
    }
    out = header;
    return Error::None;
}

/// Decodes exactly one frame occupying all `len` bytes (one socket message, one FFI copy).
inline Error decode(const std::uint8_t* bytes, std::size_t len, Frame& out,
                    std::uint32_t max_payload_len = kDefaultMaxPayloadLen) {
    FrameHeader header;
    const Error error = decode_header(bytes, len, header, max_payload_len);
    if (error != Error::None) return error;
    const std::size_t frame_len = kHeaderLen + header.payload_len;
    if (len != frame_len) return Error::LengthMismatch;
    const std::uint8_t* payload = bytes + kHeaderLen;
    if ((header.flags & kFlagPayloadCrc32) != 0 &&
        crc32(payload, header.payload_len) != header.payload_crc32) {
        return Error::ChecksumMismatch;
    }
    out.header = header;
    out.payload = payload;
    out.payload_len = header.payload_len;
    return Error::None;
}

/// Appends one encoded frame to `out`. Refuses a command, query or response without a
/// correlation id, and a payload of 4 GiB or more.
inline Error encode_into(const FrameMeta& meta, const std::uint8_t* payload, std::size_t len,
                         bool with_crc, std::vector<std::uint8_t>& out) {
    using namespace detail;
    if (static_cast<std::uint64_t>(len) > 0xFFFFFFFFull) return Error::PayloadTooLarge;
    if (requires_correlation(meta.kind) && meta.correlation == 0) {
        return Error::MissingCorrelation;
    }
    out.reserve(out.size() + kHeaderLen + len);
    out.insert(out.end(), kMagic, kMagic + 4);
    out.push_back(kEnvelopeVersion);
    out.push_back(static_cast<std::uint8_t>(meta.kind));
    put_u16(out, with_crc ? kFlagPayloadCrc32 : 0);
    out.insert(out.end(), meta.schema.tag, meta.schema.tag + 4);
    put_u16(out, meta.schema.major);
    put_u16(out, meta.schema.minor);
    put_u32(out, meta.epoch);
    put_u32(out, static_cast<std::uint32_t>(len));
    put_u64(out, meta.sequence);
    put_u64(out, meta.correlation);
    put_u64(out, static_cast<std::uint64_t>(meta.sim_time));
    put_u32(out, with_crc ? crc32(payload, len) : 0);
    put_u32(out, 0);
    if (len > 0) out.insert(out.end(), payload, payload + len);
    return Error::None;
}

} // namespace commons_wire
