# Mixing (electronic music)

Verified 2026-11-24. Sources: [sources.md](sources.md). Lines marked **(to verify)** have no checked source and must
not be stated to the user as fact; lines marked *convention* are common practice.

## 1. Gain staging

- **dBFS** = level relative to digital full scale (0 dBFS = maximum sample value). A fixed-point file clips above
  it; 32/64-bit floating-point DAW mixers do not clip internally, but anything sent to the interface outputs or
  bounced to fixed point must stay below 0 dBFS, with "a dB or two in hand" for true-peak inaccuracy [S12].
- **Convention:** many engineers calibrate **0 VU = −18 dBFS** (−16 and −20 dBFS are also used), metering average
  (RMS/VU) rather than peaks [S12]. Example practice: VU meters on every channel at −18 dBFS reference, peak LED at
  −6 dBFS [S12].
- **Why it still matters in float:** meaningful meters, fair bypass comparisons, and analogue-modelled plug-ins or
  plug-ins with fixed thresholds respond as designed [S12]. Pick one reference and stay consistent [S12].
- **COSOUND rule:** report measured channel/bus peak and RMS/short-term loudness; suggest trims in dB, never
  "turn it down a bit".

## 2. EQ

- **Types** [S13]: peaking/bell (boost or cut around a centre frequency; narrow notches e.g. hum at 50 Hz and
  harmonics), shelving (everything above/below a corner), high-pass and low-pass filters (remove unused lows/highs).
- **Q / bandwidth:** narrow for localised cuts, wide for gentle boosts; ~Q 3–4 for a sweeping boost to *find* a
  problem; high-Q bells can "ring" [S13].
- **Practice** [S13]: as little EQ as possible, prefer cuts, keep boosts to a few dB, judge in the mix not solo,
  check with bypass; analogue-style EQ shifts phase even at small boosts.
- **Frequency guide** (SOS, mostly acoustic sources) [S13]:

| Source | Ranges |
|---|---|
| Kick | punch 80–100 Hz; warmth/"box" 200–300 Hz; beater click 2.5–6 kHz |
| Snare / clap (by analogy) | body 120–400 Hz; boxiness 800 Hz–1.2 kHz; ring 2–4 kHz; crispness 4–8 kHz |
| Cymbals / hats | clunk 100–300 Hz; ring 1–6 kHz; sizzle 8–12 kHz |
| Bass | weight 80–100 Hz; warmth 100–300 Hz; attack 500–1500 Hz; noise 2–5 kHz |
| Vocals | plosives < ~100 Hz; nasal 1–3 kHz; presence 4–5 kHz; sibilance 7–12 kHz |

- **Electronic additions:** techno kick – boost the fundamental 1–3 dB (find it from the key), shelf below ~30 Hz
  [S23]; clap high-passed ~140 Hz, chords/keys low-cut ~150 Hz in a progressive-house example [S27]; pads high-passed ~142 Hz to reduce mud in an afro-house example
  [S28]. Secondary bass layers and full-range synths/pianos can be high-passed
  ~100 Hz; effect returns high-passed to keep the sub region clean [S14].

## 3. Compression

- **Threshold** – level above which gain reduction starts [S17][S19].
- **Ratio** – input-over-threshold : output-over-threshold, e.g. 4:1 → 4 dB over gives 1 dB over [S17]; at ∞:1 the
  compressor behaves as a limiter [S19]. Transient material often 4:1–8:1 [S17].
- **Knee** – roundness around the threshold; Pro-C 3: 0 dB (hard) to 72 dB (soft); soft knee is more gradual [S19].
- **Attack** – how fast gain reduction engages (Pro-C 3: 0.005–250 ms); start slow so transients pass, then
  shorten by ear [S17][S20].
- **Release** – how fast gain returns to unity; set so it recovers ~90–100 % before the next hit; often tied to
  tempo [S17]. Program-dependent/auto release exists in many units [S20].
- **Range** – caps the maximum gain change [S19]. **Lookahead** – anticipates peaks, adds latency (Pro-C 3 up to
  20 ms) [S20]. **Hold** – longer hold can create pumping [S20].
- **Make-up gain** – restores level lost to gain reduction; Pro-C 3 "Auto Gain" computes it from threshold, ratio,
  knee and attack (search-snippet of the same manual; **to verify** on the page) [S19].
- **Parallel compression** – blend a heavily compressed copy with the dry signal (bus routing or a dry/wet
  control) **(definition to verify – FabFilter page "bus, parallel and mix bus" not fetched)**.
