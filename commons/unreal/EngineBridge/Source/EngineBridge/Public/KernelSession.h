#pragma once
#include "CoreMinimal.h"
#include "tce_kernel.h"
#include "commons_wire.hpp"
#include "tce_wire_generated.h"

// Owns the kernel and all copied bytes. Call on one host thread; shutdown joins Rust workers.
class ENGINEBRIDGE_API FKernelSession {
public:
    ~FKernelSession();
    bool Start();
    void Stop();
    bool Poll(TArray<uint8>& Bytes);
    bool Snapshot(TArray<uint8>& Bytes);
    bool Send(flatbuffers::FlatBufferBuilder& Builder, bool Query = false);
    FString PanelUrl;
    FString Error;
    uint32 Epoch = 0;
private:
    bool Check(int32 Code);
    TceApiV1 Api{};
    TceVersions Versions{};
    TceKernel* Kernel = nullptr;
    uint64 Number = 0;
    uint64 Correlation = 0;
};

template<typename T> const T* ReadKernelFrame(const TArray<uint8>& Bytes, commons_wire::Frame& Frame) {
    if (commons_wire::decode(Bytes.GetData(), Bytes.Num(), Frame) != commons_wire::Error::None) return nullptr;
    flatbuffers::Verifier Verifier(Frame.payload, Frame.payload_len);
    return Verifier.VerifyBuffer<T>(nullptr) ? flatbuffers::GetRoot<T>(Frame.payload) : nullptr;
}
