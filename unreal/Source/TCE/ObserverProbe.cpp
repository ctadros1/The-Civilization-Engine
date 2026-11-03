#include "ObserverMode.h"
#include "KernelTerrainSurface.h"
#include "Components/InstancedStaticMeshComponent.h"
#include "Misc/CommandLine.h"
#include "Misc/FileHelper.h"
#include "Misc/Parse.h"
#include "HAL/PlatformMisc.h"
#include "Misc/Paths.h"
#include "Engine/World.h"
#include "GameFramework/PlayerController.h"

void AObserverScene::TickSessionProbe() {
    namespace w=tce::wire;
    const double Now=FPlatformTime::Seconds();
    if(!SmokeRequested && !Preset.IsEmpty()) {SmokeRequested=true;SmokeStarted=Now;NewWorld();}
    if(!SmokeRequested) return;
    if(Now-SmokeStarted>90) {
        UE_LOG(LogTemp,Error,TEXT("TCE_SMOKE_FAILED save/load did not complete"));
        FPlatformMisc::RequestExitWithStatus(false,1);return;
    }
    const bool Ready=Heights.Num() && People.Num() && Grass->GetInstanceCount()>0 && TerrainCollisionReady();
    if(SmokePhase==0 && Ready) {
        SmokeWorld=WorldId;SmokePeople=People.Num();SmokeSavedMinute=Minute;SmokeEpoch=Session.Epoch;
        flatbuffers::FlatBufferBuilder B;auto Body=w::CreateSaveWorldDirect(B,"Packaged lifecycle check");
        B.Finish(w::CreateCommand(B,w::CommandBody::SaveWorld,Body.Union()));Session.Send(B);
        SmokePhase=1;SmokeStarted=Now;
    }
    if(SmokePhase==1 && Now-SmokeStarted>3) {
        flatbuffers::FlatBufferBuilder B;auto Body=w::CreateListSaves(B);
        B.Finish(w::CreateQuery(B,w::QueryBody::ListSaves,Body.Union()));Session.Send(B,true);
        SmokePhase=2;
    }
    if(SmokePhase==2 && Session.Epoch!=SmokeEpoch && Ready) {
        if(WorldId!=SmokeWorld || People.Num()!=SmokePeople || Minute!=SmokeSavedMinute) {
            UE_LOG(LogTemp,Error,TEXT("TCE_SMOKE_FAILED loaded world differs"));
            FPlatformMisc::RequestExitWithStatus(false,1);return;
        }
        SmokePhase=3;
        UE_LOG(LogTemp,Display,TEXT("TCE_SMOKE_OK people=%d terrain=%dx%d pcg_foliage=%d collision=ok save_load=ok"),People.Num(),Width,Height,Grass->GetInstanceCount());
        GetWorld()->GetFirstPlayerController()->ConsoleCommand(TEXT("quit"));
    }
}

