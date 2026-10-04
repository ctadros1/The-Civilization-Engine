/* The C harness (ADR-0005): loads the kernel library at run time, as the Unreal plugin will,
 * resolves only `tce_get_api`, and drives a kernel from C. It prints the layout of every
 * structure for tests/c_harness.rs to compare with Rust's. Usage:
 *   harness <library> <content dir> <saves dir> */

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "tce_kernel.h"

typedef int32_t (*GetApi)(uint32_t abi_major, TceApiV1 *out, size_t out_size);

#ifdef _WIN32
#include <windows.h>
typedef HMODULE Library;
static Library open_library(const char *path) { return LoadLibraryA(path); }
static GetApi find_get_api(Library library) {
    return (GetApi)(void (*)(void))GetProcAddress(library, "tce_get_api");
}
#else
#include <dlfcn.h>
typedef void *Library;
static Library open_library(const char *path) { return dlopen(path, RTLD_NOW | RTLD_LOCAL); }
static GetApi find_get_api(Library library) {
    /* POSIX's way of turning dlsym's object pointer into a function pointer. */
    void *symbol = dlsym(library, "tce_get_api");
    GetApi get_api;
    memcpy(&get_api, &symbol, sizeof get_api);
    return get_api;
}
#endif

/* Where a commons-wire frame keeps its kind (ADR-0001): after the 4-byte magic and the
 * envelope version. */
#define FRAME_KIND_AT 5
#define FRAME_HEADER_LEN 56
#define KIND_WELCOME 2
#define KIND_SNAPSHOT 3

#define CHECK(condition, ...)                                                                    \
    do {                                                                                         \
        if (!(condition)) {                                                                      \
            fprintf(stderr, "harness: " __VA_ARGS__);                                            \
            fprintf(stderr, "\n");                                                               \
            return 1;                                                                            \
        }                                                                                        \
    } while (0)

static TceBytes text(const char *s) {
    TceBytes bytes;
    bytes.ptr = (const uint8_t *)s;
    bytes.len = strlen(s);
    return bytes;
}

static void print_layout(void) {
    printf("layout TceBytes %zu %zu %zu\n", sizeof(TceBytes), offsetof(TceBytes, ptr),
           offsetof(TceBytes, len));
    printf("layout TceConfig %zu %zu %zu %zu %zu\n", sizeof(TceConfig),
           offsetof(TceConfig, struct_size), offsetof(TceConfig, flags),
           offsetof(TceConfig, content_dir), offsetof(TceConfig, saves_dir));
    printf("layout TcePanelConfig %zu %zu %zu %zu %zu\n", sizeof(TcePanelConfig),
           offsetof(TcePanelConfig, struct_size), offsetof(TcePanelConfig, port),
           offsetof(TcePanelConfig, reserved), offsetof(TcePanelConfig, web_dir));
    printf("layout TceVersions %zu %zu %zu %zu %zu %zu %zu %zu\n", sizeof(TceVersions),
           offsetof(TceVersions, struct_size), offsetof(TceVersions, abi_major),
           offsetof(TceVersions, abi_minor), offsetof(TceVersions, wire_major),
           offsetof(TceVersions, wire_minor), offsetof(TceVersions, save_schema),
           offsetof(TceVersions, content_api));
    printf("layout TceApiV1 %zu %zu %zu %zu %zu %zu %zu %zu %zu %zu %zu %zu %zu %zu\n",
           sizeof(TceApiV1), offsetof(TceApiV1, struct_size), offsetof(TceApiV1, abi_major),
           offsetof(TceApiV1, abi_minor), offsetof(TceApiV1, versions),
           offsetof(TceApiV1, create), offsetof(TceApiV1, destroy), offsetof(TceApiV1, status),
           offsetof(TceApiV1, last_error), offsetof(TceApiV1, submit), offsetof(TceApiV1, poll),
           offsetof(TceApiV1, copy_snapshot), offsetof(TceApiV1, serve_panels),
           offsetof(TceApiV1, panel_url));
}

