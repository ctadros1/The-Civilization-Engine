#include "ObserverMode.h"
#include "ProceduralMeshComponent.h"
#include "Components/InstancedStaticMeshComponent.h"
#include "Camera/CameraComponent.h"
#include "Engine/DirectionalLight.h"
#include "Components/DirectionalLightComponent.h"
#include "Engine/SkyLight.h"
#include "Components/SkyLightComponent.h"
#include "Components/SkyAtmosphereComponent.h"
#include "Engine/World.h"
#include "Engine/GameViewportClient.h"
#include "GameFramework/PlayerController.h"
#include "GameFramework/DefaultPawn.h"
#include "Engine/Engine.h"
#include "SWebBrowser.h"
#include "Widgets/Layout/SBox.h"
#include "Widgets/Layout/SBorder.h"
#include "Styling/CoreStyle.h"
#include "Widgets/SOverlay.h"
#include "timed_path.hpp"
#include "Widgets/Input/SButton.h"
#include "Widgets/Text/STextBlock.h"
#include "Widgets/SBoxPanel.h"
#include "Misc/CommandLine.h"
#include "Misc/Parse.h"
#include "Widgets/SNullWidget.h"
#include "UnrealClient.h"
#include "Engine/TextureCube.h"
#include "Misc/Paths.h"
#include "KernelTerrainSurface.h"

namespace w = tce::wire;
namespace cw = commons_wire;
TSharedRef<SWidget> UObserverHud::RebuildWidget() { if(Content.IsValid()) return Content.ToSharedRef(); return SNullWidget::NullWidget; }
AObserverMode::AObserverMode() { DefaultPawnClass = nullptr; }
void AObserverMode::StartPlay() { Super::StartPlay(); GetWorld()->SpawnActor<AObserverScene>(); }

