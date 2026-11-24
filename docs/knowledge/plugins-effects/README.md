# Plugins and effects – how they work and typical settings

Section 5 (Sound Design). Verified 2026-11-24. Sources: [`sources.md`](sources.md).
Mixing and mastering depth (EQ and compression strategy, loudness) belongs to
`../audio-engineering/`. This file covers mechanism and sound-design use only.

Conventions:
- Ranges with a citation come from that source.
- Ranges marked *(editorial)* are COSOUND starting points that no source states. Tune them by ear.

---

## 1. Distortion and saturation

**Mechanism**
- Distortion is a non-linear transfer curve: each input sample is mapped to a new output value
  through a shaping curve. Ableton describes its Saturator in exactly these terms [S13]. Any
  non-linearity creates **new harmonics** [S14].
- **Hard clipping:** the signal is cut abruptly at a threshold. A sine clipped harder and harder
  tends towards a square wave, which adds mainly **odd** harmonics [S14].
- **Soft clipping/saturation:** a gradual curve. It is gentler, and it also reduces dynamic range
  [S14].
- **Asymmetric curves** treat the positive and negative halves differently, which adds **even**
  harmonics on top of the odd ones for a fuller sound [S14].

**Terminology**
- "Saturation → overdrive → fuzz" runs from subtle to extreme. The boundaries are vague, and
  tape/tube saturators also change dynamics, not just spectrum [S15].

**Electronic-music use**
- Make a sub-bass audible on small speakers by adding harmonics *(editorial, follows from [S14])*.
- Saturate whole groups or buses lightly, since saturation suits complex material better than
  clipping. Clipping works as a creative tool on single sources [S14].
- Use drive before a resonant filter for an acid-style "scream" (see synthesizers recipes).

**Typical controls**
- Drive/input gain, curve or type, tone/filter, dry/wet mix, output gain.
- Some devices add oversampling to reduce aliasing. Sylenth1's distortion, for example, offers
  overdrive, foldback, clip, decimate and bitcrush at 4× oversampling [S18].
- Multi-stage saturators combine shapers, filters and feedback; Ableton Roar is an example [S13].

**Bit-crushing/decimation:** reduces bit depth and sample rate to produce digital aliasing
artefacts. Listed among Sylenth1's distortion types [S18].

---

## 2. Reverb

**What it models**
- In a room, the sound starts with discrete **early reflections**, which cue room size and shape,
  and then becomes a diffuse **tail** [S12]. Early reflections are often taken as roughly the first
  100 ms [S9].

**Algorithmic (digital) reverb**
- Builds reflections from networks of delays: Schroeder's allpass chains plus parallel feedback
  comb filters [S7], or feedback delay networks (FDN) [S8], with tapped delay lines for early
  reflections [S9].
- It is cheap to modulate and automate.
- ChromaVerb is Apple's example: up to 14 room types, built on a circular structure in which the
  sound is gradually absorbed [S11].

**Convolution reverb**
- Convolves the signal with an **impulse response (IR)**, a recording of every reflection in a real
  space after a spike, stored as an ordinary audio file [S10].
- It is highly realistic. Space Designer can also synthesize IRs [S10].
- The limitation: changing the IR or its length means recalculating, so it is less suitable for
  heavy automation *(editorial; Apple notes recalculation, see [S10])*.

**Hybrid:** some devices combine the two, for example Ableton Hybrid Reverb [S13].

**Parameters and typical values**

| Parameter | Meaning | Typical use |
|---|---|---|
| Pre-delay | Gap before the first reflection | 1–25 ms reads as "natural" [S13]. Longer (tempo-synced) separates a vocal or lead from its tail *(editorial)* |
| Decay / RT | Tail length | ~0.3–1 s tight rooms for drums, 2–4 s halls for pads, >5 s ambient *(editorial)* |
| Size / room type | Reflection spacing and density | ChromaVerb room types [S11] |
| Density / diffusion | How quickly echoes blur into a tail | High for pads, lower for rhythmic clarity *(editorial)* |
| Damping / HF decay, low cut | Spectral shape of the tail | Cut lows to keep the kick and bass clean *(editorial)* |
| Mix (wet/dry) | Balance | 100 % wet on a send; low % as an insert *(editorial)* |

