#include "ObserverMode.h"
#include "KernelTerrainSurface.h"
#include "Components/InstancedStaticMeshComponent.h"
#include "ProceduralMeshComponent.h"
#include "Elements/PCGSurfaceSampler.h"
#include "Data/PCGBasePointData.h"
#include "Data/PCGPointArrayData.h"

namespace w=tce::wire;

void AObserverScene::BuildFoliage() {
    if(!Surface || !Heights.Num() || WaterClasses.Num()!=Heights.Num()) return;
    const double Started=FPlatformTime::Seconds();
    const FBox Bounds(FVector(FMath::Max(0.,Focus.X-12000),FMath::Max(0.,Focus.Y-12000),Surface->GetBounds().Min.Z),
        FVector(FMath::Min(Surface->GetBounds().Max.X,Focus.X+12000),FMath::Min(Surface->GetBounds().Max.Y,Focus.Y+12000),Surface->GetBounds().Max.Z));
    PCGSurfaceSampler::FSurfaceSamplerParams Params;
    Params.PointsPerSquaredMeter=.035f; Params.PointExtents=FVector(100,100,20); Params.Looseness=.5f;
    auto* Points=PCGSurfaceSampler::SampleSurface(nullptr,Params,Surface,nullptr,Bounds,UPCGPointArrayData::StaticClass());
    Grass->ClearInstances();
    const auto Transforms=Points->GetConstTransformValueRange();
    int32 Count=0;
    for(int32 I=0;I<Points->GetNumPoints();++I) {
        FVector P=Transforms[I].GetLocation(); FVector2D Metres(P.X/100,P.Y/100);
        int32 X=FMath::Clamp(int32(Metres.X/Cell),0,Width-1),Y=FMath::Clamp(int32(Metres.Y/Cell),0,Height-1);
        if(WaterClasses[Y*Width+X]!=0) continue;
        bool Excluded=false;
        for(const FBox2D& Plot:BuildingPlots) if(Plot.ExpandBy(1).IsInside(Metres)) {Excluded=true;break;}
        if(!Excluded) for(const FBox2D& Plot:FieldPlots) if(Plot.IsInside(Metres)) {Excluded=true;break;}
        if(Excluded) continue;
        // Cosmetic ground cover: no resource tree or simulation stock is created here.
        const float Scale=.14f+.06f*FMath::Abs(FMath::Sin(float(I)*17.3f));
        P.Z+=12;
        Grass->AddInstance(FTransform(FRotator(0,I*137.5f,0),P,FVector(Scale,Scale,.24f)));
        ++Count;
    }
    FoliageFocus=Focus; FoliageDirty=false;
    UE_LOG(LogTemp,Display,TEXT("TCE_PCG_SURFACE points=%d foliage=%d ms=%.3f"),Points->GetNumPoints(),Count,(FPlatformTime::Seconds()-Started)*1000);
}

void AObserverScene::ReadEnvironment(const w::Response* Reply) {
    if(auto* Tile=Reply->body_as_RasterTile()) {
        if(Tile->width()!=uint32(RasterWidth) || Tile->height()!=uint32(RasterHeight) || Tile->level()!=RasterLevel || !Tile->data()) return;
        if(Tile->layer()==w::RasterLayer::Water && Tile->data()->size()==uint64(RasterWidth)*RasterHeight) {
            WaterClasses.SetNum(RasterWidth*RasterHeight); FMemory::Memcpy(WaterClasses.GetData(),Tile->data()->data(),WaterClasses.Num());
            FoliageDirty=true; BuildWater();
        }
        if(Tile->layer()==w::RasterLayer::LakeId && Tile->data()->size()==uint64(RasterWidth)*RasterHeight*4) {
            LakeIds.SetNum(RasterWidth*RasterHeight); FMemory::Memcpy(LakeIds.GetData(),Tile->data()->data(),LakeIds.Num()*4); BuildWater();
        }
    }
    if(auto* Hydro=Reply->body_as_Hydrography()) {
        RiverReaches.Reset();
        if(Hydro->reaches()) for(auto* Reach:*Hydro->reaches()) {
            FWaterReach River; River.Width=Reach->width_m();
            if(Reach->points()) for(auto* Point:*Reach->points()) River.Points.Add(FVector2D(Point->x(),Point->y()));
            RiverReaches.Add(MoveTemp(River));
        }
        if(Hydro->lakes()) for(auto* Lake:*Hydro->lakes()) LakeLevels.Add(Lake->id(),Lake->level_m());
        BuildWater();
    }
    if(auto* Fields=Reply->body_as_Fields()) {
        FieldPlots.Reset();
        if(Fields->fields()) for(auto* Field:*Fields->fields()) {
            if(!Field->min() || !Field->size()) continue;
            FVector2D Min(Field->min()->x(),Field->min()->y()),Size(Field->size()->x(),Field->size()->y());
            FieldPlots.Add(FBox2D(Min,Min+Size));
        }
        BuildFields();
        FoliageDirty=true;
    }
}