UInstancedStaticMeshComponent* AObserverScene::Pool(const TCHAR* Name, const TCHAR* Mesh) {
    auto* Result = CreateDefaultSubobject<UInstancedStaticMeshComponent>(Name);
    Result->SetupAttachment(RootComponent);
    Result->SetStaticMesh(LoadObject<UStaticMesh>(nullptr, Mesh));
    Result->SetCollisionEnabled(ECollisionEnabled::NoCollision);
    Result->SetMobility(EComponentMobility::Movable);
    return Result;
}
AObserverScene::AObserverScene() {
    PrimaryActorTick.bCanEverTick = true;
    RootComponent = CreateDefaultSubobject<USceneComponent>(TEXT("Root"));
    Terrain = CreateDefaultSubobject<UProceduralMeshComponent>(TEXT("RuntimeTerrain"));
    Terrain->SetupAttachment(RootComponent); Terrain->bUseAsyncCooking = true;
    Terrain->SetCastShadow(false);
    Grass=Pool(TEXT("GroundCover"),TEXT("/Game/Kit/Grass.Grass"));
    Grass->SetCastShadow(false);
    FieldSurfaces=Pool(TEXT("Fields"),TEXT("/Engine/BasicShapes/Plane.Plane"));
    FieldSurfaces->SetCastShadow(false);
    Water=CreateDefaultSubobject<UProceduralMeshComponent>(TEXT("WaterSurface"));
    Water->SetupAttachment(RootComponent); Water->SetCastShadow(false);
    Citizens = Pool(TEXT("Citizens"), TEXT("/Game/Kit/Citizen.Citizen"));
    Citizens->SetCastShadow(false);
    Heads=Pool(TEXT("Heads"),TEXT("/Game/Kit/Head.Head")); Heads->SetCastShadow(false);
    Limbs=Pool(TEXT("Limbs"),TEXT("/Game/Kit/Limb.Limb")); Limbs->SetCastShadow(false);
    Loads=Pool(TEXT("CarriedGoods"),TEXT("/Game/Kit/Timber.Timber"));
    Posts = Pool(TEXT("Timber"), TEXT("/Game/Kit/TimberPost.TimberPost"));
    Walls = Pool(TEXT("EarthWalls"), TEXT("/Game/Kit/DaubPanel.DaubPanel"));
    Roofs = Pool(TEXT("Thatch"), TEXT("/Game/Kit/ThatchCone.ThatchCone"));
    Beams=Pool(TEXT("RoofTimbers"),TEXT("/Game/Kit/TimberBeam.TimberBeam"));
    Wattle=Pool(TEXT("WovenPanels"),TEXT("/Game/Kit/WattlePanel.WattlePanel"));
    Floors=Pool(TEXT("EarthFloors"),TEXT("/Game/Kit/EarthFloor.EarthFloor"));
    Hearths=Pool(TEXT("Hearths"),TEXT("/Game/Kit/HearthBowl.HearthBowl"));
    Scaffolds=Pool(TEXT("TemporaryScaffolds"),TEXT("/Game/Kit/TimberBeam.TimberBeam"));
    RoofWedges=Pool(TEXT("RoofWork"),TEXT("/Game/Kit/ThatchWedge.ThatchWedge"));
    Embers=Pool(TEXT("Embers"),TEXT("/Engine/BasicShapes/Sphere.Sphere"));
    WindowGlows=Pool(TEXT("LitOpenings"),TEXT("/Engine/BasicShapes/Cube.Cube"));
    Terrain->SetMaterial(0,LoadObject<UMaterialInterface>(nullptr,TEXT("/Game/Kit/Ground.Ground")));
    Camera = CreateDefaultSubobject<UCameraComponent>(TEXT("ObserverCamera"));
    Camera->SetupAttachment(RootComponent);
}
void AObserverScene::BeginPlay() {
    Super::BeginPlay();
    Surface=NewObject<UKernelTerrainSurface>(this);
    Sun = GetWorld()->SpawnActor<ADirectionalLight>();
    Sun->SetMobility(EComponentMobility::Movable);
    Cast<UDirectionalLightComponent>(Sun->GetLightComponent())->SetAtmosphereSunLight(true);
    Cast<UDirectionalLightComponent>(Sun->GetLightComponent())->SetForwardShadingPriority(1);
    auto* Sky = GetWorld()->SpawnActor<ASkyLight>();
    Sky->GetLightComponent()->SetMobility(EComponentMobility::Movable);
    Sky->GetLightComponent()->SetIntensity(1.8f);
    Sky->GetLightComponent()->SourceType=SLS_SpecifiedCubemap;
    Sky->GetLightComponent()->SetCubemap(LoadObject<UTextureCube>(nullptr,TEXT("/Game/Kit/Ambient.Ambient")));
    Sky->GetLightComponent()->SetLightColor(FLinearColor(.35f,.45f,.65f));
    auto* Moon=GetWorld()->SpawnActor<ADirectionalLight>();
    Moon->SetMobility(EComponentMobility::Movable); Moon->SetActorRotation(FRotator(-35,-120,0));
    Moon->GetLightComponent()->SetIntensity(.15f);
    Moon->GetLightComponent()->SetLightColor(FLinearColor(.46f,.58f,1));
    Moon->GetLightComponent()->SetCastShadows(false);
    auto* AtmosphereActor = GetWorld()->SpawnActor<AActor>();
    auto* Atmosphere = NewObject<USkyAtmosphereComponent>(AtmosphereActor);
    AtmosphereActor->SetRootComponent(Atmosphere); Atmosphere->RegisterComponent();
    auto* PC = GetWorld()->GetFirstPlayerController();
    PC->SetViewTarget(this); PC->bShowMouseCursor = true;
    FInputModeGameAndUI Input; Input.SetHideCursorDuringCapture(false); PC->SetInputMode(Input);
    if (!Session.Start()) Caption = Session.Error;
    else {
        Panels = SNew(SOverlay) + SOverlay::Slot().HAlign(HAlign_Right).Padding(0,100,16,16)
            [ SNew(SBox).WidthOverride(500)[ SAssignNew(Browser,SWebBrowser).InitialURL(Session.PanelUrl).ShowControls(false).SupportsTransparency(true)
                .OnLoadCompleted_Lambda([this] {
                    Browser->ExecuteJavascript(TEXT("window.addEventListener('tce:focus',e=>console.log('TCE_FOCUS:'+e.detail.x+','+e.detail.y));"));
                })
                .OnConsoleMessage_Lambda([this](const FString& Message,const FString&,int32,EWebBrowserConsoleLogSeverity) {
                    if(!Message.StartsWith(TEXT("TCE_FOCUS:"))) return;
                    FString X,Y;
                    if(Message.Mid(10).Split(TEXT(","),&X,&Y)) {
                        double MX=FCString::Atod(*X), MY=FCString::Atod(*Y);
                        if(FMath::IsFinite(MX) && FMath::IsFinite(MY) && MX>=0 && MY>=0 && MX<=MapWidth*MapCell && MY<=MapHeight*MapCell) {
                            FollowId=0; Focus=FVector(MX*100,MY*100,0);
                        }
                    }
                })
            ] ];
        Caption = TEXT("N: new valley | Tab: panels | Space: pause | 1/2/3: speed | Ctrl+S: save | L: load | F: follow | WASD: pan | wheel: zoom");
    }
    TSharedRef<SHorizontalBox> Bar=SNew(SHorizontalBox);
    auto Button=[&](const TCHAR* Label,TFunction<void()> Action) {
        Bar->AddSlot().AutoWidth().Padding(4)[ SNew(SButton).Text(FText::FromString(Label)).OnClicked_Lambda([Action]() {Action(); return FReply::Handled();}) ];
    };
    Button(TEXT("New valley"),[this]{NewWorld();});
    Button(TEXT("Pause / Resume"),[this]{Clock(!Paused,Speed/BaseSpeed);});
    Button(TEXT("1x"),[this]{Clock(false,1);}); Button(TEXT("10x"),[this]{Clock(false,10);}); Button(TEXT("600x"),[this]{Clock(false,600);});
    Button(TEXT("Follow resident"),[this]{FollowId=SelectedId?SelectedId:(People.Num()?People[0].Id:0);SelectPersonInPanels(FollowId);Zoom=6500;});
    Button(TEXT("Save"),[this]{flatbuffers::FlatBufferBuilder B; auto V=w::CreateSaveWorldDirect(B,"Unreal observer save"); B.Finish(w::CreateCommand(B,w::CommandBody::SaveWorld,V.Union())); Session.Send(B);});
    Button(TEXT("Load latest"),[this]{flatbuffers::FlatBufferBuilder B; auto V=w::CreateListSaves(B); B.Finish(w::CreateQuery(B,w::QueryBody::ListSaves,V.Union())); Session.Send(B,true);});
    Button(TEXT("Quit"),[this]{GetWorld()->GetFirstPlayerController()->ConsoleCommand(TEXT("quit"));});
    Toolbar=SNew(SVerticalBox)
        +SVerticalBox::Slot().AutoHeight()[Bar]
        +SVerticalBox::Slot().AutoHeight().Padding(8)[SNew(STextBlock).Text_Lambda([this]{return FText::FromString(FString::Printf(TEXT("The Civilization Engine | %d people | %02d:%02d | %s %.0fx\n%s"),People.Num(),int32(Minute/60)%24,int32(Minute)%60,Paused?TEXT("Paused"):TEXT("Running"),Speed/BaseSpeed,*Caption));})];
    TSharedRef<SOverlay> Overlay=SNew(SOverlay);
    if(Panels.IsValid()) Overlay->AddSlot()[Panels.ToSharedRef()];
    Overlay->AddSlot().VAlign(VAlign_Top)[SNew(SBorder)
        .BorderImage(FCoreStyle::Get().GetBrush(TEXT("WhiteBrush")))
        .BorderBackgroundColor(FLinearColor(.015f,.02f,.025f,.9f)).Padding(0)[Toolbar.ToSharedRef()]];
    Hud=CreateWidget<UObserverHud>(PC);
    Hud->Content=Overlay; Hud->AddToViewport();
}
void AObserverScene::EndPlay(const EEndPlayReason::Type Reason) {
    if(Hud) { Hud->RemoveFromParent(); Hud->Content.Reset(); Hud=nullptr; }
    Toolbar.Reset();
    Browser.Reset(); Panels.Reset(); Session.Stop(); Super::EndPlay(Reason);
}
void AObserverScene::NewWorld() {
    if (Preset.IsEmpty()) return;
    flatbuffers::FlatBufferBuilder B; FTCHARToUTF8 P(*Preset);
    auto Body = w::CreateNewWorldDirect(B, 12, P.Get(), 512, "First light", 32);
    B.Finish(w::CreateCommand(B,w::CommandBody::NewWorld,Body.Union())); Session.Send(B);
}
void AObserverScene::Clock(bool Pause,float Rate) {
    flatbuffers::FlatBufferBuilder B; auto Body=w::CreateSetClock(B,Pause,Rate*BaseSpeed);
    B.Finish(w::CreateCommand(B,w::CommandBody::SetClock,Body.Union())); Session.Send(B);
}
void AObserverScene::QueryTerrain() {
    QueryRaster(w::RasterLayer::Elevation);
}
void AObserverScene::QueryRaster(w::RasterLayer Layer) {
    flatbuffers::FlatBufferBuilder B;
    auto Body=w::CreateGetRaster(B,Layer,RasterLevel,0,0,RasterWidth,RasterHeight);
    B.Finish(w::CreateQuery(B,w::QueryBody::GetRaster,Body.Union())); Session.Send(B,true);
}
void AObserverScene::ReadSnapshot() {
    cw::Frame F; auto* S=ReadKernelFrame<w::Snapshot>(SnapshotBytes,F); if(!S) return;
    Session.Epoch=F.header.meta.epoch;
    if(S->last_error() && S->last_error()->size()) Caption=UTF8_TO_TCHAR(S->last_error()->c_str());
    if(!S->world()) return;
    auto* World=S->world();
    FString Id=UTF8_TO_TCHAR(World->world_id()->c_str());
    if(CurrentEpoch!=Session.Epoch || Id!=WorldId) {
        CurrentEpoch=Session.Epoch; WorldId=Id; MapWidth=World->width(); MapHeight=World->height(); MapCell=World->cell_size_m();
        RasterLevel=0; RasterWidth=MapWidth; RasterHeight=MapHeight;
        while(int64(RasterWidth)*RasterHeight>1024*1024) {++RasterLevel;RasterWidth=(RasterWidth+1)/2;RasterHeight=(RasterHeight+1)/2;}
        SeaLevel=World->sea_level_m(); WaterClasses.Reset(); LakeIds.Reset(); LakeLevels.Reset(); RiverReaches.Reset();
        Grass->ClearInstances(); FieldSurfaces->ClearInstances(); Water->ClearAllMeshSections();
        BuildingPlots.Reset(); FieldPlots.Reset(); FoliageDirty=true; FieldRev=MAX_uint64;
        Heights.Reset(); People.Reset(); Trips.Reset(); CarriedKg.Reset(); HouseholdFuel.Reset(); LastPositions.Reset(); LastFacing.Reset(); SelectedId=0; Terrain->ClearAllMeshSections();
        for(auto& Tile:TerrainTiles) Tile->DestroyComponent();
        TerrainTiles.Reset(); TerrainSteps.Reset(); TerrainCollision.Reset();
        Posts->ClearInstances(); Walls->ClearInstances(); Roofs->ClearInstances();
        BuildingRev=MAX_uint64; GroundRev=MAX_uint64; FollowId=0;
        FocusedWorld=false;
        Focus=FVector(MapWidth*MapCell*50,MapHeight*MapCell*50,0); QueryTerrain();
        for(auto Layer:{w::RasterLayer::Water,w::RasterLayer::LakeId}) QueryRaster(Layer);
        flatbuffers::FlatBufferBuilder B; auto Body=w::CreateGetHydrography(B,1);
        B.Finish(w::CreateQuery(B,w::QueryBody::GetHydrography,Body.Union())); Session.Send(B,true);
    }
    if(S->clock()) {
        auto* C=S->clock(); Minute=C->minute(); Paused=C->paused(); Speed=C->speed();
        Still=Paused || C->mode()==w::ClockMode::Accelerated; Received=FPlatformTime::Seconds();
    }
    People.Reset();
    std::vector<uint64> Needed;
    if(S->people()) for(auto* P:*S->people()) {
        if(!P->pos()) continue;
        People.Add({P->id(),P->trip(),P->trip_rev(),FVector2D(P->pos()->x(),P->pos()->y()),P->household(),P->age_years(),P->asleep()});
        if(P->trip() && (!Trips.Contains(P->trip()) || Trips[P->trip()].Rev!=P->trip_rev())) Needed.push_back(P->trip());
    }
    if(!FocusedWorld && People.Num()) { Focus=FVector(People[0].Pos.X*100,People[0].Pos.Y*100,0); FocusedWorld=true; }
    TSet<uint64> Active; for(const auto& P:People) if(P.Trip) Active.Add(P.Trip);
    for(auto It=Trips.CreateIterator();It;++It) if(!Active.Contains(It.Key())) It.RemoveCurrent();
    if(!Needed.empty()) {
        flatbuffers::FlatBufferBuilder B; auto Body=w::CreateGetTripsDirect(B,&Needed);
        B.Finish(w::CreateQuery(B,w::QueryBody::GetTrips,Body.Union())); Session.Send(B,true);
    }
    if(BuildingRev!=S->buildings_rev()) {
        BuildingRev=S->buildings_rev(); BuildingQueryPending=true; flatbuffers::FlatBufferBuilder B; auto Body=w::CreateGetBuildings(B);
        B.Finish(w::CreateQuery(B,w::QueryBody::GetBuildings,Body.Union())); Session.Send(B,true);
    }
    if(GroundRev!=S->earthworks_rev()) { GroundRev=S->earthworks_rev(); if(Heights.Num()) QueryTerrain(); }
    if(FieldRev!=S->fields_rev()) {
        FieldRev=S->fields_rev(); flatbuffers::FlatBufferBuilder B; auto Body=w::CreateGetFields(B);
        B.Finish(w::CreateQuery(B,w::QueryBody::GetFields,Body.Union())); Session.Send(B,true);
    }
    if(S->task()) Caption=FString::Printf(TEXT("%s: %s %.0f%%"),UTF8_TO_TCHAR(S->task()->name()->c_str()),UTF8_TO_TCHAR(S->task()->stage()->c_str()),S->task()->fraction()*100);
    else if(!S->last_error() || !S->last_error()->size()) Caption=TEXT("WASD: move | wheel: zoom | F: follow | Tab: panels | Ctrl+S: save");
}
void AObserverScene::ReadReply(const TArray<uint8>& Bytes) {
    cw::Frame F;
    if(cw::decode(Bytes.GetData(),Bytes.Num(),F)!=cw::Error::None) return;
    if(F.header.meta.kind==cw::FrameKind::Welcome) {
        auto* W=ReadKernelFrame<w::Welcome>(Bytes,F);
        if(W && W->speed_1x()>0) BaseSpeed=W->speed_1x();
        if(W && W->presets()) for(auto* P:*W->presets()) if(P->is_default()) Preset=UTF8_TO_TCHAR(P->id()->c_str());
        return;
    }
    if(F.header.meta.kind==cw::FrameKind::Error) {
        auto* E=ReadKernelFrame<w::ErrorInfo>(Bytes,F); if(E && E->message()) Caption=UTF8_TO_TCHAR(E->message()->c_str()); return;
    }
    if(F.header.meta.kind!=cw::FrameKind::Response || F.header.meta.epoch!=Session.Epoch) return;
    auto* R=ReadKernelFrame<w::Response>(Bytes,F); if(!R) return;
    ReadEnvironment(R);
    if(auto* Tile=R->body_as_RasterTile()) BuildTerrain(Tile);
    if(auto* Buildings=R->body_as_Buildings()) BuildBuildings(Buildings);
    if(auto* Person=R->body_as_PersonInfo()) {
        CarriedKg.Add(Person->id(),Person->carry_kg()+Person->carry_water_l());
        HouseholdFuel.Add(Person->household(),Person->household_fuel_days());
    }
    if(auto* Paths=R->body_as_Trips()) if(Paths->trips()) for(auto* Path:*Paths->trips()) {
        if(!Path->points() || !Path->minutes() || Path->points()->size()!=Path->minutes()->size()) continue;
        FWalkPath P; P.Depart=Path->depart_minute(); P.Rev=Path->rev();
        for(auto* Point:*Path->points()) { P.XY.Add(Point->x()); P.XY.Add(Point->y()); }
        for(float Time:*Path->minutes()) P.Minutes.Add(Time);
        Trips.Add(Path->id(),MoveTemp(P));
    }
    if(auto* Saves=R->body_as_SaveList()) if(Saves->saves()) for(auto* Save:*Saves->saves()) if(Save->compatible()) {
        flatbuffers::FlatBufferBuilder B; auto Body=w::CreateLoadWorldDirect(B,Save->file()->c_str());
        B.Finish(w::CreateCommand(B,w::CommandBody::LoadWorld,Body.Union())); Session.Send(B); break;
    }
}
float AObserverScene::Ground(float X,float Y) const {
    if(!Heights.Num() || Width<2 || Height<2) return 0;
    float GX=FMath::Clamp(X/Cell,0.f,float(Width-1)), GY=FMath::Clamp(Y/Cell,0.f,float(Height-1));
    int32 IX=FMath::Min(int32(GX),Width-2), IY=FMath::Min(int32(GY),Height-2);
    return FMath::Lerp(FMath::Lerp(Heights[IY*Width+IX],Heights[IY*Width+IX+1],GX-IX),FMath::Lerp(Heights[(IY+1)*Width+IX],Heights[(IY+1)*Width+IX+1],GX-IX),GY-IY);
}
void AObserverScene::BuildTerrain(const w::RasterTile* Tile) {
    if(Tile->layer()!=w::RasterLayer::Elevation || !Tile->data()) return;
    int32 W=Tile->width(), H=Tile->height();
    if(W<2 || H<2 || W>4096 || H>4096) return;
    int32 Stride=Tile->format()==w::RasterFormat::F32 ? 4 : Tile->format()==w::RasterFormat::U16 ? 2 : 1;
    if(Tile->data()->size()!=uint64(W)*H*Stride) return;
    const bool SameSize=Width==W && Height==H && Heights.Num()==W*H;
    TArray<float> Previous=MoveTemp(Heights);
    Width=W; Height=H; Cell=MapCell*float(1<<Tile->level()); Heights.SetNum(W*H);
    for(int32 I=0;I<W*H;++I) {
        float V=0; const uint8* P=Tile->data()->data()+I*Stride;
        if(Stride==4) FMemory::Memcpy(&V,P,4); else if(Stride==2) V=P[0]+256*P[1]; else V=P[0];
        Heights[I]=V*Tile->scale()+Tile->offset();
    }
    UpdateTerrainTiles(Previous,!SameSize);
    Surface->Initialize(Heights,Width,Height,Cell); FoliageDirty=true;
    BuildFields();
    bool WetChanged=!SameSize;
    if(!WetChanged && WaterClasses.Num()==Heights.Num()) for(int32 I=0;I<Heights.Num();++I)
        if(WaterClasses[I] && FMath::Abs(Previous[I]-Heights[I])>.005f) {WetChanged=true;break;}
    if(WetChanged) BuildWater();
}

