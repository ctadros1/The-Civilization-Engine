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
    Citizens = Pool(TEXT("Citizens"), TEXT("/Game/Kit/Citizen.Citizen"));
    Citizens->SetCastShadow(false);
    Posts = Pool(TEXT("Timber"), TEXT("/Game/Kit/Timber.Timber"));
    Walls = Pool(TEXT("EarthWalls"), TEXT("/Game/Kit/EarthWall.EarthWall"));
    Roofs = Pool(TEXT("Thatch"), TEXT("/Game/Kit/Thatch.Thatch"));
    Terrain->SetMaterial(0,LoadObject<UMaterialInterface>(nullptr,TEXT("/Game/Kit/Ground.Ground")));
    Camera = CreateDefaultSubobject<UCameraComponent>(TEXT("ObserverCamera"));
    Camera->SetupAttachment(RootComponent);
}
void AObserverScene::BeginPlay() {
    Super::BeginPlay();
    Sun = GetWorld()->SpawnActor<ADirectionalLight>();
    Sun->SetMobility(EComponentMobility::Movable);
    Cast<UDirectionalLightComponent>(Sun->GetLightComponent())->SetAtmosphereSunLight(true);
    auto* Sky = GetWorld()->SpawnActor<ASkyLight>();
    Sky->GetLightComponent()->SetMobility(EComponentMobility::Movable);
    Sky->GetLightComponent()->SetIntensity(0.6f);
    Sky->GetLightComponent()->SourceType=SLS_SpecifiedCubemap;
    Sky->GetLightComponent()->SetCubemap(LoadObject<UTextureCube>(nullptr,TEXT("/Game/Kit/Ambient.Ambient")));
    Sky->GetLightComponent()->SetLightColor(FLinearColor(.35f,.45f,.65f));
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
    Button(TEXT("Follow resident"),[this]{FollowId=People.Num()?People[0].Id:0;});
    Button(TEXT("Save"),[this]{flatbuffers::FlatBufferBuilder B; auto V=w::CreateSaveWorldDirect(B,"Unreal observer save"); B.Finish(w::CreateCommand(B,w::CommandBody::SaveWorld,V.Union())); Session.Send(B);});
    Button(TEXT("Load latest"),[this]{flatbuffers::FlatBufferBuilder B; auto V=w::CreateListSaves(B); B.Finish(w::CreateQuery(B,w::QueryBody::ListSaves,V.Union())); Session.Send(B,true);});
    Button(TEXT("Quit"),[this]{GetWorld()->GetFirstPlayerController()->ConsoleCommand(TEXT("quit"));});
    Toolbar=SNew(SVerticalBox)
        +SVerticalBox::Slot().AutoHeight()[Bar]
        +SVerticalBox::Slot().AutoHeight().Padding(8)[SNew(STextBlock).Text_Lambda([this]{return FText::FromString(FString::Printf(TEXT("The Civilization Engine | %d people | %02d:%02d | %s %.0fx\n%s"),People.Num(),int32(Minute/60)%24,int32(Minute)%60,Paused?TEXT("Paused"):TEXT("Running"),Speed/BaseSpeed,*Caption));})];
    TSharedRef<SOverlay> Overlay=SNew(SOverlay);
    if(Panels.IsValid()) Overlay->AddSlot()[Panels.ToSharedRef()];
    Overlay->AddSlot().VAlign(VAlign_Top)[Toolbar.ToSharedRef()];
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
    flatbuffers::FlatBufferBuilder B;
    auto Body=w::CreateGetRaster(B,w::RasterLayer::Elevation,0,0,0,MapWidth,MapHeight);
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
        Heights.Reset(); People.Reset(); Trips.Reset(); Terrain->ClearAllMeshSections();
        Posts->ClearInstances(); Walls->ClearInstances(); Roofs->ClearInstances();
        BuildingRev=MAX_uint64; GroundRev=MAX_uint64; FollowId=0;
        FocusedWorld=false;
        Focus=FVector(MapWidth*MapCell*50,MapHeight*MapCell*50,0); QueryTerrain();
    }
    if(S->clock()) {
        auto* C=S->clock(); Minute=C->minute(); Paused=C->paused(); Speed=C->speed();
        Still=Paused || C->mode()==w::ClockMode::Accelerated; Received=FPlatformTime::Seconds();
    }
    People.Reset();
    std::vector<uint64> Needed;
    if(S->people()) for(auto* P:*S->people()) {
        if(!P->pos()) continue;
        People.Add({P->id(),P->trip(),P->trip_rev(),FVector2D(P->pos()->x(),P->pos()->y())});
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
        BuildingRev=S->buildings_rev(); flatbuffers::FlatBufferBuilder B; auto Body=w::CreateGetBuildings(B);
        B.Finish(w::CreateQuery(B,w::QueryBody::GetBuildings,Body.Union())); Session.Send(B,true);
    }
    if(GroundRev!=S->earthworks_rev()) { GroundRev=S->earthworks_rev(); if(Heights.Num()) QueryTerrain(); }
    if(S->task()) Caption=FString::Printf(TEXT("%s: %s %.0f%%"),UTF8_TO_TCHAR(S->task()->name()->c_str()),UTF8_TO_TCHAR(S->task()->stage()->c_str()),S->task()->fraction()*100);
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
    if(auto* Tile=R->body_as_RasterTile()) BuildTerrain(Tile);
    if(auto* Buildings=R->body_as_Buildings()) BuildBuildings(Buildings);
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
    if(!SameSize) Terrain->ClearAllMeshSections();
    int32 Section=0;
    for(int32 Y0=0;Y0<H-1;Y0+=64) for(int32 X0=0;X0<W-1;X0+=64) {
        int32 NX=FMath::Min(65,W-X0), NY=FMath::Min(65,H-Y0);
        bool Changed=!SameSize;
        if(SameSize) for(int32 Y=0;Y<NY && !Changed;++Y) for(int32 X=0;X<NX;++X)
            if(FMath::Abs(Previous[(Y0+Y)*W+X0+X]-Heights[(Y0+Y)*W+X0+X])>.005f) { Changed=true; break; }
        if(!Changed) { ++Section; continue; }
        TArray<FVector> V,N; TArray<int32> Indices; TArray<FVector2D> UV; TArray<FLinearColor> Colors; TArray<FProcMeshTangent> Tangents;
        for(int32 Y=0;Y<NY;++Y) for(int32 X=0;X<NX;++X) {
            float Z=Heights[(Y0+Y)*W+X0+X];
            V.Add(FVector((X0+X)*Cell*100,(Y0+Y)*Cell*100,Z*100));
            float DX=Ground((X0+X+1)*Cell,(Y0+Y)*Cell)-Ground((X0+X-1)*Cell,(Y0+Y)*Cell);
            float DY=Ground((X0+X)*Cell,(Y0+Y+1)*Cell)-Ground((X0+X)*Cell,(Y0+Y-1)*Cell);
            N.Add(FVector(-DX,-DY,2*Cell).GetSafeNormal()); UV.Add(FVector2D((X0+X)*0.1,(Y0+Y)*0.1));
            Colors.Add(FLinearColor(0.24f,0.31f,0.14f));
        }
        for(int32 Y=0;Y<NY-1;++Y) for(int32 X=0;X<NX-1;++X) { int32 A=Y*NX+X; Indices.Append({A,A+NX,A+1,A+1,A+NX,A+NX+1}); }
        Terrain->CreateMeshSection_LinearColor(Section,V,Indices,N,UV,Colors,Tangents,false);
        Terrain->SetMaterial(Section++,LoadObject<UMaterialInterface>(nullptr,TEXT("/Game/Kit/Ground.Ground")));
    }
    UE_LOG(LogTemp,Display,TEXT("Runtime terrain: %dx%d, %d tiles"),W,H,Section);
}
void AObserverScene::BuildBuildings(const w::Buildings* Data) {
    Posts->ClearInstances(); Walls->ClearInstances(); Roofs->ClearInstances();
    if(!Data->buildings()) return;
    for(auto* B:*Data->buildings()) {
        if(!B->centre() || B->state()==w::BuildingState::Ruin) continue;
        const float X=B->centre()->x(), Y=B->centre()->y(), Z=Ground(X,Y)*100;
        if(B->stage()>=1 && B->posts()) for(auto* P:*B->posts())
            Posts->AddInstance(FTransform(FRotator::ZeroRotator,FVector(P->x()*100,P->y()*100,Z+125),FVector(.15,.15,2.5)));
        if(B->stage()>=2 && B->outline()) {
            auto* O=B->outline(); for(uint32 I=1;I<O->size();++I) {
                FVector A(O->Get(I-1)->x()*100,O->Get(I-1)->y()*100,Z+100), C(O->Get(I)->x()*100,O->Get(I)->y()*100,Z+100);
                float Length=FVector::Distance(A,C);
                Walls->AddInstance(FTransform((C-A).Rotation(),(A+C)/2,FVector(Length/100,.12,2)));
            }
        }
        if(B->roofed()) Roofs->AddInstance(FTransform(FRotator::ZeroRotator,FVector(X*100,Y*100,Z+275),FVector(B->roof_radius_m()*2,B->roof_radius_m()*2,1.5)));
    }
}
void AObserverScene::Tick(float Delta) {
    Super::Tick(Delta);
    for(int I=0;I<64 && Session.Poll(ReplyBytes);++I) ReadReply(ReplyBytes);
    if(Session.Snapshot(SnapshotBytes)) ReadSnapshot();
    if(FParse::Param(FCommandLine::Get(),TEXT("TCESmoke"))) {
        if(!SmokeRequested && !Preset.IsEmpty()) { SmokeRequested=true; NewWorld(); }
        if(Heights.Num() && People.Num()) {
            UE_LOG(LogTemp,Display,TEXT("TCE_SMOKE_OK people=%d terrain=%dx%d"),People.Num(),Width,Height);
            GetWorld()->GetFirstPlayerController()->ConsoleCommand(TEXT("quit"));
        }
    }
    if(FParse::Param(FCommandLine::Get(),TEXT("TCEDemo"))) {
        if(!SmokeRequested && !Preset.IsEmpty()) { SmokeRequested=true; NewWorld(); }
        if(Heights.Num() && People.Num() && !DemoAdvanceRequested) {
            DemoAdvanceRequested=true;
            DemoTarget=(int64(Minute)/1440+12)*1440+21*60;
            flatbuffers::FlatBufferBuilder B; auto Body=w::CreateRunUntil(B,DemoTarget);
            B.Finish(w::CreateCommand(B,w::CommandBody::RunUntil,Body.Union())); Session.Send(B);
        }
        if(DemoTarget && Minute>=DemoTarget && Paused) {
            if(DemoStarted==0) { DemoStarted=FPlatformTime::Seconds(); FollowId=People.Num()?People[0].Id:0; }
            double Elapsed=FPlatformTime::Seconds()-DemoStarted;
            if(Elapsed>5) FrameTimes.Add(Delta*1000.0);
            if(Elapsed>15 && !DemoCaptureRequested) {
                DemoCaptureRequested=true;
                FrameTimes.Sort();
                double Sum=0; for(double V:FrameTimes) Sum+=V;
                if(FrameTimes.Num()) UE_LOG(LogTemp,Display,TEXT("TCE_DEMO_FRAME_MS mean=%.3f p99=%.3f samples=%d people=%d"),Sum/FrameTimes.Num(),FrameTimes[FMath::Min(FrameTimes.Num()-1,int32(FrameTimes.Num()*.99))],FrameTimes.Num(),People.Num());
                FScreenshotRequest::RequestScreenshot(TEXT("FirstLight.png"),true,false);
                flatbuffers::FlatBufferBuilder B; auto Body=w::CreateSaveWorldDirect(B,"First light at night");
                B.Finish(w::CreateCommand(B,w::CommandBody::SaveWorld,Body.Union())); Session.Send(B);
            }
        }
    }
    auto* PC=GetWorld()->GetFirstPlayerController(); if(!PC) return;
    if(PC->WasInputKeyJustPressed(EKeys::N)) NewWorld();
    if(PC->WasInputKeyJustPressed(EKeys::SpaceBar)) Clock(!Paused,Speed/BaseSpeed);
    if(PC->WasInputKeyJustPressed(EKeys::One)) Clock(false,1);
    if(PC->WasInputKeyJustPressed(EKeys::Two)) Clock(false,10);
    if(PC->WasInputKeyJustPressed(EKeys::Three)) Clock(false,600);
    if(PC->WasInputKeyJustPressed(EKeys::Tab) && Panels.IsValid()) { ShowPanels=!ShowPanels; Panels->SetVisibility(ShowPanels?EVisibility::SelfHitTestInvisible:EVisibility::Collapsed); }
    if(PC->WasInputKeyJustPressed(EKeys::F)) FollowId=FollowId ? 0 : (People.Num()?People[0].Id:0);
    if(PC->WasInputKeyJustPressed(EKeys::S) && PC->IsInputKeyDown(EKeys::LeftControl)) {
        flatbuffers::FlatBufferBuilder B; auto Body=w::CreateSaveWorldDirect(B,"Unreal observer save"); B.Finish(w::CreateCommand(B,w::CommandBody::SaveWorld,Body.Union())); Session.Send(B);
    }
    if(PC->WasInputKeyJustPressed(EKeys::L)) { flatbuffers::FlatBufferBuilder B; auto Body=w::CreateListSaves(B); B.Finish(w::CreateQuery(B,w::QueryBody::ListSaves,Body.Union())); Session.Send(B,true); }
    Zoom=FMath::Clamp(Zoom-PC->GetInputAnalogKeyState(EKeys::MouseWheelAxis)*Zoom*.15f,600.f,300000.f);
    if(!FollowId) {
        Focus.X+=(PC->IsInputKeyDown(EKeys::W)-PC->IsInputKeyDown(EKeys::S))*Delta*Zoom;
        Focus.Y+=(PC->IsInputKeyDown(EKeys::D)-PC->IsInputKeyDown(EKeys::A))*Delta*Zoom;
    }
    double T=engine_bridge::estimate_minute(Minute,Still,Speed,Received*1000,FPlatformTime::Seconds()*1000);
    if(Citizens->GetInstanceCount()!=People.Num()) { Citizens->ClearInstances(); for(int I=0;I<People.Num();++I) Citizens->AddInstance(FTransform::Identity); }
    for(int32 I=0;I<People.Num();++I) {
        auto& P=People[I]; FVector2D Pos=P.Pos;
        if(!Still) if(auto* Path=Trips.Find(P.Trip)) if(Path->Rev==P.Rev) { auto Pt=engine_bridge::position_on_path(Path->XY.GetData(),Path->Minutes.GetData(),Path->Minutes.Num(),Path->Depart,T); Pos=FVector2D(Pt.x,Pt.y); }
        FVector Location(Pos.X*100,Pos.Y*100,Ground(Pos.X,Pos.Y)*100+85);
        Citizens->UpdateInstanceTransform(I,FTransform(FRotator::ZeroRotator,Location,FVector(.45,.45,1.7)),false,false,true);
        if(P.Id==FollowId) Focus=Location;
    }
    Citizens->MarkRenderStateDirty();
    Focus.Z=Ground(Focus.X/100,Focus.Y/100)*100;
    Camera->SetWorldLocation(Focus+FVector(-Zoom*.7,-Zoom*.7,Zoom*.8));
    Camera->SetWorldRotation((Focus-Camera->GetComponentLocation()).Rotation());
    float Hour=FMath::Fmod(float(T),1440.f)/60.f;
    if(Sun) { Sun->SetActorRotation(FRotator(-(Hour-6)*15,35,0)); Sun->GetLightComponent()->SetIntensity(FMath::Max(.04f,FMath::Sin((Hour-6)*PI/12)*5.f)); }
}