// Replays kernel-produced raster responses only in an explicit diagnostic session. No edits
// enter the live kernel, no fixture is saved as a world, and the original surface is restored.
void AObserverScene::TickTerrainProbe(float Delta) {
    const double Now=FPlatformTime::Seconds();
    auto Fail=[](const TCHAR* Reason) {
        UE_LOG(LogTemp,Error,TEXT("TCE_SPIKE_FAILED %s"),Reason);
        FPlatformMisc::RequestExitWithStatus(false,1);
    };
    if(!SmokeRequested && !Preset.IsEmpty()) {SmokeRequested=true;NewWorld();}
    if(ProbePhase==0 && Heights.Num() && Grass->GetInstanceCount()>0) {
        FString Folder,Metadata;
        if(!FParse::Value(FCommandLine::Get(),TEXT("TCESpikeDir="),Folder) || !FFileHelper::LoadFileToString(Metadata,*FPaths::Combine(Folder,TEXT("metadata.txt")))) {Fail(TEXT("fixture metadata missing"));return;}
        TArray<FString> Values; Metadata.ParseIntoArrayWS(Values);
        if(Values.Num()!=4 || !FFileHelper::LoadFileToArray(ProbeBaseline,*FPaths::Combine(Folder,TEXT("baseline.bin"))) || !FFileHelper::LoadFileToArray(ProbeEdited,*FPaths::Combine(Folder,TEXT("edited.bin")))) {Fail(TEXT("fixture rasters missing"));return;}
        ProbeX=FCString::Atoi(*Values[0]); ProbeY=FCString::Atoi(*Values[1]); ProbeBefore=FCString::Atof(*Values[2]); ProbeAfter=FCString::Atof(*Values[3]);
        if(ProbeX<0 || ProbeY<0 || ProbeX>=Width-1 || ProbeY>=Height-1 || FMath::Abs(Heights[ProbeY*Width+ProbeX]-ProbeBefore)>.02f) {Fail(TEXT("kernel and fixture terrain differ"));return;}
        Focus=FVector(ProbeX*Cell*100+.1,ProbeY*Cell*100+.1,ProbeBefore*100);
        Zoom=18000; ProbeStarted=Now; ProbePhase=1;
        return;
    }
    if(ProbePhase==1 && Now-ProbeStarted>5 && TerrainCollisionReady()) {
        ProbePhase=2; ProbeNextEdit=Now; ProbeStarted=Now;
    }
    if(ProbePhase!=2) return;
    ProbeFrames.Add(Delta*1000.0);
    if(ProbeEdits>0 && Now>ProbeNextEdit-.25 && Now<ProbeNextEdit-.1) {
        if(!TerrainCollisionReady()) {Fail(TEXT("collision did not follow the current patch"));return;}
        int32 Covered=0;
        for(int32 I=0;I<Grass->GetInstanceCount();++I) {
            FTransform Instance; Grass->GetInstanceTransform(I,Instance,false);
            FVector P=Instance.GetLocation();
            if(FMath::Abs(P.Z-12-Ground(P.X/100,P.Y/100)*100)>.1f) {Fail(TEXT("PCG foliage did not follow the current patch"));return;}
            if(P.X>=ProbeX*Cell*100 && P.Y>=ProbeY*Cell*100 && P.X<(ProbeX+2)*Cell*100 && P.Y<(ProbeY+2)*Cell*100) ++Covered;
        }
        if(Covered==0) {Fail(TEXT("no foliage samples covered the edited patch"));return;}
    }
    if(ProbeEdits<20 && Now>=ProbeNextEdit) {
        const bool Changed=ProbeEdits%2==0;
        const auto& Payload=Changed?ProbeEdited:ProbeBaseline;
        commons_wire::FrameMeta Meta; Meta.kind=commons_wire::FrameKind::Response; Meta.schema.major=1;
        FMemory::Memcpy(Meta.schema.tag,"TCE",4); Meta.epoch=Session.Epoch; Meta.correlation=10000+ProbeEdits;
        std::vector<uint8> Bytes;
        if(commons_wire::encode_into(Meta,Payload.GetData(),Payload.Num(),false,Bytes)!=commons_wire::Error::None) {Fail(TEXT("fixture frame invalid"));return;}
        TArray<uint8> Frame; Frame.Append(Bytes.data(),static_cast<int32>(Bytes.size()));
        const double Started=FPlatformTime::Seconds(); ReadReply(Frame);
        ProbePatchTimes.Add((FPlatformTime::Seconds()-Started)*1000);
        const float Expected=Changed?ProbeAfter:ProbeBefore;
        if(FMath::Abs(Heights[ProbeY*Width+ProbeX]-Expected)>.02f || FMath::Abs(Surface->Elevation(ProbeX*Cell,ProbeY*Cell)-Expected)>.02f || LastChangedTiles>4) {Fail(TEXT("local terrain or PCG height mismatch"));return;}
        UE_LOG(LogTemp,Display,TEXT("TCE_SPIKE_PATCH index=%d changed_tiles=%d terrain_ms=%.3f reader_ms=%.3f"),ProbeEdits,LastChangedTiles,LastTerrainBuildMs,ProbePatchTimes.Last());
        ++ProbeEdits; ProbeNextEdit=Now+.5;
    }
    if(ProbeEdits==20 && Now-ProbeStarted>12 && TerrainCollisionReady()) {
        ProbeFrames.Sort(); ProbePatchTimes.Sort();
        auto Percentile=[](const TArray<double>& Values) {return Values[FMath::Min(Values.Num()-1,FMath::FloorToInt(Values.Num()*.99))];};
        double Sum=0; for(double Value:ProbeFrames) Sum+=Value;
        const double P99=Percentile(ProbeFrames),PatchP99=Percentile(ProbePatchTimes);
        UE_LOG(LogTemp,Display,TEXT("TCE_SPIKE_RESULT frames=%d mean_ms=%.3f p99_ms=%.3f patch_p99_ms=%.3f foliage=%d area_km2=%.3f collision=ok restored=ok"),ProbeFrames.Num(),Sum/ProbeFrames.Num(),P99,PatchP99,Grass->GetInstanceCount(),double(MapWidth)*MapHeight*MapCell*MapCell/1000000.0);
        ProbePhase=3;
        if(P99>16.667 || PatchP99>16.667) {Fail(TEXT("60 fps/edit budget not met"));return;}
        UE_LOG(LogTemp,Display,TEXT("TCE_SPIKE_OK"));
        GetWorld()->GetFirstPlayerController()->ConsoleCommand(TEXT("quit"));
    }
}