- **COSOUND rule:** give settings as starting points *plus* the measured reason (e.g. crest factor of the stem).

## 4. Sidechain (kick → bass ducking and volume shaping)

- **Sidechain filter:** EQ on the detector only (not the audio) – e.g. high-pass so a kick's lows don't trigger
  bus compression [S18].
- **External sidechain ducking:** compressor on the bass, kick feeds the detector; each kick briefly lowers the
  bass; common in dance music [S18]. Ducking the bass **2–3 dB** frees headroom; more starts to sound unnatural
  (unless pumping is the aesthetic) [S14]. Prefer a compressor on the bass channel over ducking the whole mix [S14].
- **Group ducking:** NI deep-house example sidechains a group of all non-drum elements from the kick
  [S29].
- **Silent trigger / volume shaping:** a muted trigger signal (or LFO/volume-shaper tools) can create precise
  envelopes, e.g. on pads [S18].

## 5. Phase, polarity and mono compatibility

- **Correlation meter** [S15]: +1 = identical L/R (mono); 0 = uncorrelated (widest); −1 = one side
  polarity-inverted. Normal stereo sits between +1 and 0; brief negative dips are usually harmless, sustained
  negative readings mean content will be lost in mono.
- **Polarity vs phase:** polarity inversion is a 180° flip at all frequencies; phase shift is frequency- and
  time-dependent (EQ, delays) **(general definition – SOS treats inversion as the 180° case [S15]; EQ adds phase shift [S14]; full distinction to verify)**.
- **Aligning layered bass/kick** [S14]: check polarity first, then timing; flip one layer, match levels, nudge for
  deepest cancellation, flip back; re-check after EQ (EQ adds phase shift).

## 6. Stereo width and low end

- **Sub mono convention:** keep kick, sub and main bass mono; content below ≈ **100 Hz** almost always mono;
  widening above ≈ **100–150 Hz** acceptable if the mono fold-down still works [S21].
- **Why:** club/PA systems often sum lows to mono; stereo synth basses with phase differences lose level there;
  excessive stereo bass makes a vinyl cutting head jump [S14][S21].
- **Fixes:** M/S EQ or mid/side utility to reduce side content in the lows; high-pass (~100 Hz) the widening aux;
  layer a mono sub under a filtered stereo bass [S14][S21].

## 7. Kick–bass relationship

- Kick and bass are the main headroom bottleneck at mix and master [S14].
- Separate them in **time** (off-beat bass, rolling bass resting on the kick 16th) or in **frequency** (bass owns
  the sub, kick owns the attack/punch) [S14][S23].
- Tune the kick to the key; pick kicks pitched to the track rather than heavy re-pitching
  [S30]; shifting kick pitch can move its peaks into the bass's gaps [S14].
- Techno recipe: off-beat sine sub one octave down, sidechained to the kick, with the kick's boosted
  fundamental cut from the sub [S23].
- Bass on small speakers: audibility comes from the mid-range harmonics – check on small speakers; ~1 kHz lift,
  octave layers, parallel distortion EQ'd to the useful range [S14].

## 8. Reverb and delay

- Tempo-synced delays (1/8, dotted 1/8, 1/4) are standard in the genre tutorials
  [S31][S29][S27].
- High-pass effect returns so the sub stays mono and clean [S14]; pre-delay at zero on a small hall with a dotted
  1/8 delay (80/20 mix, low cut 100 Hz) on keys in one progressive-house example
  [S27].
- Techno "rumble": kick → convolution reverb, high-cut ~200 Hz, blended back [S23].
- Ducking reverbs/delays from the dry signal or kick to keep clarity **(to verify)**.

## 9. Saturation and transient shaping

- Parallel distortion adds harmonics for small-speaker bass translation; saturation can tame harsh resonant synth
  peaks [S14]; bus saturation is used on melodic-techno groups [S31].
- Transient shaping (attack/sustain gain control) to sharpen or soften drums **(to verify – no checked source)**.
  Compressor attack also shapes transients (slow attack lets them through) [S17].

## 10. Balance

- Level first, then EQ problem areas, then dynamics; compare against references in the target subgenre; check mono
  and multiple systems [S23]. In techno the kick is the focal point [S23].
- **COSOUND rule:** balance advice must reference measured relative levels (stem short-term LUFS or RMS) or a
  measured reference-track spectrum – see [measurement-analysis.md](measurement-analysis.md).
