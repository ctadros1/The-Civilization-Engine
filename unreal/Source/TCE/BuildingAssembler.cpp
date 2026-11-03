#include "ObserverMode.h"
#include "Components/InstancedStaticMeshComponent.h"
#include "Components/PointLightComponent.h"

namespace w=tce::wire;

void AObserverScene::BuildBuildings(const w::Buildings* Data) {
    BuildingQueryPending=false;
    for(auto* Pool:{Posts.Get(),Walls.Get(),Roofs.Get(),Beams.Get(),Wattle.Get(),Floors.Get(),Hearths.Get(),Scaffolds.Get(),RoofWedges.Get()}) Pool->ClearInstances();
    BuildingPlots.Reset(); HearthPresentations.Reset(); FoliageDirty=true;
    if(!Data->buildings()) return;
    int32 PartCount=0;
    for(auto* Building:*Data->buildings()) {
        if(!Building->centre() || Building->state()==w::BuildingState::Ruin) continue;
        if(Building->plot_min() && Building->plot_size()) BuildingPlots.Add(FBox2D(FVector2D(Building->plot_min()->x(),Building->plot_min()->y()),FVector2D(Building->plot_min()->x()+Building->plot_size()->x(),Building->plot_min()->y()+Building->plot_size()->y())));
        const auto* Parts=Building->parts();
        if(!Parts) { Caption=TEXT("This client needs kernel wire 1.107 or later for building parts."); continue; }
        const float Base=Ground(Building->centre()->x(),Building->centre()->y())*100;
        int32 Totals[5]{},Seen[5]{};
        float Eaves=0;
        for(auto* Part:*Parts) {
            if(Part->stage()<5) ++Totals[Part->stage()];
            if(Part->kind()==1 && Part->at() && Part->size()) Eaves=FMath::Max(Eaves,float(Part->at()->z()+Part->size()->z()/2));
        }
        const bool Round=Building->grammar() && Building->grammar()->str()=="hut";
        for(auto* Part:*Parts) {
            if(!Part->at() || !Part->size() || Part->stage()>=5) continue;
            int32 Ordinal=Seen[Part->stage()]++;
            const bool Current=Part->stage()==Building->stage();
            const bool Done=Part->stage()<Building->stage();
            const float Progress=FMath::Clamp(Building->progress(),0.f,1.f);
            const bool Visible=Done || (Current && Ordinal<FMath::FloorToInt(Totals[Part->stage()]*Progress));
            FVector Position(Part->at()->x(),Part->at()->y(),Base+Part->at()->z());
            FVector Scale(Part->size()->x()/100.f,Part->size()->y()/100.f,Part->size()->z()/100.f);
            FRotator Rotation(Part->tilt()/100.f,Part->angle()*360.f/65536.f,0);
            if(Part->kind()==5 && Round) {
                Rotation.Pitch=0;
                if(Done) { Roofs->AddInstance(FTransform(Rotation,Position,Scale)); ++PartCount; }
                else if(Current) for(int32 Wedge=0;Wedge<FMath::FloorToInt(Progress*16);++Wedge) {
                    FRotator Turn=Rotation; Turn.Yaw+=Wedge*22.5f;
                    RoofWedges->AddInstance(FTransform(Turn,Position,Scale)); ++PartCount;
                }
                continue;
            }
            if(!Visible) continue;
            UInstancedStaticMeshComponent* Target=nullptr;
            switch(Part->kind()) {
                case 0: Target=Floors; Position.Z=Base+.5f; Scale.Z=1; break;
                case 1: case 15: Target=Posts; Rotation.Pitch=0; break;
                case 3: case 13: Target=Wattle; break;
                case 4: case 17: Target=Walls; break;
                case 5: case 18: Target=Walls; break;
                case 6: Target=Floors; if(Scale.Z==0) Scale.Z=1; break;
                case 7: Target=Hearths; break;
                case 20: break; // An opening is absence of wall, not a solid cube.
                default: Target=Beams; break;
            }
            if(Target) { Target->AddInstance(FTransform(Rotation,Position,Scale)); ++PartCount; }
            if(Part->kind()==7 && Building->door()) {
                FHearthPresentation Light;
                Light.Household=Building->household(); Light.Hearth=Position+FVector(0,0,25);
                Light.Opening=FVector(Building->door()->x()*100,Building->door()->y()*100,Base+Eaves*.4f);
                Light.Direction=FMath::RadiansToDegrees(Building->door_dir()); HearthPresentations.Add(Light);
            }
        }
        // Timber working platforms follow the active wall/roof work, not a second construction state.
        if((Building->stage()==2 || Building->stage()==3) && Building->outline()) {
            const auto* Outline=Building->outline();
            for(uint32 Index=0;Index<Outline->size();Index+=4) {
                const auto* Point=Outline->Get(Index);
                FVector Radial(Point->x()-Building->centre()->x(),Point->y()-Building->centre()->y(),0);
                FVector Foot(Point->x()*100,Point->y()*100,Base); Foot+=Radial.GetSafeNormal()*65;
                const float WorkHeight=FMath::Max(100.f,Eaves*.75f);
                Scaffolds->AddInstance(FTransform(FRotator(90,0,0),Foot+FVector(0,0,WorkHeight/2),FVector(WorkHeight/100,.075,.075)));
                Scaffolds->AddInstance(FTransform(FRotator(0,Radial.Rotation().Yaw+90,0),Foot+FVector(0,0,WorkHeight),FVector(1.4,.12,.08)));
            }
        }
    }
    UE_LOG(LogTemp,Display,TEXT("TCE_BUILDING_PARTS rendered=%d buildings=%d"),PartCount,Data->buildings()->size());
}

