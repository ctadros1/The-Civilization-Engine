using UnrealBuildTool;
using System.IO;
public class EngineBridge : ModuleRules {
    public EngineBridge(ReadOnlyTargetRules Target) : base(Target) {
        PCHUsage = PCHUsageMode.UseExplicitOrSharedPCHs;
        PublicDependencyModuleNames.AddRange(new[] {"Core", "CoreUObject", "Engine"});
        string Root = Path.GetFullPath(Path.Combine(PluginDirectory, "../../.."));
        PublicSystemIncludePaths.AddRange(new[] {
            Path.Combine(Root, "kernel/crates/civ-ffi/include"),
            Path.Combine(Root, "kernel/crates/civ-schema/cpp"),
            Path.Combine(Root, "commons/cpp/include"),
            Path.Combine(Root, "commons/cpp/third_party/flatbuffers/include")
        });
        RuntimeDependencies.Add("$(TargetOutputDir)/tce_kernel.dll", Path.Combine(Root, "dist/tce_kernel/x86_64-pc-windows-msvc/tce_kernel.dll"));
        RuntimeDependencies.Add("$(ProjectDir)/Data/content/...", Path.Combine(Root, "content/..."), StagedFileType.NonUFS);
        RuntimeDependencies.Add("$(ProjectDir)/Data/web/...", Path.Combine(Root, "web/dist/..."), StagedFileType.NonUFS);
    }
}