void AObserverScene::BuildFields() {
    FieldSurfaces->ClearInstances();
    if(!Heights.Num()) return;
    for(const FBox2D& Plot:FieldPlots) for(double X=Plot.Min.X;X<Plot.Max.X;X+=2) for(double Y=Plot.Min.Y;Y<Plot.Max.Y;Y+=2) {
        double SX=FMath::Min(2.,Plot.Max.X-X),SY=FMath::Min(2.,Plot.Max.Y-Y);
        double CX=X+SX/2,CY=Y+SY/2;
        FVector Normal(-(Ground(CX+1,CY)-Ground(CX-1,CY)),-(Ground(CX,CY+1)-Ground(CX,CY-1)),2);
        FQuat Rotation=FQuat::FindBetweenNormals(FVector::UpVector,Normal.GetSafeNormal());
        FieldSurfaces->AddInstance(FTransform(Rotation,FVector(CX*100,CY*100,Ground(CX,CY)*100+3),FVector(SX,SY,1)));
    }
    FieldSurfaces->SetMaterial(0,LoadObject<UMaterialInterface>(nullptr,TEXT("/Game/Kit/Soil.Soil")));
}

void AObserverScene::BuildWater() {
    if(!Heights.Num() || WaterClasses.Num()!=Heights.Num() || LakeIds.Num()!=Heights.Num()) return;
    TArray<FVector> Vertices,Normals; TArray<int32> Indices; TArray<FVector2D> UV;
    for(int32 Y=0;Y<Height-1;++Y) for(int32 X=0;X<Width-1;++X) {
        const int32 Index=Y*Width+X;
        if(WaterClasses[Index]==0 || WaterClasses[Index]==1) continue;
        float Level=WaterClasses[Index]==3?SeaLevel:Heights[Index]+.06f;
        if(WaterClasses[Index]==2) { const float* Lake=LakeLevels.Find(LakeIds[Index]); if(!Lake) continue; Level=*Lake; }
        int32 Base=Vertices.Num();
        for(FIntPoint Corner:{FIntPoint(0,0),FIntPoint(0,1),FIntPoint(1,0),FIntPoint(1,1)}) {
            float PX=(X+Corner.X)*Cell,PY=(Y+Corner.Y)*Cell;
            float Z=WaterClasses[Index]==1?Ground(PX,PY)+.06f:Level;
            Vertices.Add(FVector(PX*100,PY*100,Z*100)); Normals.Add(FVector::UpVector); UV.Add(FVector2D(PX*.05f,PY*.05f));
        }
        Indices.Append({Base,Base+1,Base+2,Base+2,Base+1,Base+3});
    }
    // Continuous ribbons use the kernel's reach centreline and width. They do not carve ground.
    for(const FWaterReach& River:RiverReaches) for(int32 I=1;I<River.Points.Num();++I) {
        FVector2D A=River.Points[I-1],B=River.Points[I];
        FVector2D Direction=(B-A).GetSafeNormal(),Side(-Direction.Y,Direction.X);
        const int32 Segments=FMath::Max(1,FMath::CeilToInt((B-A).Size()/2));
        for(int32 J=0;J<Segments;++J) {
            FVector2D P=FMath::Lerp(A,B,double(J)/Segments),Q=FMath::Lerp(A,B,double(J+1)/Segments);
            int32 Base=Vertices.Num();
            for(FVector2D Centre:{P,Q}) for(float Sign:{-1.f,1.f}) {
                FVector2D Edge=Centre+Side*(River.Width*.5f*Sign);
                Vertices.Add(FVector(Edge.X*100,Edge.Y*100,(Ground(Centre.X,Centre.Y)+.08f)*100));
                Normals.Add(FVector::UpVector); UV.Add(FVector2D(Edge.X*.05f,Edge.Y*.05f));
            }
            Indices.Append({Base,Base+1,Base+2,Base+2,Base+1,Base+3});
        }
    }
    TArray<FLinearColor> Colors; TArray<FProcMeshTangent> Tangents;
    Water->CreateMeshSection_LinearColor(0,Vertices,Indices,Normals,UV,Colors,Tangents,false);
    Water->SetMaterial(0,LoadObject<UMaterialInterface>(nullptr,TEXT("/Game/Kit/Water.Water")));
}
