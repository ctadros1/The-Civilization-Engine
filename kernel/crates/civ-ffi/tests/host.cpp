// A C++ host of the kernel library, as the Unreal plugin will be (ADR-0005), without Unreal: it
// loads the library at run time, reads frames with commons_wire.hpp and their payloads with the
// generated C++ readers, builds a command with the generated builders, and watches the world it
// asked for arrive. Usage:
//   host <library> <content dir> <saves dir>

#include <chrono>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <string>
#include <thread>
#include <vector>

#include "commons_wire.hpp"
#include "tce_kernel.h"
#include "tce_wire_generated.h"

namespace cw = commons_wire;
namespace wire = tce::wire;

typedef int32_t (*GetApi)(uint32_t abi_major, TceApiV1* out, size_t out_size);

#ifdef _WIN32
#include <windows.h>
static GetApi load(const char* path) {
    HMODULE library = LoadLibraryA(path);
    if (library == nullptr) return nullptr;
    return reinterpret_cast<GetApi>(
        reinterpret_cast<void (*)()>(GetProcAddress(library, "tce_get_api")));
}
#else
#include <dlfcn.h>
static GetApi load(const char* path) {
    void* library = dlopen(path, RTLD_NOW | RTLD_LOCAL);
    if (library == nullptr) return nullptr;
    void* symbol = dlsym(library, "tce_get_api");
    GetApi get_api = nullptr;
    std::memcpy(&get_api, &symbol, sizeof get_api);
    return get_api;
}
#endif

#define CHECK(condition, what)                                                                   \
    do {                                                                                         \
        if (!(condition)) {                                                                      \
            std::printf("host: FAILED %s (line %d)\n", what, __LINE__);                          \
            return 1;                                                                            \
        }                                                                                        \
    } while (0)

static TceBytes text(const std::string& s) {
    TceBytes bytes;
    bytes.ptr = reinterpret_cast<const uint8_t*>(s.data());
    bytes.len = s.size();
    return bytes;
}

/// Asks for a frame's size, then copies it, as a host polling once a frame does.
static bool poll(const TceApiV1& api, TceKernel* kernel, std::vector<uint8_t>& frame) {
    size_t len = 0;
    if (api.poll(kernel, nullptr, 0, &len) != TCE_BUFFER_TOO_SMALL) return false;
    frame.resize(len);
    return api.poll(kernel, frame.data(), frame.size(), &len) == TCE_OK;
}

/// Decodes a frame and verifies its payload as the table `T`.
template <typename T>
static const T* read(const std::vector<uint8_t>& bytes, cw::Frame& frame) {
    if (cw::decode(bytes.data(), bytes.size(), frame) != cw::Error::None) return nullptr;
    flatbuffers::Verifier verifier(frame.payload, frame.payload_len);
    if (!verifier.VerifyBuffer<T>(nullptr)) return nullptr;
    return flatbuffers::GetRoot<T>(frame.payload);
}

int main(int argc, char** argv) {
    if (argc != 4) {
        std::printf("usage: host <library> <content dir> <saves dir>\n");
        return 2;
    }
    GetApi get_api = load(argv[1]);
    CHECK(get_api != nullptr, "loading the library");
    TceApiV1 api;
    std::memset(&api, 0, sizeof api);
    CHECK(get_api(TCE_ABI_MAJOR, &api, sizeof api) == TCE_OK, "the table");
    TceVersions versions;
    std::memset(&versions, 0, sizeof versions);
    versions.struct_size = static_cast<uint32_t>(sizeof versions);
    CHECK(api.versions(&versions) == TCE_OK, "the versions");

    const std::string content = argv[2];
    const std::string saves = argv[3];
    TceConfig config;
    std::memset(&config, 0, sizeof config);
    config.struct_size = static_cast<uint32_t>(sizeof config);
    config.content_dir = text(content);
    config.saves_dir = text(saves);
    TceKernel* kernel = nullptr;
    CHECK(api.create(&config, &kernel) == TCE_OK, "creating a kernel");

    // The Welcome names the landscapes; take the default one.
    std::vector<uint8_t> bytes;
    cw::Frame frame;
    CHECK(poll(api, kernel, bytes), "polling the welcome");
    const wire::Welcome* welcome = read<wire::Welcome>(bytes, frame);
    CHECK(frame.header.meta.kind == cw::FrameKind::Welcome && welcome != nullptr, "the welcome");
    std::string preset;
    for (const wire::PresetInfo* p : *welcome->presets()) {
        if (p->is_default() && p->id() != nullptr) preset = p->id()->str();
    }
    CHECK(!preset.empty(), "a default landscape");

    // Ask for a world, as a command built with the generated builders.
    flatbuffers::FlatBufferBuilder fbb;
    const auto body = wire::CreateNewWorldDirect(fbb, 12, preset.c_str(), 256, "Plugin Valley", 32);
    fbb.Finish(wire::CreateCommand(fbb, wire::CommandBody::NewWorld, body.Union()));
    cw::FrameMeta meta;
    meta.kind = cw::FrameKind::Command;
    std::memcpy(meta.schema.tag, "TCE", 4);
    meta.schema.major = static_cast<uint16_t>(versions.wire_major);
    meta.schema.minor = static_cast<uint16_t>(versions.wire_minor);
    meta.correlation = 1;
    std::vector<uint8_t> command;
    CHECK(cw::encode_into(meta, fbb.GetBufferPointer(), fbb.GetSize(), false, command) ==
              cw::Error::None,
          "encoding the command");
    CHECK(api.submit(kernel, command.data(), command.size()) == TCE_OK, "submitting");

    // Poll for the reply and copy snapshots until the band has arrived.
    const auto start = std::chrono::steady_clock::now();
    bool answered = false;
    uint64_t number = 0;
    std::string world_name;
    size_t people = 0;
    std::vector<uint8_t> snapshot(1 << 20);
    while (world_name.empty() || people == 0) {
        CHECK(std::chrono::steady_clock::now() - start < std::chrono::seconds(120), "in time");
        while (poll(api, kernel, bytes)) {
            cw::Frame reply;
            if (cw::decode(bytes.data(), bytes.size(), reply) == cw::Error::None &&
                reply.header.meta.correlation == 1) {
                CHECK(reply.header.meta.kind == cw::FrameKind::Response, "an answer, not an error");
                answered = true;
            }
        }
        size_t len = 0;
        uint64_t got = 0;
        const int32_t code =
            api.copy_snapshot(kernel, number, snapshot.data(), snapshot.size(), &len, &got);
        if (code == TCE_OK) {
            number = got;
            const auto end = snapshot.begin() + static_cast<std::ptrdiff_t>(len);
            std::vector<uint8_t> copy(snapshot.begin(), end);
            const wire::Snapshot* s = read<wire::Snapshot>(copy, frame);
            CHECK(s != nullptr, "a snapshot");
            if (s->world() != nullptr && s->task() == nullptr && s->people() != nullptr &&
                s->world()->name() != nullptr) {
                world_name = s->world()->name()->str();
                people = s->people()->size();
            }
        } else {
            CHECK(code == TCE_NONE, "copying a snapshot");
            std::this_thread::sleep_for(std::chrono::milliseconds(10));
        }
    }
    CHECK(answered, "the command's answer");
    CHECK(world_name == "Plugin Valley", "the world's name");
    CHECK(api.destroy(kernel) == TCE_OK, "destroying the kernel");
    std::printf("host: ok, %s with %zu people, wire %u.%u\n", world_name.c_str(), people,
                versions.wire_major, versions.wire_minor);
    return 0;
}
