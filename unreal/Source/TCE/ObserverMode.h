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
class UKernelTerrainSurface;
class UPointLightComponent;
struct FHearthPresentation { uint64 Household=0; FVector Hearth, Opening; float Direction=0; };
struct FWalker {
    uint64 Id = 0, Trip = 0;
    uint32 Rev = 0;
    FVector2D Pos;
    uint64 Household = 0;
    float Age = 20;
    bool Asleep = false;
};
struct FWalkPath { TArray<float> XY, Minutes; double Depart = 0; uint32 Rev = 0; };
struct FWaterReach { TArray<FVector2D> Points; float Width=0; };

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
    void QueryRaster(tce::wire::RasterLayer Layer);
    void BuildTerrain(const tce::wire::RasterTile* Tile);
    void BuildBuildings(const tce::wire::Buildings* Buildings);
    void BuildFoliage();
    void BuildWater();
    void BuildFields();
    void ReadEnvironment(const tce::wire::Response* Reply);
    void DrawCrowd(double RenderMinute);
    void RefreshCarriedGoods();
    void DrawHearthLight(float Hour);
    void UpdateTerrainTiles(const TArray<float>& Previous, bool Force);
    bool TerrainCollisionReady() const;
    void TickTerrainProbe(float Delta);
    void TickSessionProbe();
    void PickPerson();
    void SelectPersonInPanels(uint64 Id);
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
    UPROPERTY() TArray<TObjectPtr<UProceduralMeshComponent>> TerrainTiles;
    TArray<int32> TerrainSteps;
    TArray<bool> TerrainCollision;
    FVector TerrainLodFocus = FVector(MAX_flt,MAX_flt,0);
    double LastTerrainBuildMs = 0;
    int32 LastChangedTiles = 0;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Citizens;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Posts;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Walls;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Roofs;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Heads;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Limbs;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Loads;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Beams;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Wattle;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Floors;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Hearths;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Scaffolds;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> RoofWedges;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Embers;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> WindowGlows;
    UPROPERTY() TArray<TObjectPtr<UPointLightComponent>> HearthLights;
    TArray<FHearthPresentation> HearthPresentations;
    TMap<uint64,float> HouseholdFuel;
    TMap<uint64,float> CarriedKg;
    TMap<uint64,FVector> LastPositions;
    TMap<uint64,float> LastFacing;
    double LastCarryQuery = 0;
    UPROPERTY() TObjectPtr<UCameraComponent> Camera;
    UPROPERTY() TObjectPtr<ADirectionalLight> Sun;
    UPROPERTY() TObjectPtr<UObserverHud> Hud;
    UPROPERTY() TObjectPtr<UKernelTerrainSurface> Surface;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> Grass;
    UPROPERTY() TObjectPtr<UInstancedStaticMeshComponent> FieldSurfaces;
    UPROPERTY() TObjectPtr<UProceduralMeshComponent> Water;
    TArray<uint8> WaterClasses;
    TArray<uint32> LakeIds;
    TMap<uint32,float> LakeLevels;
    TArray<FWaterReach> RiverReaches;
    TArray<FBox2D> BuildingPlots, FieldPlots;
    FVector FoliageFocus = FVector(MAX_flt,MAX_flt,0);
    float SeaLevel = 0;
    bool FoliageDirty = true;
    uint64 FieldRev = MAX_uint64;
    TSharedPtr<SWidget> Panels;
    TSharedPtr<SWidget> Toolbar;
    TSharedPtr<SWebBrowser> Browser;
    FString Preset, WorldId, Caption;
    uint64 BuildingRev = MAX_uint64, GroundRev = MAX_uint64;
    bool BuildingQueryPending = true;
    uint64 FollowId = 0;
    uint64 SelectedId = 0;
    double Minute = 0, Received = 0;
    float Speed = 1, BaseSpeed = 1, Cell = 1, MapCell = 1, Yaw = 0, Zoom = 16000;
    int32 Width = 0, Height = 0, MapWidth = 0, MapHeight = 0;
    int32 RasterWidth = 0, RasterHeight = 0;
    uint8 RasterLevel = 0;
    uint32 CurrentEpoch = 0;
    FVector Focus = FVector::ZeroVector;
    bool Paused = true, Still = true, ShowPanels = true;
    bool SmokeRequested = false;
    int32 SmokePhase=0, SmokePeople=0;
    uint32 SmokeEpoch=0;
    double SmokeStarted=0, SmokeSavedMinute=0;
    FString SmokeWorld;
    bool FocusedWorld = false;
    bool DemoAdvanceRequested = false;
    bool DemoCaptureRequested = false;
    double DemoStarted = 0;
    int64 DemoTarget = 0;
    TArray<double> FrameTimes;
    double DemoNextFrame=0;
    int32 DemoFrameIndex=0;
    TArray<uint8> ProbeBaseline, ProbeEdited;
    TArray<double> ProbeFrames, ProbePatchTimes;
    int32 ProbePhase=0, ProbeEdits=0, ProbeX=0, ProbeY=0;
    float ProbeBefore=0, ProbeAfter=0;
    double ProbeStarted=0, ProbeNextEdit=0;
};

UCLASS()
class AObserverMode : public AGameModeBase {
    GENERATED_BODY()
public:
    AObserverMode();
    virtual void StartPlay() override;
};
