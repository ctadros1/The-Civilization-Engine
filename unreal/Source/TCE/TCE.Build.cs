using UnrealBuildTool;
public class TCE : ModuleRules {
    public TCE(ReadOnlyTargetRules Target) : base(Target) {
        PCHUsage = PCHUsageMode.UseExplicitOrSharedPCHs;
        PublicDependencyModuleNames.AddRange(new[] {"Core", "CoreUObject", "Engine", "EngineBridge", "InputCore", "ProceduralMeshComponent", "Slate", "SlateCore", "WebBrowser", "UMG"});
    }
}