void AObserverScene::DrawHearthLight(float Hour) {
    const bool Night=Hour<6 || Hour>=18;
    if(HearthLights.Num()==0) for(int32 I=0;I<8;++I) {
        auto* Light=NewObject<UPointLightComponent>(this);
        Light->SetupAttachment(RootComponent); Light->SetMobility(EComponentMobility::Movable);
        Light->SetCastShadows(false); Light->SetIntensity(180); Light->SetAttenuationRadius(600);
        Light->SetLightColor(FLinearColor(1,.34f,.07f)); Light->RegisterComponent(); HearthLights.Add(Light);
    }
    TArray<FTransform> Coal,Openings; int32 Active=0;
    for(const FHearthPresentation& Hearth:HearthPresentations) {
        if(!Night || HouseholdFuel.FindRef(Hearth.Household)<=0) continue;
        Coal.Add(FTransform(FRotator::ZeroRotator,Hearth.Hearth,FVector(.25,.25,.08)));
        const FRotator Facing(0,Hearth.Direction,0);
        Openings.Add(FTransform(FRotator(0,Hearth.Direction-90,0),Hearth.Opening-Facing.Vector()*10,FVector(.55,.015,.7)));
        if(Active<HearthLights.Num() && FVector::DistSquared2D(Hearth.Hearth,Focus)<FMath::Square(15000.)) {
            HearthLights[Active]->SetWorldLocation(Hearth.Hearth); HearthLights[Active++]->SetVisibility(true);
        }
    }
    for(int32 I=Active;I<HearthLights.Num();++I) HearthLights[I]->SetVisibility(false);
    auto Update=[](UInstancedStaticMeshComponent* Pool,const TArray<FTransform>& Values) {
        if(Pool->GetInstanceCount()!=Values.Num()) { Pool->ClearInstances(); Pool->AddInstances(Values,false,false,false); }
        else if(Values.Num()) Pool->BatchUpdateInstancesTransforms(0,Values,false,true,true);
    };
    Update(Embers,Coal); Update(WindowGlows,Openings);
    auto* Glow=LoadObject<UMaterialInterface>(nullptr,TEXT("/Game/Kit/HearthGlow.HearthGlow"));
    Embers->SetMaterial(0,Glow); WindowGlows->SetMaterial(0,Glow);
}
