import {
  AbsoluteFill,
  Composition,
  OffthreadVideo,
  Sequence,
  interpolate,
  spring,
  staticFile,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";

const FPS = 25;
const WIDTH = 1920;
const HEIGHT = 1080;
const TITLE = 3 * FPS;
const OUTRO = 3 * FPS;
const colors = { ink: "#101a1d", parchment: "#e8dfc9", muted: "#c0c5bd", gold: "#e0b86b", river: "#75b5c4" };

type Film = {
  id: string;
  source: string;
  sourceStart: number;
  sourceFrames: number;
  title: string;
  subtitle: string;
  edition: string;
  chapters: { from: number; duration: number; number: string; title: string; body: string }[];
};

const films: Film[] = [
  {
    id: "M3aEconomy",
    source: "m3a-source.webm",
    sourceStart: 100,
    sourceFrames: 1064,
    title: "Two ways to hold the land",
    subtitle: "Compare household fields and village fields in the same valley.",
    edition: "M3a · VILLAGE ECONOMY",
    chapters: [
      { from: 0, duration: 20 * FPS, number: "01", title: "Household fields", body: "Follow household wealth and market activity as people farm and trade." },
      { from: 20 * FPS, duration: 22 * FPS, number: "02", title: "Village fields", body: "The same seed with shared land tenure. Compare markets and household wealth." },
    ],
  },
  {
    id: "M3bKnowledge",
    source: "m3b-source.webm",
    sourceStart: 75,
    sourceFrames: 950,
    title: "How a craft enters a village",
    subtitle: "Follow a technique from the observer's hand into village knowledge and building.",
    edition: "M3b · KNOWLEDGE & BUILDINGS",
    chapters: [
      { from: 0, duration: 16 * FPS, number: "01", title: "Framing enters the valley", body: "The observer teaches jointed timber framing to one founder." },
      { from: 16 * FPS, duration: 22 * FPS, number: "02", title: "Knowledge and buildings", body: "Inspect who knows the craft and how the village's homes were built." },
    ],
  },
  {
    id: "M3cWeather",
    source: "m3c-source.webm",
    sourceStart: 75,
    sourceFrames: 1200,
    title: "A village through the seasons",
    subtitle: "Compare lived weather, harvests and market conditions across years.",
    edition: "M3c · SEASONS & TIME",
    chapters: [
      { from: 0, duration: 18 * FPS, number: "01", title: "Read the years", body: "Weather records put this valley's dry and wet years beside its usual pattern." },
      { from: 18 * FPS, duration: 30 * FPS, number: "02", title: "Advance the simulation", body: "Watch the clock move at accelerated speed as the village continues to change." },
    ],
  },
  {
    id: "M4aGovernment",
    source: "m4a-source.webm",
    sourceStart: 75,
    sourceFrames: 455,
    title: "How a village makes rules",
    subtitle: "See proposals, offices, laws and notable standing emerge in a lived community.",
    edition: "M4a · GOVERNMENT",
    chapters: [
      { from: 0, duration: 9 * FPS, number: "01", title: "A gathering takes shape", body: "The observer reports the polity's name, customs, offices and proposals." },
      { from: 9 * FPS, duration: 9.2 * FPS, number: "02", title: "Read the decisions", body: "Open a law's history and inspect the people who hold standing." },
    ],
  },
  {
    id: "M4bJustice",
    source: "m4b-source.webm",
    sourceStart: 75,
    sourceFrames: 485,
    title: "From taking to public account",
    subtitle: "Compare what happened, what residents believed and what a gathering heard.",
    edition: "M4b · INCIDENTS & JUSTICE",
    chapters: [
      { from: 0, duration: 243, number: "01", title: "An incident has layers", body: "The observer keeps events, local knowledge and public accounts distinct." },
      { from: 243, duration: 242, number: "02", title: "Follow the case", body: "Open a case to see its reported steps and any law or decision that followed." },
    ],
  },
  {
    id: "M5aNeighbours",
    source: "m5a-source.webm",
    sourceStart: 75,
    sourceFrames: 1200,
    title: "Three settlements, one changing world",
    subtitle: "Trace migration, settlement contact and resident histories across thirty simulated years.",
    edition: "M5a · NEIGHBOURS",
    chapters: [
      { from: 0, duration: 20 * FPS, number: "01", title: "Year ten: the migration snapshot", body: "See the settlement list after the wave arrived and compare how groups were founded." },
      { from: 20 * FPS, duration: 28 * FPS, number: "02", title: "Year thirty: histories on record", body: "Compare population and contact, then inspect residents' recorded histories." },
    ],
  },
  {
    id: "M0Foundations",
    source: "m0-source.webm",
    sourceStart: 300,
    sourceFrames: 148,
    title: "Make a world",
    subtitle: "Generate a valley, then inspect the terrain and rivers it contains.",
    edition: "M0 · FOUNDATIONS",
    chapters: [
      { from: 0, duration: 2 * FPS, number: "01", title: "A seeded valley is ready", body: "The kernel has generated its terrain and waterways." },
      { from: 2 * FPS, duration: 4 * FPS, number: "02", title: "Inspect relief and water", body: "The map shows the valley floor, ridges and river network." },
    ],
  },
  {
    id: "M0Recovery",
    source: "m0-recovery-source.webm",
    sourceStart: 25,
    sourceFrames: 57,
    title: "Recover an interrupted world",
    subtitle: "After an unclean shutdown, the observer offers and restores the saved world.",
    edition: "M0 · SAVE RECOVERY",
    chapters: [
      { from: 0, duration: 3 * FPS, number: "01", title: "Recover the saved world", body: "The observer finds the last autosave, then reloads its map and state." },
    ],
  },
  {
    id: "M1Village",
    source: "m1-source.webm",
    sourceStart: 0,
    sourceFrames: 2688,
    title: "A band settles",
    subtitle: "Follow a founding band as a village takes shape over ten simulated years.",
    edition: "M1 · VILLAGE LIFE",
    chapters: [
      { from: 0, duration: 30 * FPS, number: "01", title: "Found a settlement", body: "A band arrives at the river and establishes its first camp." },
      { from: 30 * FPS, duration: 48 * FPS, number: "02", title: "Daily life leaves a mark", body: "Households gather food, build, farm, raise families and expand the village." },
      { from: 75 * FPS, duration: 32 * FPS, number: "03", title: "Ten years later", body: "The run saves and reloads the world at its current state." },
    ],
  },
];

const FadeIn: React.FC<{ children: React.ReactNode; delay?: number }> = ({ children, delay = 0 }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const reveal = spring({ frame: frame - delay, fps, config: { damping: 18, mass: 0.75 } });
  return <div style={{ opacity: reveal, transform: `translateY(${(1 - reveal) * 22}px)` }}>{children}</div>;
};

const TitleCard: React.FC<{ film: Film }> = ({ film }) => (
  <AbsoluteFill style={{ background: `radial-gradient(ellipse at 84% 34%, #24383a 0%, ${colors.ink} 56%)`, color: colors.parchment, padding: "130px 150px", fontFamily: "Avenir Next, Inter, -apple-system, BlinkMacSystemFont, sans-serif", overflow: "hidden" }}>
    <svg aria-hidden="true" viewBox="0 0 780 800" style={{ position: "absolute", right: 20, top: 120, width: 800, height: 800, opacity: 0.22 }}>
      <path d="M500 -30 C260 130 600 190 330 320 S590 470 200 610 S350 730 20 820" fill="none" stroke={colors.river} strokeWidth="10" />
      <path d="M650 -30 C400 130 740 190 470 320 S730 470 340 610 S490 730 160 820" fill="none" stroke={colors.gold} strokeWidth="2" />
      <path d="M370 -30 C130 130 470 190 200 320 S460 470 70 610" fill="none" stroke={colors.parchment} strokeWidth="2" />
    </svg>
    <div style={{ position: "relative", zIndex: 1, maxWidth: 1400 }}>
      <FadeIn delay={5}><div style={{ color: colors.gold, fontSize: 30, fontWeight: 700, letterSpacing: 7, textTransform: "uppercase" }}>The Civilization Engine · {film.edition}</div></FadeIn>
      <FadeIn delay={16}><h1 style={{ fontSize: 104, lineHeight: 1.04, fontWeight: 650, margin: "72px 0 34px", maxWidth: 1300 }}>{film.title}</h1></FadeIn>
      <FadeIn delay={28}><p style={{ fontSize: 44, lineHeight: 1.35, color: colors.muted, maxWidth: 1160, margin: 0 }}>{film.subtitle}</p></FadeIn>
      <FadeIn delay={42}><div style={{ marginTop: 90, width: 115, height: 6, background: colors.gold }} /></FadeIn>
    </div>
  </AbsoluteFill>
);

const LowerThird: React.FC<{ chapter: Film["chapters"][number] }> = ({ chapter }) => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  const opacity = interpolate(frame, [0, 12, durationInFrames - 12, durationInFrames], [0, 1, 1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  return <AbsoluteFill style={{ justifyContent: "flex-end", padding: "0 78px 72px", pointerEvents: "none", opacity }}>
    <div style={{ width: 1310, boxSizing: "border-box", padding: "32px 44px 34px", background: "rgba(12, 20, 23, 0.91)", borderLeft: `7px solid ${colors.gold}`, borderRadius: 4, color: colors.parchment, fontFamily: "Avenir Next, Inter, -apple-system, BlinkMacSystemFont, sans-serif", boxShadow: "0 18px 60px rgba(0,0,0,.24)" }}>
      <div style={{ display: "flex", gap: 24, alignItems: "center", marginBottom: 9 }}><span style={{ color: colors.gold, fontSize: 29, fontWeight: 750, letterSpacing: 2 }}>{chapter.number}</span><h2 style={{ fontSize: 43, lineHeight: 1.1, margin: 0, fontWeight: 700 }}>{chapter.title}</h2></div>
      <p style={{ fontSize: 29, lineHeight: 1.3, color: colors.muted, margin: "0 0 0 65px" }}>{chapter.body}</p>
    </div>
  </AbsoluteFill>;
};

const OutroCard: React.FC<{ film: Film }> = ({ film }) => (
  <AbsoluteFill style={{ background: `linear-gradient(135deg, ${colors.ink}, #24383a)`, color: colors.parchment, justifyContent: "center", padding: "0 160px", fontFamily: "Avenir Next, Inter, -apple-system, BlinkMacSystemFont, sans-serif" }}>
    <FadeIn delay={5}><div style={{ color: colors.gold, fontSize: 28, fontWeight: 700, letterSpacing: 6, textTransform: "uppercase" }}>The Civilization Engine · {film.edition}</div></FadeIn>
    <FadeIn delay={15}><h2 style={{ fontSize: 76, lineHeight: 1.1, maxWidth: 1460, margin: "34px 0 24px" }}>{film.title}</h2></FadeIn>
    <FadeIn delay={30}><p style={{ fontSize: 34, color: colors.muted, margin: 0 }}>A recorded observer session · outcomes and details vary by world</p></FadeIn>
  </AbsoluteFill>
);

const FilmComposition: React.FC<{ film: Film }> = ({ film }) => (
  <AbsoluteFill style={{ backgroundColor: colors.ink }}>
    <Sequence durationInFrames={TITLE}><TitleCard film={film} /></Sequence>
    <Sequence from={TITLE} durationInFrames={film.sourceFrames}>
      <OffthreadVideo src={staticFile(film.source)} startFrom={film.sourceStart} style={{ width: WIDTH, height: HEIGHT, objectFit: "contain" }} />
    </Sequence>
    {film.chapters.map((chapter) => <Sequence key={chapter.number} from={TITLE + chapter.from} durationInFrames={chapter.duration}><LowerThird chapter={chapter} /></Sequence>)}
    <Sequence from={TITLE + film.sourceFrames} durationInFrames={OUTRO}><OutroCard film={film} /></Sequence>
  </AbsoluteFill>
);

export const ArchiveCompositions: React.FC = () => <>{films.map((film) => <Composition key={film.id} id={film.id} component={FilmComposition} durationInFrames={TITLE + film.sourceFrames + OUTRO} fps={FPS} width={WIDTH} height={HEIGHT} defaultProps={{ film }} />)}</>;
