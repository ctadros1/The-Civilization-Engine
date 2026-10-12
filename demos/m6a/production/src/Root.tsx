import { MyComposition } from "./Composition";
import { ArchiveCompositions } from "./ArchiveCompositions";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <MyComposition />
      <ArchiveCompositions />
    </>
  );
};
