#include "ObserverMode.h"
#include "Engine/World.h"

bool AObserverScene::TerrainCollisionReady() const {
    if(!Heights.Num()) return false;
    const float X=FMath::FloorToFloat(Focus.X/(Cell*100))*Cell*100+.1f;
    const float Y=FMath::FloorToFloat(Focus.Y/(Cell*100))*Cell*100+.1f;
    const float Expected=Ground(X/100,Y/100)*100;
    FHitResult Hit;
    if(!GetWorld()->LineTraceSingleByChannel(Hit,FVector(X,Y,Expected+10000),FVector(X,Y,Expected-10000),ECC_Visibility)) return false;
    return Hit.GetComponent() && Hit.GetComponent()->ComponentHasTag(TEXT("TCE_Terrain")) && FMath::Abs(Hit.ImpactPoint.Z-Expected)<2;
}
