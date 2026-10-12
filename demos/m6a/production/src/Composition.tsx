import {
  AbsoluteFill,
  Composition,
  OffthreadVideo,
  Sequence,
  spring,
  staticFile,
  useCurrentFrame,
  useVideoConfig,
  interpolate,
} from "remotion";

const FPS = 25;
const TITLE_FRAMES = 3 * FPS;
const SOURCE_START = 5 * FPS;
const SOURCE_END = 43 * FPS;
const SOURCE_FRAMES = SOURCE_END - SOURCE_START;
const OUTRO_FRAMES = 3 * FPS;
const WIDTH = 1920;
const HEIGHT = 1080;
const DURATION = TITLE_FRAMES + SOURCE_FRAMES + OUTRO_FRAMES;

const palette = {
  ink: "#101a1d",
  panel: "rgba(12, 20, 23, 0.91)",
  parchment: "#e8dfc9",
  muted: "#c0c5bd",
  gold: "#e0b86b",
  river: "#75b5c4",
};

const FadeIn: React.FC<{ children: React.ReactNode; delay?: number }> = ({ children, delay = 0 }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const reveal = spring({ frame: frame - delay, fps, config: { damping: 18, mass: 0.75 } });
  return <div style={{ opacity: reveal, transform: `translateY(${(1 - reveal) * 22}px)` }}>{children}</div>;
};

const TitleCard: React.FC = () => (
  <AbsoluteFill
    style={{
      background: `radial-gradient(ellipse at 84% 34%, #24383a 0%, ${palette.ink} 56%)`,
      color: palette.parchment,
      padding: "130px 150px",
      fontFamily: "Avenir Next, Inter, -apple-system, BlinkMacSystemFont, sans-serif",
      overflow: "hidden",
    }}
  >
    <svg
      aria-hidden="true"
      viewBox="0 0 780 800"
      style={{ position: "absolute", right: 20, top: 120, width: 800, height: 800, opacity: 0.22 }}
    >
      <path d="M500 -30 C260 130 600 190 330 320 S590 470 200 610 S350 730 20 820" fill="none" stroke={palette.river} strokeWidth="10" />
      <path d="M650 -30 C400 130 740 190 470 320 S730 470 340 610 S490 730 160 820" fill="none" stroke={palette.gold} strokeWidth="2" />
      <path d="M370 -30 C130 130 470 190 200 320 S460 470 70 610" fill="none" stroke={palette.parchment} strokeWidth="2" />
    </svg>
    <div style={{ position: "relative", zIndex: 1, maxWidth: 1380 }}>
      <FadeIn delay={5}>
        <div style={{ color: palette.gold, fontSize: 30, fontWeight: 700, letterSpacing: 7, textTransform: "uppercase" }}>
          The Civilization Engine <span style={{ color: palette.muted }}>· M6a live demo</span>
        </div>
      </FadeIn>
      <FadeIn delay={16}>
        <h1 style={{ fontSize: 104, lineHeight: 1.04, fontWeight: 650, margin: "72px 0 34px", maxWidth: 1200 }}>
          Water, illness, and a village response
        </h1>
      </FadeIn>
      <FadeIn delay={28}>
        <p style={{ fontSize: 44, lineHeight: 1.35, color: palette.muted, maxWidth: 1080, margin: 0 }}>
          One infection enters. Follow its routes and what the village makes of it.
        </p>
      </FadeIn>
      <FadeIn delay={42}>
        <div style={{ marginTop: 90, width: 115, height: 6, background: palette.gold }} />
      </FadeIn>
    </div>
  </AbsoluteFill>
);

