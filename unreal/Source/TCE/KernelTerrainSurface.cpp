#include "KernelTerrainSurface.h"
#include "Data/PCGPointData.h"
#include "PCGPoint.h"

void UKernelTerrainSurface::Initialize(const TArray<float>& InHeights,int32 InWidth,int32 InHeight,float InCell) {
    Heights=InHeights; Width=InWidth; Height=InHeight; Cell=InCell;
    Transform=FTransform::Identity;
    float Low=TNumericLimits<float>::Max(), High=TNumericLimits<float>::Lowest();
    for(float Value:Heights) { Low=FMath::Min(Low,Value); High=FMath::Max(High,Value); }
    LocalBounds=FBox(FVector(0,0,Low*100),FVector((Width-1)*Cell*100,(Height-1)*Cell*100,High*100));
}
float UKernelTerrainSurface::Elevation(float XMetres,float YMetres) const {
    if(Width<2 || Height<2 || Heights.Num()!=Width*Height) return 0;
    float X=FMath::Clamp(XMetres/Cell,0.f,float(Width-1)),Y=FMath::Clamp(YMetres/Cell,0.f,float(Height-1));
    int32 IX=FMath::Min(int32(X),Width-2),IY=FMath::Min(int32(Y),Height-2);
    return FMath::Lerp(FMath::Lerp(Heights[IY*Width+IX],Heights[IY*Width+IX+1],X-IX),FMath::Lerp(Heights[(IY+1)*Width+IX],Heights[(IY+1)*Width+IX+1],X-IX),Y-IY);
}
bool UKernelTerrainSurface::ProjectPoint(const FTransform& InTransform,const FBox&,const FPCGProjectionParams&,FPCGPoint& OutPoint,UPCGMetadata*) const {
    FVector Position=InTransform.GetLocation();
    if(Position.X<0 || Position.Y<0 || Position.X>LocalBounds.Max.X || Position.Y>LocalBounds.Max.Y) return false;
    Position.Z=Elevation(Position.X/100,Position.Y/100)*100;
    OutPoint=FPCGPoint(FTransform(InTransform.GetRotation(),Position,InTransform.GetScale3D()),1,0);
    return true;
}
bool UKernelTerrainSurface::SamplePoint(const FTransform& InTransform,const FBox& Bounds,FPCGPoint& OutPoint,UPCGMetadata* OutMetadata) const {
    FPCGProjectionParams Params;
    if(!ProjectPoint(InTransform,Bounds,Params,OutPoint,OutMetadata)) return false;
    return Bounds.TransformBy(InTransform).IsInsideOrOn(OutPoint.Transform.GetLocation());
}
UPCGSpatialData* UKernelTerrainSurface::CopyInternal(FPCGContext*) const {
    auto* Copy=NewObject<UKernelTerrainSurface>(); Copy->Initialize(Heights,Width,Height,Cell); return Copy;
}
const UPCGPointData* UKernelTerrainSurface::CreatePointData(FPCGContext*) const {
    // The surface is sampled by PCG's bounded Surface Sampler; do not expand the whole world.
    return NewObject<UPCGPointData>();
}
