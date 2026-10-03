// Tests of commons_wire.hpp against the golden vectors the Rust and TypeScript decoders use, and
// the same header validation the Rust tests check. Run by commons-cpp's cargo test, which writes
// the vectors, one per line, into the file named by the first argument:
//   name hex kind flags tag_hex major minor epoch sequence correlation sim_time crc payload_hex
// (payload_hex is "-" when the payload is empty).

#include <cstdint>
#include <cstdio>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>

#include "commons_wire.hpp"

namespace cw = commons_wire;

static int failures = 0;

#define CHECK(condition)                                                                         \
    do {                                                                                         \
        if (!(condition)) {                                                                      \
            std::printf("FAILED %s:%d: %s\n", __FILE__, __LINE__, #condition);                   \
            ++failures;                                                                          \
        }                                                                                        \
    } while (0)

static std::vector<std::uint8_t> unhex(const std::string& text) {
    std::vector<std::uint8_t> out;
    if (text == "-") return out;
    for (std::size_t i = 0; i + 1 < text.size(); i += 2) {
        out.push_back(static_cast<std::uint8_t>(std::stoul(text.substr(i, 2), nullptr, 16)));
    }
    return out;
}

struct Vector {
    std::string name;
    std::vector<std::uint8_t> bytes;
    unsigned kind = 0;
    unsigned flags = 0;
    std::vector<std::uint8_t> tag;
    unsigned major = 0;
    unsigned minor = 0;
    std::uint32_t epoch = 0;
    std::uint64_t sequence = 0;
    std::uint64_t correlation = 0;
    std::int64_t sim_time = 0;
    std::uint32_t crc = 0;
    std::vector<std::uint8_t> payload;
};

static cw::Error decode_error(const std::vector<std::uint8_t>& bytes) {
    cw::Frame frame;
    return cw::decode(bytes.data(), bytes.size(), frame);
}

int main(int argc, char** argv) {
    if (argc != 2) {
        std::printf("usage: wire_test <vectors file>\n");
        return 2;
    }
    std::ifstream file(argv[1]);
    std::vector<Vector> vectors;
    std::string line;
    while (std::getline(file, line)) {
        if (line.empty()) continue;
        std::istringstream in(line);
        Vector v;
        std::string hex, tag, sequence, correlation, sim_time, payload;
        in >> v.name >> hex >> v.kind >> v.flags >> tag >> v.major >> v.minor >> v.epoch >>
            sequence >> correlation >> sim_time >> v.crc >> payload;
        v.bytes = unhex(hex);
        v.tag = unhex(tag);
        v.sequence = std::stoull(sequence);
        v.correlation = std::stoull(correlation);
        v.sim_time = std::stoll(sim_time);
        v.payload = unhex(payload);
        vectors.push_back(v);
    }
    CHECK(vectors.size() >= 3);

    for (const Vector& v : vectors) {
        cw::Frame frame;
        const cw::Error error = cw::decode(v.bytes.data(), v.bytes.size(), frame);
        CHECK(error == cw::Error::None);
        if (error != cw::Error::None) {
            std::printf("  %s: %s\n", v.name.c_str(), cw::error_name(error));
            continue;
        }
        const cw::FrameMeta& m = frame.header.meta;
        CHECK(static_cast<unsigned>(m.kind) == v.kind);
        CHECK(frame.header.flags == v.flags);
        CHECK(v.tag.size() == 4);
        for (std::size_t i = 0; i < 4 && i < v.tag.size(); ++i) CHECK(m.schema.tag[i] == v.tag[i]);
        CHECK(m.schema.major == v.major);
        CHECK(m.schema.minor == v.minor);
        CHECK(m.epoch == v.epoch);
        CHECK(m.sequence == v.sequence);
        CHECK(m.correlation == v.correlation);
        CHECK(m.sim_time == v.sim_time);
        CHECK(frame.header.payload_crc32 == v.crc);
        CHECK(frame.payload_len == v.payload.size());
        CHECK(std::vector<std::uint8_t>(frame.payload, frame.payload + frame.payload_len) ==
              v.payload);

        // The encoder reproduces the golden bytes.
        std::vector<std::uint8_t> encoded;
        const cw::Error encoding = cw::encode_into(m, v.payload.data(), v.payload.size(),
                                                   (v.flags & cw::kFlagPayloadCrc32) != 0, encoded);
        CHECK(encoding == cw::Error::None);
        CHECK(encoded == v.bytes);
    }

    // Each header field is validated, as in the Rust tests.
    const std::vector<std::uint8_t> good =
        vectors.size() > 1 ? vectors[1].bytes : std::vector<std::uint8_t>();
    CHECK(good.size() > cw::kHeaderLen);
    if (good.size() > cw::kHeaderLen) {
        auto bad = good;
        bad[0] = 'X';
        CHECK(decode_error(bad) == cw::Error::BadMagic);
        bad = good;
        bad[4] = 2;
        CHECK(decode_error(bad) == cw::Error::UnsupportedEnvelopeVersion);
        bad = good;
        bad[5] = 0;
        CHECK(decode_error(bad) == cw::Error::UnknownKind);
        bad = good;
        bad[6] = 0x03;
        CHECK(decode_error(bad) == cw::Error::UnknownFlags);
        bad = good;
        bad[52] = 1;
        CHECK(decode_error(bad) == cw::Error::ReservedNotZero);
        bad = good;
        bad[6] = 0;
        CHECK(decode_error(bad) == cw::Error::ChecksumWithoutFlag);
        bad = good;
        bad.back() = static_cast<std::uint8_t>(bad.back() ^ 0xFFu);
        CHECK(decode_error(bad) == cw::Error::ChecksumMismatch);
        bad = good;
        bad.push_back(0);
        CHECK(decode_error(bad) == cw::Error::LengthMismatch);
        bad = good;
        bad.pop_back();
        CHECK(decode_error(bad) == cw::Error::LengthMismatch);
        bad = good;
        for (std::size_t i = 32; i < 40; ++i) bad[i] = 0;
        CHECK(decode_error(bad) == cw::Error::MissingCorrelation);
        cw::FrameHeader header;
        CHECK(cw::decode_header(good.data(), cw::kHeaderLen - 1, header) == cw::Error::Truncated);
        cw::Frame frame;
        CHECK(cw::decode(good.data(), good.size(), frame, 4) == cw::Error::PayloadTooLarge);

        cw::FrameMeta command;
        command.kind = cw::FrameKind::Command;
        std::vector<std::uint8_t> out;
        CHECK(cw::encode_into(command, nullptr, 0, false, out) == cw::Error::MissingCorrelation);
        CHECK(out.empty());
    }

    // zlib's CRC-32 of "123456789".
    const std::uint8_t check[] = {'1', '2', '3', '4', '5', '6', '7', '8', '9'};
    CHECK(cw::crc32(check, sizeof check) == 0xCBF43926u);

    if (failures == 0) std::printf("wire_test: ok (%zu vectors)\n", vectors.size());
    return failures == 0 ? 0 : 1;
}