void AObserverScene::UpdateTerrainTiles(const TArray<float>& Previous,bool Force) {
    const double Started=FPlatformTime::Seconds(); LastChangedTiles=0;
    const int32 W=Width,H=Height;
    int32 Section=0;
    for(int32 Y0=0;Y0<H-1;Y0+=64) for(int32 X0=0;X0<W-1;X0+=64) {
        const int32 X1=FMath::Min(X0+64,W-1),Y1=FMath::Min(Y0+64,H-1);
        FBox2D Box(FVector2D(X0*Cell,Y0*Cell),FVector2D(X1*Cell,Y1*Cell));
        const double Distance=FVector2D::Distance(Box.GetCenter(),FVector2D(Focus.X/100,Focus.Y/100));
        const int32 Step=Distance<500?1:Distance<1500?2:4;
        const bool Collision=Box.ComputeSquaredDistanceToPoint(FVector2D(Focus.X/100,Focus.Y/100))<10000;
        while(TerrainTiles.Num()<=Section) {
            auto* Mesh=NewObject<UProceduralMeshComponent>(this);
            Mesh->SetupAttachment(RootComponent); Mesh->bUseAsyncCooking=true; Mesh->SetCastShadow(false);
            Mesh->ComponentTags.Add(TEXT("TCE_Terrain")); Mesh->RegisterComponent();
            TerrainTiles.Add(Mesh); TerrainSteps.Add(0); TerrainCollision.Add(false);
        }
        bool Changed=Force || TerrainSteps[Section]!=Step || TerrainCollision[Section]!=Collision;
        if(!Changed && Previous.Num()==Heights.Num()) for(int32 Y=FMath::Max(0,Y0-1);Y<=FMath::Min(H-1,Y1+1) && !Changed;++Y)
            for(int32 X=FMath::Max(0,X0-1);X<=FMath::Min(W-1,X1+1);++X)
                if(FMath::Abs(Previous[Y*W+X]-Heights[Y*W+X])>.005f) {Changed=true;break;}
        if(!Changed) { ++Section; continue; }
        ++LastChangedTiles;
        const int32 NX=FMath::DivideAndRoundUp(X1-X0,Step)+1,NY=FMath::DivideAndRoundUp(Y1-Y0,Step)+1;
        TArray<FVector> V,N; TArray<int32> Indices; TArray<FVector2D> UV; TArray<FLinearColor> Colors; TArray<FProcMeshTangent> Tangents;
        for(int32 Y=0;Y<NY;++Y) for(int32 X=0;X<NX;++X) {
            const int32 GX=FMath::Min(X1,X0+X*Step),GY=FMath::Min(Y1,Y0+Y*Step);
            float Z=Heights[GY*W+GX];
            V.Add(FVector(GX*Cell*100,GY*Cell*100,Z*100));
            float DX=Ground((GX+1)*Cell,GY*Cell)-Ground((GX-1)*Cell,GY*Cell);
            float DY=Ground(GX*Cell,(GY+1)*Cell)-Ground(GX*Cell,(GY-1)*Cell);
            N.Add(FVector(-DX,-DY,2*Cell).GetSafeNormal()); UV.Add(FVector2D(GX*0.1,GY*0.1));
            Colors.Add(FLinearColor(0.24f,0.31f,0.14f));
        }
        for(int32 Y=0;Y<NY-1;++Y) for(int32 X=0;X<NX-1;++X) { int32 A=Y*NX+X; Indices.Append({A,A+NX,A+1,A+1,A+NX,A+NX+1}); }
        // Vertical skirts cover the small height differences at neighboring CPU LOD edges.
        auto Skirt=[&](int32 A,int32 B) {
            int32 Start=V.Num();
            const FVector NA=N[A],NB=N[B]; const FVector2D UA=UV[A],UB=UV[B]; const FLinearColor CA=Colors[A],CB=Colors[B];
            V.Add(V[A]-FVector(0,0,Cell*Step*100)); V.Add(V[B]-FVector(0,0,Cell*Step*100));
            N.Add(NA); N.Add(NB); UV.Add(UA); UV.Add(UB); Colors.Add(CA); Colors.Add(CB);
            Indices.Append({A,Start,B,B,Start,Start+1});
        };
        for(int32 X=0;X<NX-1;++X) {Skirt(X,X+1);Skirt((NY-1)*NX+X,(NY-1)*NX+X+1);}
        for(int32 Y=0;Y<NY-1;++Y) {Skirt(Y*NX,(Y+1)*NX);Skirt(Y*NX+NX-1,(Y+1)*NX+NX-1);}
        auto* Mesh=TerrainTiles[Section].Get();
        Mesh->SetCollisionEnabled(Collision?ECollisionEnabled::QueryOnly:ECollisionEnabled::NoCollision);
        Mesh->SetCollisionResponseToAllChannels(ECR_Block);
        Mesh->CreateMeshSection_LinearColor(0,V,Indices,N,UV,Colors,Tangents,Collision);
        Mesh->SetMaterial(0,Terrain->GetMaterial(0));
        TerrainSteps[Section]=Step; TerrainCollision[Section]=Collision; ++Section;
    }
    LastTerrainBuildMs=(FPlatformTime::Seconds()-Started)*1000; TerrainLodFocus=Focus;
    if(LastChangedTiles) UE_LOG(LogTemp,Display,TEXT("TCE_TERRAIN_UPDATE tiles=%d changed=%d ms=%.3f"),Section,LastChangedTiles,LastTerrainBuildMs);
}
void AObserverScene::Tick(float Delta) {
    Super::Tick(Delta);
    for(int I=0;I<64 && Session.Poll(ReplyBytes);++I) ReadReply(ReplyBytes);
    if(Session.Snapshot(SnapshotBytes)) ReadSnapshot();
    if(FParse::Param(FCommandLine::Get(),TEXT("TCESpike"))) TickTerrainProbe(Delta);
    if(FParse::Param(FCommandLine::Get(),TEXT("TCESmoke"))) TickSessionProbe();
    if(FParse::Param(FCommandLine::Get(),TEXT("TCEDemo"))) {
        if(!SmokeRequested && !Preset.IsEmpty()) { SmokeRequested=true; NewWorld(); }
        if(Heights.Num() && People.Num() && !DemoAdvanceRequested) {
            DemoAdvanceRequested=true;
            int32 Days=90,Hour=21;
            FParse::Value(FCommandLine::Get(),TEXT("TCEDemoDays="),Days);
            FParse::Value(FCommandLine::Get(),TEXT("TCEDemoHour="),Hour);
            Days=FMath::Clamp(Days,1,365);Hour=FMath::Clamp(Hour,0,23);
            DemoTarget=(int64(Minute)/1440+Days)*1440+Hour*60;
            flatbuffers::FlatBufferBuilder B; auto Body=w::CreateRunUntil(B,DemoTarget);
            B.Finish(w::CreateCommand(B,w::CommandBody::RunUntil,Body.Union())); Session.Send(B);
        }
        if(DemoTarget && Minute>=DemoTarget && (Paused || DemoStarted>0) && (!BuildingQueryPending || DemoStarted>0)) {
            if(DemoStarted==0) {
                DemoStarted=FPlatformTime::Seconds(); FollowId=People.Num()?People[0].Id:0; Zoom=3500;
                SelectPersonInPanels(FollowId);
                if(FParse::Param(FCommandLine::Get(),TEXT("TCEDemoOverview")) && BuildingPlots.Num()) {
                    // A close construction view needs a real plot, not the empty midpoint of dispersed homes.
                    FVector2D Centre=BuildingPlots[0].GetCenter();
                    FollowId=0;Focus=FVector(Centre.X*100,Centre.Y*100,0);
                }
                if(FParse::Param(FCommandLine::Get(),TEXT("TCEDemoRun"))) Clock(false,1);
            }
            double Elapsed=FPlatformTime::Seconds()-DemoStarted;
            FString FrameFolder;
            if(Elapsed>5 && Elapsed<15 && Elapsed>=DemoNextFrame && FParse::Value(FCommandLine::Get(),TEXT("TCEDemoFrames="),FrameFolder)) {
                FScreenshotRequest::RequestScreenshot(FPaths::Combine(FrameFolder,FString::Printf(TEXT("frame%04d.jpg"),DemoFrameIndex++)),true,false);
                DemoNextFrame=Elapsed+1.0/30;
            }
            if(Elapsed>5) FrameTimes.Add(Delta*1000.0);
            if(Elapsed>15 && !DemoCaptureRequested) {
                DemoCaptureRequested=true;
                FrameTimes.Sort();
                double Sum=0; for(double V:FrameTimes) Sum+=V;
                if(FrameTimes.Num()) UE_LOG(LogTemp,Display,TEXT("TCE_DEMO_FRAME_MS mean=%.3f p99=%.3f samples=%d people=%d"),Sum/FrameTimes.Num(),FrameTimes[FMath::Min(FrameTimes.Num()-1,int32(FrameTimes.Num()*.99))],FrameTimes.Num(),People.Num());
                UE_LOG(LogTemp,Display,TEXT("TCE_DEMO_PRESENTATION posts=%d wall_panels=%d roofs=%d roof_work=%d scaffolds=%d carried_loads=%d lit_hearths=%d follow=%llu"),Posts->GetInstanceCount(),Walls->GetInstanceCount(),Roofs->GetInstanceCount(),RoofWedges->GetInstanceCount(),Scaffolds->GetInstanceCount(),Loads->GetInstanceCount(),Embers->GetInstanceCount(),FollowId);
                FString ImageName=TEXT("FirstLight.png"); FParse::Value(FCommandLine::Get(),TEXT("TCEDemoImage="),ImageName);
                FScreenshotRequest::RequestScreenshot(FPaths::GetCleanFilename(ImageName),true,false);
                flatbuffers::FlatBufferBuilder B; auto Body=w::CreateSaveWorldDirect(B,"First light at night");
                B.Finish(w::CreateCommand(B,w::CommandBody::SaveWorld,Body.Union())); Session.Send(B);
            }
            if(Elapsed>20 && FParse::Param(FCommandLine::Get(),TEXT("TCEDemoExit"))) GetWorld()->GetFirstPlayerController()->ConsoleCommand(TEXT("quit"));
        }
    }
    auto* PC=GetWorld()->GetFirstPlayerController(); if(!PC) return;
    if(PC->WasInputKeyJustPressed(EKeys::N)) NewWorld();
    if(PC->WasInputKeyJustPressed(EKeys::SpaceBar)) Clock(!Paused,Speed/BaseSpeed);
    if(PC->WasInputKeyJustPressed(EKeys::One)) Clock(false,1);
    if(PC->WasInputKeyJustPressed(EKeys::Two)) Clock(false,10);
    if(PC->WasInputKeyJustPressed(EKeys::Three)) Clock(false,600);
    if(PC->WasInputKeyJustPressed(EKeys::Tab) && Panels.IsValid()) { ShowPanels=!ShowPanels; Panels->SetVisibility(ShowPanels?EVisibility::SelfHitTestInvisible:EVisibility::Collapsed); }
    if(PC->WasInputKeyJustPressed(EKeys::F)) {FollowId=FollowId?0:(SelectedId?SelectedId:(People.Num()?People[0].Id:0));SelectPersonInPanels(FollowId);}
    if(PC->WasInputKeyJustPressed(EKeys::LeftMouseButton)) PickPerson();
    if(PC->WasInputKeyJustPressed(EKeys::S) && PC->IsInputKeyDown(EKeys::LeftControl)) {
        flatbuffers::FlatBufferBuilder B; auto Body=w::CreateSaveWorldDirect(B,"Unreal observer save"); B.Finish(w::CreateCommand(B,w::CommandBody::SaveWorld,Body.Union())); Session.Send(B);
    }
    if(PC->WasInputKeyJustPressed(EKeys::L)) { flatbuffers::FlatBufferBuilder B; auto Body=w::CreateListSaves(B); B.Finish(w::CreateQuery(B,w::QueryBody::ListSaves,Body.Union())); Session.Send(B,true); }
    Zoom=FMath::Clamp(Zoom-PC->GetInputAnalogKeyState(EKeys::MouseWheelAxis)*Zoom*.15f,600.f,300000.f);
    if(!FollowId) {
        Focus.X+=(PC->IsInputKeyDown(EKeys::W)-PC->IsInputKeyDown(EKeys::S))*Delta*Zoom;
        Focus.Y+=(PC->IsInputKeyDown(EKeys::D)-PC->IsInputKeyDown(EKeys::A))*Delta*Zoom;
    }
    Yaw+=(PC->IsInputKeyDown(EKeys::E)-PC->IsInputKeyDown(EKeys::Q))*Delta*60;
    double T=engine_bridge::estimate_minute(Minute,Still,Speed,Received*1000,FPlatformTime::Seconds()*1000);
    DrawCrowd(T); RefreshCarriedGoods();
    Focus.Z=Ground(Focus.X/100,Focus.Y/100)*100;
    if(Heights.Num() && FVector::DistSquared2D(Focus,TerrainLodFocus)>FMath::Square(5000.)) {
        TArray<float> Unchanged; UpdateTerrainTiles(Unchanged,false);
    }
    if(Heights.Num() && (FoliageDirty || FVector::DistSquared2D(Focus,FoliageFocus)>FMath::Square(3000.0))) BuildFoliage();
    Camera->SetWorldLocation(Focus+FRotator(0,Yaw,0).RotateVector(FVector(-Zoom*.7,-Zoom*.7,Zoom*.8)));
    Camera->SetWorldRotation((Focus-Camera->GetComponentLocation()).Rotation());
    float Hour=FMath::Fmod(float(T),1440.f)/60.f;
    DrawHearthLight(Hour);
    if(Sun) { Sun->SetActorRotation(FRotator(-(Hour-6)*15,35,0)); Sun->GetLightComponent()->SetIntensity(FMath::Max(.04f,FMath::Sin((Hour-6)*PI/12)*5.f)); }
}
