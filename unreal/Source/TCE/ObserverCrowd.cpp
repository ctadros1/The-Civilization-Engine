#include "ObserverMode.h"
#include "Components/InstancedStaticMeshComponent.h"
#include "timed_path.hpp"
#include "GameFramework/PlayerController.h"
#include "Engine/World.h"
#include "SWebBrowser.h"

namespace w=tce::wire;

void AObserverScene::RefreshCarriedGoods() {
    const double Now=FPlatformTime::Seconds();
    if(Now-LastCarryQuery<.5) return;
    LastCarryQuery=Now;
    int32 Requested=0;
    for(const FWalker& Person:People) {
        if(Requested>=64) break;
        if(FVector2D::DistSquared(Person.Pos,FVector2D(Focus.X/100,Focus.Y/100))>FMath::Square(180.)) continue;
        flatbuffers::FlatBufferBuilder B; auto Body=w::CreateGetPerson(B,Person.Id,0);
        B.Finish(w::CreateQuery(B,w::QueryBody::GetPerson,Body.Union())); Session.Send(B,true); ++Requested;
    }
}

void AObserverScene::DrawCrowd(double RenderMinute) {
    TArray<FTransform> Bodies,HeadTransforms,LimbTransforms,LoadTransforms;
    for(const FWalker& Person:People) {
        FVector2D Pos=Person.Pos;
        if(!Still) if(const auto* Path=Trips.Find(Person.Trip)) if(Path->Rev==Person.Rev) {
            auto Point=engine_bridge::position_on_path(Path->XY.GetData(),Path->Minutes.GetData(),Path->Minutes.Num(),Path->Depart,RenderMinute);
            Pos=FVector2D(Point.x,Point.y);
        }
        FVector Base(Pos.X*100,Pos.Y*100,Ground(Pos.X,Pos.Y)*100);
        if(Person.Id==FollowId) Focus=Base;
        FVector Previous=LastPositions.FindRef(Person.Id);
        FVector Direction=(Base-Previous).GetSafeNormal2D();
        float FacingYaw=LastFacing.FindRef(Person.Id);
        if(!Direction.IsNearlyZero() && !Previous.IsNearlyZero()) FacingYaw=Direction.Rotation().Yaw;
        LastFacing.Add(Person.Id,FacingYaw);
        LastPositions.Add(Person.Id,Base);
        float Size=FMath::Clamp(Person.Age/16.f,.45f,1.f);
        FRotator Facing(0,FacingYaw,0);
        const bool Walking=Person.Trip && !Still;
        const float Swing=Walking?FMath::Sin(float(RenderMinute*15+Person.Id%19))*.4f:0;
        auto Part=[&](TArray<FTransform>& Target,FVector Offset,FVector Scale,FRotator Rotation=FRotator::ZeroRotator) {
            Target.Add(FTransform((Facing+Rotation).Quaternion(),Base+Facing.RotateVector(Offset*Size),Scale*Size));
        };
        Part(Bodies,FVector(0,0,110),FVector(.42,.28,.65));
        if(FVector::DistSquared2D(Base,Focus)<FMath::Square(25000.)) {
            Part(HeadTransforms,FVector(0,0,158),FVector(.25));
            Part(LimbTransforms,FVector(0,-12,43),FVector(.13,.13,.78),FRotator(Swing*45,0,0));
            Part(LimbTransforms,FVector(0,12,43),FVector(.13,.13,.78),FRotator(-Swing*45,0,0));
            Part(LimbTransforms,FVector(0,-27,106),FVector(.11,.11,.64),FRotator(-Swing*35,0,0));
            Part(LimbTransforms,FVector(0,27,106),FVector(.11,.11,.64),FRotator(Swing*35,0,0));
            if(const float* Carry=CarriedKg.Find(Person.Id)) if(*Carry>.01f) {
                float BundleWidth=FMath::Clamp(FMath::Pow(*Carry,.333f)*.12f,.13f,.42f);
                Part(LoadTransforms,FVector(30,0,100),FVector(BundleWidth,BundleWidth,.3));
            }
        }
    }
    auto Update=[](UInstancedStaticMeshComponent* Pool,const TArray<FTransform>& Transforms) {
        if(Pool->GetInstanceCount()!=Transforms.Num()) {
            Pool->ClearInstances(); Pool->AddInstances(Transforms,false,false,false);
        } else if(Transforms.Num()) Pool->BatchUpdateInstancesTransforms(0,Transforms,false,true,true);
    };
    Update(Citizens,Bodies); Update(Heads,HeadTransforms); Update(Limbs,LimbTransforms); Update(Loads,LoadTransforms);
}

void AObserverScene::SelectPersonInPanels(uint64 Id) {
    SelectedId=Id;
    if(Browser.IsValid() && Id) Browser->ExecuteJavascript(FString::Printf(TEXT("window.__TCE__?.select(%llu);"),Id));
}

void AObserverScene::PickPerson() {
    auto* PC=GetWorld()->GetFirstPlayerController();
    float MouseX=0,MouseY=0;
    if(!PC || !PC->GetMousePosition(MouseX,MouseY)) return;
    uint64 Closest=0; double Best=FMath::Square(22.);
    for(const FWalker& Person:People) if(const FVector* Position=LastPositions.Find(Person.Id)) {
        FVector2D Screen;
        if(!PC->ProjectWorldLocationToScreen(*Position+FVector(0,0,90),Screen)) continue;
        double Distance=FVector2D::DistSquared(Screen,FVector2D(MouseX,MouseY));
        if(Distance<Best) {Best=Distance;Closest=Person.Id;}
    }
    if(Closest) SelectPersonInPanels(Closest);
}
