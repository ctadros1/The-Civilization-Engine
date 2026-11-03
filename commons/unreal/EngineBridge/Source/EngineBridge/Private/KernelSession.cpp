#include "KernelSession.h"
#include "HAL/PlatformProcess.h"
#include "Misc/Paths.h"
#include "Modules/ModuleManager.h"
#include "Misc/CommandLine.h"
#include "Misc/Parse.h"
IMPLEMENT_MODULE(FDefaultModuleImpl, EngineBridge)

FKernelSession::~FKernelSession() { Stop(); }
void FKernelSession::Stop() { if (Kernel) { Api.destroy(Kernel); Kernel = nullptr; } Number = 0; }
bool FKernelSession::Check(int32 Code) {
    if (Code >= 0) return true;
    uint8 Buffer[4096]{}; size_t Len = 0;
    if (Kernel && Api.last_error(Kernel, Buffer, sizeof(Buffer)-1, &Len) == TCE_OK)
        Error = FString(UTF8_TO_TCHAR(reinterpret_cast<const char*>(Buffer)));
    else Error = FString::Printf(TEXT("Kernel error %d"), Code);
    UE_LOG(LogTemp, Error, TEXT("%s"), *Error);
    return false;
}
bool FKernelSession::Start() {
    // Keep this handle loaded for the process lifetime, including repeated PIE sessions.
    static void* Library = FPlatformProcess::GetDllHandle(*FPaths::ConvertRelativePathToFull(FPaths::Combine(FPaths::ProjectDir(), TEXT("Binaries/Win64/tce_kernel.dll"))));
    if (!Library) { Error = TEXT("tce_kernel.dll is missing. Run tools/build-unreal.ps1."); return false; }
    using FGetApi = int32(*)(uint32, TceApiV1*, size_t);
    auto GetApi = reinterpret_cast<FGetApi>(FPlatformProcess::GetDllExport(Library, TEXT("tce_get_api")));
    if (!GetApi || GetApi(TCE_ABI_MAJOR, &Api, sizeof(Api)) != TCE_OK || Api.struct_size < sizeof(Api) || Api.abi_major != TCE_ABI_MAJOR) {
        Error = TEXT("Incompatible kernel ABI."); return false;
    }
    Versions.struct_size = sizeof(Versions);
    if (!Check(Api.versions(&Versions))) return false;
    if (Versions.wire_major != 1 || Versions.wire_minor < 107) {
        Error = TEXT("This client needs kernel wire 1.107 or later."); return false;
    }
    FString Data = FPaths::ConvertRelativePathToFull(FPaths::Combine(FPaths::ProjectDir(), TEXT("Data")));
    if (!FPaths::DirectoryExists(FPaths::Combine(Data, TEXT("content"))))
        Data = FPaths::ConvertRelativePathToFull(FPaths::Combine(FPaths::ProjectDir(), TEXT("..")));
    FTCHARToUTF8 Content(*FPaths::Combine(Data, TEXT("content")));
    FString SaveFolder=FPaths::ConvertRelativePathToFull(FPaths::Combine(FPaths::ProjectSavedDir(),TEXT("Worlds")));
    FParse::Value(FCommandLine::Get(),TEXT("TCESaves="),SaveFolder);
    FTCHARToUTF8 Saves(*SaveFolder);
    TceConfig Config{}; Config.struct_size = sizeof(Config);
    Config.content_dir = {reinterpret_cast<const uint8*>(Content.Get()), static_cast<size_t>(Content.Length())};
    Config.saves_dir = {reinterpret_cast<const uint8*>(Saves.Get()), static_cast<size_t>(Saves.Length())};
    if (!Check(Api.create(&Config, &Kernel))) return false;
    FString WebPath = FPaths::Combine(Data, TEXT("web"));
    if (!FPaths::FileExists(FPaths::Combine(WebPath, TEXT("index.html")))) WebPath /= TEXT("dist");
    FTCHARToUTF8 Web(*WebPath);
    TcePanelConfig Panels{}; Panels.struct_size = sizeof(Panels);
    Panels.web_dir = {reinterpret_cast<const uint8*>(Web.Get()), static_cast<size_t>(Web.Length())};
    if (!Check(Api.serve_panels(Kernel, &Panels))) return false;
    uint8 Url[2048]{}; size_t Len = 0;
    if (!Check(Api.panel_url(Kernel, Url, sizeof(Url)-1, &Len))) return false;
    PanelUrl = UTF8_TO_TCHAR(reinterpret_cast<const char*>(Url));
    PanelUrl += PanelUrl.Contains(TEXT("?")) ? TEXT("&view=panels") : TEXT("?view=panels");
    UE_LOG(LogTemp, Display, TEXT("EngineBridge ready: ABI %u.%u, wire %u.%u"), Versions.abi_major, Versions.abi_minor, Versions.wire_major, Versions.wire_minor);
    return true;
}
bool FKernelSession::Poll(TArray<uint8>& Bytes) {
    if (!Kernel) return false;
    size_t Len = 0;
    int32 Code = Api.poll(Kernel, nullptr, 0, &Len);
    if (Code == TCE_NONE) return false;
    if (Code != TCE_BUFFER_TOO_SMALL || Len > 64*1024*1024) { Check(Code); return false; }
    Bytes.SetNumUninitialized(static_cast<int32>(Len));
    return Check(Api.poll(Kernel, Bytes.GetData(), Bytes.Num(), &Len));
}
bool FKernelSession::Snapshot(TArray<uint8>& Bytes) {
    if (!Kernel) return false;
    if (Bytes.Num() == 0) Bytes.SetNumUninitialized(1024*1024);
    size_t Len = 0; uint64 Got = 0;
    int32 Code = Api.copy_snapshot(Kernel, Number, Bytes.GetData(), Bytes.Num(), &Len, &Got);
    if (Code == TCE_BUFFER_TOO_SMALL && Len <= 64*1024*1024) {
        Bytes.SetNumUninitialized(static_cast<int32>(Len));
        Code = Api.copy_snapshot(Kernel, Number, Bytes.GetData(), Bytes.Num(), &Len, &Got);
    }
    if (Code == TCE_NONE) return false;
    if (!Check(Code) || Code != TCE_OK) return false;
    Bytes.SetNum(static_cast<int32>(Len)); Number = Got; return true;
}
bool FKernelSession::Send(flatbuffers::FlatBufferBuilder& Builder, bool Query) {
    if (!Kernel) return false;
    commons_wire::FrameMeta Meta;
    Meta.kind = Query ? commons_wire::FrameKind::Query : commons_wire::FrameKind::Command;
    FMemory::Memcpy(Meta.schema.tag, "TCE", 4);
    Meta.schema.major = static_cast<uint16>(Versions.wire_major);
    Meta.schema.minor = static_cast<uint16>(Versions.wire_minor);
    Meta.epoch = Epoch; Meta.correlation = ++Correlation;
    std::vector<uint8> Bytes;
    if (commons_wire::encode_into(Meta, Builder.GetBufferPointer(), Builder.GetSize(), false, Bytes) != commons_wire::Error::None) return false;
    return Check(Api.submit(Kernel, Bytes.data(), Bytes.size()));
}