int main(int argc, char **argv) {
    CHECK(argc == 4, "usage: harness <library> <content dir> <saves dir>");
    print_layout();

    Library library = open_library(argv[1]);
    CHECK(library != NULL, "could not load %s", argv[1]);
    GetApi get_api = find_get_api(library);
    CHECK(get_api != NULL, "the library has no tce_get_api");

    TceApiV1 api;
    memset(&api, 0, sizeof api);
    CHECK(get_api(TCE_ABI_MAJOR + 1, &api, sizeof api) == TCE_INCOMPATIBLE,
          "another major was accepted");
    CHECK(get_api(TCE_ABI_MAJOR, &api, sizeof api) == TCE_OK, "no table");
    CHECK(api.struct_size == sizeof api, "the table is %u bytes, not %zu", api.struct_size,
          sizeof api);

    TceVersions versions;
    memset(&versions, 0, sizeof versions);
    versions.struct_size = (uint32_t)sizeof versions;
    CHECK(api.versions(&versions) == TCE_OK, "no versions");
    printf("versions %u.%u wire %u.%u save %u content %u\n", versions.abi_major,
           versions.abi_minor, versions.wire_major, versions.wire_minor, versions.save_schema,
           versions.content_api);

    TceConfig config;
    memset(&config, 0, sizeof config);
    config.struct_size = (uint32_t)sizeof config;
    config.content_dir = text(argv[2]);
    config.saves_dir = text(argv[3]);
    TceKernel *kernel = NULL;
    int32_t code = api.create(&config, &kernel);
    CHECK(code == TCE_OK && kernel != NULL, "create returned %d", (int)code);
    CHECK(api.status(kernel) == TCE_OK, "the kernel is not running");

    /* The first ordered frame is the host's Welcome. */
    size_t len = 0;
    code = api.poll(kernel, NULL, 0, &len);
    CHECK(code == TCE_BUFFER_TOO_SMALL && len > FRAME_HEADER_LEN, "poll for size: %d, %zu",
          (int)code, len);
    uint8_t *frame = (uint8_t *)malloc(len);
    CHECK(frame != NULL, "out of memory");
    CHECK(api.poll(kernel, frame, len, &len) == TCE_OK, "poll");
    CHECK(memcmp(frame, "CWIR", 4) == 0, "the first frame is not a commons-wire frame");
    CHECK(frame[FRAME_KIND_AT] == KIND_WELCOME, "the first frame's kind is %u",
          (unsigned)frame[FRAME_KIND_AT]);
    free(frame);

    /* The latest snapshot, then nothing newer while nothing changes. */
    uint64_t number = 0;
    code = api.copy_snapshot(kernel, 0, NULL, 0, &len, &number);
    CHECK(code == TCE_BUFFER_TOO_SMALL, "copy_snapshot for size: %d", (int)code);
    frame = (uint8_t *)malloc(len);
    CHECK(frame != NULL, "out of memory");
    CHECK(api.copy_snapshot(kernel, 0, frame, len, &len, &number) == TCE_OK, "copy_snapshot");
    CHECK(frame[FRAME_KIND_AT] == KIND_SNAPSHOT, "the snapshot's kind is %u",
          (unsigned)frame[FRAME_KIND_AT]);
    CHECK(number >= 1, "snapshot number %llu", (unsigned long long)number);
    free(frame);

    /* Junk is refused, and the kernel says why. */
    const uint8_t junk[] = {1, 2, 3};
    CHECK(api.submit(kernel, junk, sizeof junk) == TCE_BAD_FRAME, "junk was accepted");
    char why[512];
    CHECK(api.last_error(kernel, (uint8_t *)why, sizeof why - 1, &len) == TCE_OK, "last_error");
    why[len] = '\0';
    printf("refused: %s\n", why);

    CHECK(api.destroy(kernel) == TCE_OK, "destroy");
    printf("harness: ok\n");
    return 0;
}