---

## 3. Delay

**Mechanism**
- A buffer that plays audio back later. Feeding part of the output back to the input creates
  repeating echoes, each quieter than the last. Feedback must stay below unity or the repeats build
  into howl [S1].

**Variants**
- **Tempo-synced delay**, the norm in plug-ins [S1].
- **Ping-pong:** two delay lines cross-fed left/right so repeats bounce across the stereo field
  [S1].
- **Slapback:** one short repeat [S1].
- **Tape-style:** adds wobble and saturation [S1].
- **Filtered delays:** HP/LP in the feedback path. Sylenth1 and Ableton Echo have these [S13][S18].

**Electronic-music use**
- 1/8, dotted-1/8 and 1/4 synced repeats on plucks and leads *(editorial)*.
- Dub-style rising feedback throws [S1].
- Mid/side or ping-pong for width (Ableton Echo modes: Stereo, Ping Pong, Mid/Side) [S13].

---

## 4. Chorus, flanger, phaser (modulation effects)

| Effect | Mechanism | Typical delay range | Character |
|---|---|---|---|
| **Chorus** | One or more short delay lines with LFO-modulated delay time; the dry/wet mix simulates several players in unison [S5] | "A few tens of ms", LFO ≈3 Hz [S1]; 10–40 ms in Logic's guidance [S2] | Thickening, width; below ~15 ms the delay itself is not perceived, so 100 % wet gives vibrato [S1] |
| **Flanger** | Feedforward comb filter whose delay is varied over time; the comb's peaks and notches "squeeze in and out" [S3]. Add feedback for intensity [S1] | Short values (below chorus range) [S2] | Jet-like "whoosh", metallic with feedback |
| **Phaser** | Chain of all-pass filters that sweep **notches** through the spectrum. Commercial phasers use non-uniformly spaced notches, unlike the flanger's harmonic comb [S4]. Ableton exposes Notches (number of all-pass stages), Center and Spread [S13] | n/a (no audible delay) [S1] | Gentler, "swirling" sweep on pads, keys, guitars [S1] |

Usual controls: rate (Hz or synced), depth, delay/center, feedback, stereo spread/phase, mix.
Examples: Ableton Phaser-Flanger (with a Doubler mode) and Chorus-Ensemble [S13]; Sylenth1's
4-stage chorus/flanger and 6-stage phaser [S18].

---

## 5. EQ and compressor (brief)

**EQ filter shapes:**
- Low cut and high cut (selectable slopes, e.g. 12 or 48 dB/oct).
- Low and high shelf.
- Peak (bell).
- Notch.

Ableton EQ Eight offers up to eight such bands per channel, in Stereo, L/R or M/S [S13]. A slope is
the number of filter stages × 6 dB/oct [S17].

**Compressor parameters** (Logic Compressor) [S16]:
- **Threshold:** level above which gain is reduced.
- **Ratio:** amount of reduction.
- **Attack/Release:** reaction and recovery times.
- **Knee:** low = hard, high = soft.
- **Make-up gain.**

Circuit models emulate classic designs (Platinum Digital, VCA, FET, Opto) [S16]. Ableton's Glue
Compressor models a classic 1980s console bus compressor [S13].

Typical electronic-music uses *(editorial)*:
- Sidechain ducking of pads and bass from the kick.
- Bus "glue".
- Parallel compression on drums.

Details are in `../audio-engineering/`.

---

## 6. Notes for COSOUND

- Effect parameter names differ between vendors (Mix/Dry-Wet/Amount, Decay/Time/RT60). The agent
  needs a synonym map per plug-in. Real names come from the host's parameter enumeration (see
  `../integration-sdks/`).
- Time controls may be in ms or in note values depending on a sync switch. Always read the
  parameter's display string, not only its normalized value.
