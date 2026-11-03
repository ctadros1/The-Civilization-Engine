#pragma once
#include "CoreMinimal.h"
#include "GameFramework/GameModeBase.h"
#include "GameFramework/Actor.h"
#include "KernelSession.h"
#include "Blueprint/UserWidget.h"
#include "ObserverMode.generated.h"

class UProceduralMeshComponent;
class UInstancedStaticMeshComponent;
class UCameraComponent;
class ADirectionalLight;
class SWidget;
class SWebBrowser;
struct FWalker {
    uint64 Id = 0, Trip = 0;
    uint32 Rev = 0;
    FVector2D Pos;
};
struct FWalkPath { TArray<float> XY, Minutes; double Depart = 0; uint32 Rev = 0; };

// UMG owns viewport placement and teardown; Slate supplies the native controls and CEF surface.
UCLASS()
class UObserverHud : public UUserWidget {
    GENERATED_BODY()
public:
    TSharedPtr<SWidget> Content;
protected:
    virtual TSharedRef<SWidget> RebuildWidget() override;
};

UCLASS()
class AObserverScene : public AActor {
    GENERATED_BODY()
public:
    AObserverScene();
    virtual void BeginPlay() override;
    virtual void Tick(float Delta) override;
    virtual void EndPlay(const EEndPlayReason::Type Reason) override;
private:
    void ReadSnapshot();
    void ReadReply(const TArray<uint8>& Bytes);
    void QueryTerrain();
    void BuildTerrain(const tce::wire::RasterTile* Tile);
    void BuildBuildings(const tce::wire::Buildings* Buildings);
    float Ground(float X, float Y) const;
    void NewWorld();
    void Clock(bool Pause, float Rate);
    UInstancedStaticMeshComponent* Pool(const TCHAR* Name, const TCHAR* Mesh);
    FKernelSession Session;
    TArray<uint8> SnapshotBytes, ReplyBytes;
    TArray<float> Heights;
    TArray<FWalker> People;
    TMap<uint64, FWalkPath> Trips;
    UPROPERTY() TObjectPtr<UProceduralMeshComponent> Terrain;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Citizens;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Posts;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Walls;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Roofs;
    UPROPERTY() TObjectPtr<UCameraComponent> Camera;
    UPROPERTY() TObjectPtr<ADirectionalLight> Sun;
    UPROPERTY() TObjectPtr<UObserverHud> Hud;
    TSharedPtr<SWidget> Panels;
    TSharedPtr<SWidget> Toolbar;
    TSharedPtr<SWebBrowser> Browser;
    FString Preset, WorldId, Caption;
    uint64 BuildingRev = MAX_uint64, GroundRev = MAX_uint64;
    uint64 FollowId = 0;
    double Minute = 0, Received = 0;
    float Speed = 1, BaseSpeed = 1, Cell = 1, MapCell = 1, Yaw = 0, Zoom = 16000;
    int32 Width = 0, Height = 0, MapWidth = 0, MapHeight = 0;
    uint32 CurrentEpoch = 0;
    FVector Focus = FVector::ZeroVector;
    bool Paused = true, Still = true, ShowPanels = true;
    bool SmokeRequested = false;
    bool FocusedWorld = false;
    bool DemoAdvanceRequested = false;
    bool DemoCaptureRequested = false;
    double DemoStarted = 0;
    int64 DemoTarget = 0;
    TArray<double> FrameTimes;
};

UCLASS()
class AObserverMode : public AGameModeBase {
    GENERATED_BODY()
public:
    AObserverMode();
    virtual void StartPlay() override;
};
