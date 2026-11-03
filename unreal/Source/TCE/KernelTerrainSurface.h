#pragma once
#include "CoreMinimal.h"
#include "Data/PCGSurfaceData.h"
#include "KernelTerrainSurface.generated.h"

// A disposable PCG view of the kernel heightfield. It never writes simulation state.
UCLASS()
class UKernelTerrainSurface : public UPCGSurfaceData {
    GENERATED_BODY()
public:
    void Initialize(const TArray<float>& InHeights, int32 InWidth, int32 InHeight, float InCell);
    float Elevation(float XMetres, float YMetres) const;
    virtual FBox GetBounds() const override { return LocalBounds; }
    virtual FBox GetStrictBounds() const override { return LocalBounds; }
    virtual bool HasNonTrivialTransform() const override { return false; }
    virtual bool SamplePoint(const FTransform& InTransform, const FBox& Bounds, FPCGPoint& OutPoint, UPCGMetadata* OutMetadata) const override;
    virtual bool ProjectPoint(const FTransform& InTransform, const FBox& Bounds, const FPCGProjectionParams& Params, FPCGPoint& OutPoint, UPCGMetadata* OutMetadata) const override;
protected:
    virtual UPCGSpatialData* CopyInternal(FPCGContext* Context) const override;
    virtual const UPCGPointData* CreatePointData(FPCGContext* Context) const override;
private:
    UPROPERTY() TArray<float> Heights;
    int32 Width = 0, Height = 0;
    float Cell = 1;
};