const LowerThird: React.FC<{ number: string; title: string; body: string }> = ({ number, title, body }) => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  const opacity = interpolate(frame, [0, 12, durationInFrames - 12, durationInFrames], [0, 1, 1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  return (
    <AbsoluteFill style={{ justifyContent: "flex-end", padding: "0 78px 72px", pointerEvents: "none", opacity }}>
      <div
        style={{
          width: 1310,
          boxSizing: "border-box",
          padding: "32px 44px 34px",
          background: palette.panel,
          borderLeft: `7px solid ${palette.gold}`,
          borderRadius: 4,
          color: palette.parchment,
          fontFamily: "Avenir Next, Inter, -apple-system, BlinkMacSystemFont, sans-serif",
          boxShadow: "0 18px 60px rgba(0,0,0,.24)",
        }}
      >
        <div style={{ display: "flex", gap: 24, alignItems: "center", marginBottom: 9 }}>
          <span style={{ color: palette.gold, fontSize: 29, fontWeight: 750, letterSpacing: 2 }}>{number}</span>
          <h2 style={{ fontSize: 43, lineHeight: 1.1, margin: 0, fontWeight: 700 }}>{title}</h2>
        </div>
        <p style={{ fontSize: 29, lineHeight: 1.3, color: palette.muted, margin: "0 0 0 65px" }}>{body}</p>
      </div>
    </AbsoluteFill>
  );
};

const OutroCard: React.FC = () => (
  <AbsoluteFill
    style={{
      background: `linear-gradient(135deg, ${palette.ink}, #24383a)`,
      color: palette.parchment,
      justifyContent: "center",
      padding: "0 160px",
      fontFamily: "Avenir Next, Inter, -apple-system, BlinkMacSystemFont, sans-serif",
    }}
  >
    <FadeIn delay={5}>
      <div style={{ color: palette.gold, fontSize: 28, fontWeight: 700, letterSpacing: 6, textTransform: "uppercase" }}>
        An emergent history
      </div>
    </FadeIn>
    <FadeIn delay={15}>
      <h2 style={{ fontSize: 82, lineHeight: 1.1, maxWidth: 1420, margin: "34px 0 24px" }}>
        The engine records outcomes. It does not script them.
      </h2>
    </FadeIn>
    <FadeIn delay={30}>
      <p style={{ fontSize: 36, color: palette.muted, margin: 0 }}>One run of The Civilization Engine · outcomes vary by world</p>
    </FadeIn>
  </AbsoluteFill>
);

const M6aFilm: React.FC = () => (
  <AbsoluteFill style={{ backgroundColor: palette.ink }}>
    <Sequence durationInFrames={TITLE_FRAMES}>
      <TitleCard />
    </Sequence>
    <Sequence from={TITLE_FRAMES} durationInFrames={SOURCE_FRAMES}>
      <OffthreadVideo
        src={staticFile("m6a-source.webm")}
        startFrom={SOURCE_START}
        endAt={SOURCE_END}
        style={{ width: WIDTH, height: HEIGHT, objectFit: "contain" }}
      />
    </Sequence>
    <Sequence from={TITLE_FRAMES} durationInFrames={5 * FPS}>
      <LowerThird number="01" title="A village by the water" body="The observer introduces one illness. Daily life continues around it." />
    </Sequence>
    <Sequence from={TITLE_FRAMES + 5 * FPS} durationInFrames={20 * FPS}>
      <LowerThird number="02" title="Follow the outbreak" body="Compare recorded routes with residents' knowledge and what their polity records." />
    </Sequence>
    <Sequence from={TITLE_FRAMES + 25 * FPS} durationInFrames={13 * FPS}>
      <LowerThird number="03" title="Where households drew water" body="The Water lens summarizes each household's recorded water use over 30 days." />
    </Sequence>
    <Sequence from={TITLE_FRAMES + SOURCE_FRAMES} durationInFrames={OUTRO_FRAMES}>
      <OutroCard />
    </Sequence>
  </AbsoluteFill>
);

export const MyComposition = () => (
  <Composition
    id="M6aShowcase"
    component={M6aFilm}
    durationInFrames={DURATION}
    fps={FPS}
    width={WIDTH}
    height={HEIGHT}
  />
);
