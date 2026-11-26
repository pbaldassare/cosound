//! Audio features for the library: spectrum, transients, tempo, key, timbre descriptors and a
//! similarity vector, computed from the decoded signal. Everything here is an ESTIMATE: tempo and
//! key carry a confidence and are left unknown (None) when the signal does not support them.
use crate::audio::wav::{Measurements, Meter};
use crate::music::theory;
use serde::Serialize;

const N: usize = 2048;
const HOP: usize = 512;
pub const VERSION: u32 = 1;
const NOTES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

/// Iterative radix-2 FFT (magnitudes only).
struct Fft {
    cos: Vec<f32>,
    sin: Vec<f32>,
    rev: Vec<usize>,
}

impl Fft {
    fn new(n: usize) -> Fft {
        let bits = n.trailing_zeros();
        Fft {
            cos: (0..n / 2).map(|k| (2.0 * std::f32::consts::PI * k as f32 / n as f32).cos()).collect(),
            sin: (0..n / 2).map(|k| -(2.0 * std::f32::consts::PI * k as f32 / n as f32).sin()).collect(),
            rev: (0..n).map(|i| i.reverse_bits() >> (usize::BITS - bits)).collect(),
        }
    }
    fn mags(&self, x: &[f32], out: &mut [f32]) {
        let n = x.len();
        let mut re: Vec<f32> = (0..n).map(|i| x[self.rev[i]]).collect();
        let mut im = vec![0.0f32; n];
        let mut len = 2;
        while len <= n {
            let step = n / len;
            for start in (0..n).step_by(len) {
                for k in 0..len / 2 {
                    let (c, s) = (self.cos[k * step], self.sin[k * step]);
                    let (a, b) = (start + k, start + k + len / 2);
                    let tr = re[b] * c - im[b] * s;
                    let ti = re[b] * s + im[b] * c;
                    re[b] = re[a] - tr;
                    im[b] = im[a] - ti;
                    re[a] += tr;
                    im[a] += ti;
                }
            }
            len <<= 1;
        }
        for (k, o) in out.iter_mut().enumerate() {
            *o = (re[k] * re[k] + im[k] * im[k]).sqrt();
        }
    }
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Estimate<T> {
    pub value: Option<T>,
    /// 0..1; meaningful only when `value` is Some.
    pub confidence: f64,
    /// analysis | analysis+loop_length | embedded | filename | user | unknown
    pub source: String,
    pub note: Option<String>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Descriptors {
    pub deep: f64,
    pub bright: f64,
    pub dark: f64,
    pub punchy: f64,
    pub soft: f64,
    pub noisy: f64,
    pub tonal: f64,
    pub warm: f64,
    pub aggressive: f64,
    pub round: f64,
    pub wide: f64,
    pub atmospheric: f64,
    pub percussive: f64,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Features {
    pub version: u32,
    pub analyzed_s: f64,
    pub centroid_hz: f64,
    pub rolloff_hz: f64,
    pub flatness: f64,
    pub zero_crossing_rate: f64,
    /// Energy share: sub <60 Hz, low 60–250, low-mid 250–2k, high-mid 2k–6k, high >6 kHz.
    pub bands: [f64; 5],
    pub onsets: usize,
    pub onset_rate: f64,
    pub attack_ms: Option<f64>,
    pub decay_ms: Option<f64>,
    pub bpm: Estimate<f64>,
    pub key: Estimate<String>,
    pub key_candidates: Vec<(String, f64)>,
    pub root_note: Estimate<String>,
    pub loop_bars: Option<f64>,
    pub is_loop: bool,
    pub is_one_shot: bool,
    pub descriptors: Descriptors,
    /// Normalized timbre vector for similarity (estimate, not a fingerprint).
    pub vector: Vec<f32>,
}

pub struct Analyzer {
    sr: f32,
    meter: Meter,
    pending: Vec<f32>,
    window: Vec<f32>,
    fft: Fft,
    mag: Vec<f32>,
    prev: Vec<f32>,
    flux: Vec<f32>,
    jumps: Vec<u16>,
    env: Vec<f32>,
    w_sum: f64,
    centroid: f64,
    rolloff: f64,
    flat: f64,
    bands: [f64; 5],
    chroma: [f64; 12],
    bin_pc: Vec<Option<usize>>,
    bin_band: Vec<usize>,
    zc: u64,
    last: f32,
    mono_n: u64,
    x: Vec<f64>,
}

impl Analyzer {
    pub fn channels(&self) -> usize {
        self.x.len()
    }

    pub fn new(rate: u32, channels: usize) -> Analyzer {
        let sr = rate as f32;
        let bin_hz = sr / N as f32;
        Analyzer {
            sr,
            meter: Meter::new(rate, channels),
            pending: Vec::with_capacity(N * 2),
            window: (0..N).map(|i| 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / N as f32).cos()).collect(),
            fft: Fft::new(N),
            mag: vec![0.0; N / 2 + 1],
            prev: vec![0.0; N / 2 + 1],
            flux: Vec::new(),
            jumps: Vec::new(),
            env: Vec::new(),
            w_sum: 0.0,
            centroid: 0.0,
            rolloff: 0.0,
            flat: 0.0,
            bands: [0.0; 5],
            chroma: [0.0; 12],
            bin_pc: (0..=N / 2)
                .map(|k| {
                    let f = k as f32 * bin_hz;
                    (55.0..2100.0).contains(&f).then(|| ((69.0 + 12.0 * (f / 440.0).log2()).round() as i64).rem_euclid(12) as usize)
                })
                .collect(),
            bin_band: (0..=N / 2)
                .map(|k| match k as f32 * bin_hz {
                    f if f < 60.0 => 0,
                    f if f < 250.0 => 1,
                    f if f < 2000.0 => 2,
                    f if f < 6000.0 => 3,
                    _ => 4,
                })
                .collect(),
            zc: 0,
            last: 0.0,
            mono_n: 0,
            x: vec![0.0; channels.max(1)],
        }
    }

    /// Interleaved samples, `ch` channels.
    pub fn push(&mut self, buf: &[f32], ch: usize) {
        if ch != self.x.len() {
            return;
        }
        for frame in buf.chunks_exact(ch) {
            let mut m = 0.0f32;
            for (c, &v) in frame.iter().enumerate() {
                self.x[c] = v as f64;
                m += v;
            }
            self.meter.push(&self.x);
            let m = m / ch as f32;
            if (m >= 0.0) != (self.last >= 0.0) {
                self.zc += 1;
            }
            self.last = m;
            self.mono_n += 1;
            self.pending.push(m);
            if self.pending.len() == N {
                self.frame();
                self.pending.drain(..HOP);
            }
        }
    }

    fn frame(&mut self) {
        let w: Vec<f32> = self.pending.iter().zip(&self.window).map(|(a, b)| a * b).collect();
        self.fft.mags(&w, &mut self.mag);
        let rms = (self.pending[N - HOP..].iter().map(|v| v * v).sum::<f32>() / HOP as f32).sqrt();
        self.env.push(rms);
        let mut flux = 0.0;
        let mut jumps = 0u16;
        for (m, p) in self.mag.iter().zip(self.prev.iter_mut()) {
            let l = (1.0 + 100.0 * m).ln();
            let d = l - *p;
            flux += d.max(0.0);
            if d > 2.0 && l > 2.0 {
                jumps = jumps.saturating_add(1);
            }
            *p = l;
        }
        self.flux.push(flux);
        self.jumps.push(jumps);
        let pw: Vec<f64> = self.mag.iter().map(|m| (*m as f64) * (*m as f64)).collect();
        let total: f64 = pw[1..].iter().sum();
        if total < 1e-9 {
            return;
        }
        let bin_hz = self.sr as f64 / N as f64;
        let msum: f64 = self.mag[1..].iter().map(|m| *m as f64).sum();
        let centroid = self.mag.iter().enumerate().skip(1).map(|(k, m)| k as f64 * bin_hz * *m as f64).sum::<f64>() / msum.max(1e-12);
        let mut acc = 0.0;
        let mut roll = 0.0;
        for (k, p) in pw.iter().enumerate().skip(1) {
            acc += p;
            if acc >= 0.85 * total {
                roll = k as f64 * bin_hz;
                break;
            }
        }
        let n = (pw.len() - 1) as f64;
        let geo = (pw[1..].iter().map(|p| (p + 1e-12).ln()).sum::<f64>() / n).exp();
        let flat = geo / (total / n + 1e-12);
        let weight = total.sqrt();
        self.w_sum += weight;
        self.centroid += centroid * weight;
        self.rolloff += roll * weight;
        self.flat += flat * weight;
        for (k, p) in pw.iter().enumerate().skip(1) {
            self.bands[self.bin_band[k]] += p;
            if let Some(pc) = self.bin_pc[k] {
                self.chroma[pc] += p;
            }
        }
    }

    pub fn measurements(&self, encoding: &'static str, bits: u16) -> Measurements {
        self.meter.finish(encoding, bits)
    }

    /// `full_duration_s`: the whole file's length (may be longer than what was analyzed).
    pub fn finish(&self, full_duration_s: f64, m: &Measurements) -> Features {
        let fr = self.sr as f64 / HOP as f64;
        let analyzed = self.mono_n as f64 / self.sr as f64;
        let w = self.w_sum.max(1e-12);
        let centroid = self.centroid / w;
        let flatness = (self.flat / w).clamp(0.0, 1.0);
        let btot: f64 = self.bands.iter().sum::<f64>().max(1e-12);
        let bands = self.bands.map(|b| r3(b / btot));
        // onsets: peaks of the flux above mean + 1.5 sd, at least 50 ms apart
        let n = self.flux.len();
        let mean = self.flux.iter().map(|v| *v as f64).sum::<f64>() / n.max(1) as f64;
        let sd = (self.flux.iter().map(|v| (*v as f64 - mean).powi(2)).sum::<f64>() / n.max(1) as f64).sqrt();
        let min_gap = (0.05 * fr).ceil() as usize;
        let fmax = self.flux.iter().skip(2).map(|v| *v as f64).fold(0.0, f64::max);
        let mut onsets = Vec::new();
        for i in 1..n.saturating_sub(1) {
            let v = self.flux[i] as f64;
            // a real attack also makes the level jump within ~2 frames (slow beating does not)
            // a real attack makes some frequencies jump by >17 dB (slow beating moves them ~6 dB)
            let rise = self.jumps[i.saturating_sub(2)..(i + 1).min(n)].iter().any(|j| *j >= 3);
            if v > mean + 1.5 * sd && v > 0.25 * fmax && rise && v >= self.flux[i - 1] as f64 && v >= self.flux[i + 1] as f64 && onsets.last().is_none_or(|&l: &usize| i - l >= min_gap) {
                onsets.push(i);
            }
        }
        // attack and decay from the frame envelope
        let peak_i = self.env.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).map(|(i, _)| i);
        let (attack_ms, decay_ms) = match peak_i {
            Some(p) if self.env[p] > 1e-4 => {
                let pk = self.env[p];
                let start = self.env[..=p].iter().position(|v| *v > pk * 0.01).unwrap_or(p);
                let full = self.env[..=p].iter().position(|v| *v >= pk * 0.9).unwrap_or(p);
                let end = self.env[p..].iter().position(|v| *v < pk * 0.1).map(|e| p + e);
                (Some(r1((full - start) as f64 * 1000.0 / fr)), end.map(|e| r1((e - p) as f64 * 1000.0 / fr)))
            }
            _ => (None, None),
        };
        let mut f = Features {
            version: VERSION,
            analyzed_s: r3(analyzed),
            centroid_hz: r1(centroid),
            rolloff_hz: r1(self.rolloff / w),
            flatness: r3(flatness),
            zero_crossing_rate: r3(self.zc as f64 / self.mono_n.max(1) as f64),
            bands,
            onsets: onsets.len(),
            onset_rate: r3(onsets.len() as f64 / analyzed.max(1e-3)),
            attack_ms,
            decay_ms,
            ..Default::default()
        };
        f.bpm = self.tempo(&onsets, full_duration_s, fr);
        if let Some(b) = f.bpm.value {
            let bars = full_duration_s * b / 240.0;
            let nearest = [1.0, 2.0, 4.0, 8.0, 16.0, 32.0].into_iter().min_by(|a, b| (bars - a).abs().total_cmp(&(bars - b).abs())).unwrap_or(1.0);
            if ((bars - nearest) / nearest).abs() < 0.03 {
                f.loop_bars = Some(nearest);
            }
        }
        f.is_one_shot = full_duration_s < 2.0 && onsets.len() <= 2;
        f.is_loop = !f.is_one_shot && (f.loop_bars.is_some() || (f.bpm.value.is_some() && f.bpm.confidence >= 0.5 && full_duration_s < 60.0));
        // key and root note from the chroma
        let csum: f64 = self.chroma.iter().sum();
        let tonal = 1.0 - (flatness / 0.5).clamp(0.0, 1.0);
        f.key = Estimate { value: None, confidence: 0.0, source: "unknown".into(), note: Some("too short or no pitched content".into()) };
        f.root_note = Estimate { value: None, confidence: 0.0, source: "unknown".into(), note: Some("too short or no pitched content".into()) };
        if csum > 0.0 && analyzed >= 0.3 {
            let ranked = theory::detect_key(&self.chroma);
            f.key_candidates = ranked.iter().take(3).map(|(k, c)| (k.name(), r3(*c))).collect();
            let (best, corr) = (&ranked[0].0, ranked[0].1);
            let margin = corr - ranked.get(1).map_or(0.0, |r| r.1);
            let (pc, share) = self.chroma.iter().enumerate().map(|(i, v)| (i, v / csum)).max_by(|a, b| a.1.total_cmp(&b.1)).unwrap_or((0, 0.0));
            if share > 0.45 && !f.is_one_shot {
                // one pitch class carries most of the energy (a drum loop's kick, a one-note bass): no key to read
                f.key = Estimate { value: None, confidence: 0.0, source: "unknown".into(), note: Some("one pitch dominates (drums or a single-note part): see root_note".into()) };
            } else if analyzed >= 1.0 && corr >= 0.6 && flatness < 0.35 && !f.is_one_shot {
                // near-tied candidates (e.g. relative major/minor) lower the confidence
                let conf = ((corr - 0.5) / 0.4).clamp(0.0, 1.0) * tonal.max(0.2) * (margin / 0.1).clamp(0.3, 1.0);
                f.key = Estimate { value: Some(best.name()), confidence: r3(conf), source: "analysis".into(), note: None };
            } else {
                f.key = Estimate { value: None, confidence: 0.0, source: "unknown".into(), note: Some(if f.is_one_shot { "one-shot: key not meaningful, see root_note" } else { "not enough tonal content" }.into()) };
            }
            f.root_note = if share > 0.25 && flatness < 0.3 {
                Estimate { value: Some(NOTES[pc].into()), confidence: r3(share.min(1.0)), source: "analysis".into(), note: None }
            } else {
                Estimate { value: None, confidence: 0.0, source: "unknown".into(), note: Some("no dominant pitch".into()) }
            };
        }
        // descriptors (0..1 scores)
        let low_share = bands[0] + bands[1];
        let bright = ((centroid.max(1.0).log2() - 500f64.log2()) / (6000f64.log2() - 500f64.log2())).clamp(0.0, 1.0);
        let attack = attack_ms.unwrap_or(50.0);
        let crest = m.crest_factor_db;
        let punchy = 0.5 * ((30.0 - attack) / 30.0).clamp(0.0, 1.0) + 0.5 * ((crest - 6.0) / 12.0).clamp(0.0, 1.0);
        let percussive = (f.onset_rate / 4.0).clamp(0.0, 1.0).max(if full_duration_s < 1.0 && attack < 15.0 { 1.0 } else { 0.0 });
        let long = ((full_duration_s - 1.5) / 3.0).clamp(0.0, 1.0);
        let wide = m.stereo_correlation.map_or(0.0, |c| (1.0 - c).clamp(0.0, 1.0));
        f.descriptors = Descriptors {
            deep: r3((low_share / 0.7).clamp(0.0, 1.0)),
            bright: r3(bright),
            dark: r3(1.0 - bright),
            punchy: r3(punchy),
            soft: r3(1.0 - punchy),
            noisy: r3((flatness / 0.5).clamp(0.0, 1.0)),
            tonal: r3(tonal),
            warm: r3((bands[2] / 0.5).clamp(0.0, 1.0) * (1.0 - bright) * tonal.max(0.3)),
            aggressive: r3(0.5 * (bands[3] / 0.3).clamp(0.0, 1.0) + 0.5 * (1.0 - ((crest - 6.0) / 12.0).clamp(0.0, 1.0))),
            round: r3(tonal * (1.0 - bright) * (1.0 - (bands[4] * 10.0).clamp(0.0, 1.0))),
            wide: r3(wide),
            atmospheric: r3(long * (1.0 - punchy) * (1.0 - percussive)),
            percussive: r3(percussive),
        };
        let d = &f.descriptors;
        let dur_n = ((full_duration_s.max(0.05).ln() - 0.05f64.ln()) / (60f64.ln() - 0.05f64.ln())).clamp(0.0, 1.0);
        f.vector = [d.bright, d.noisy, d.deep, bands[0], bands[1], bands[2], bands[3], bands[4], d.punchy, (f.zero_crossing_rate * 10.0).min(1.0), dur_n, (f.onset_rate / 8.0).min(1.0), d.tonal, ((crest - 3.0) / 20.0).clamp(0.0, 1.0)]
            .iter()
            .map(|v| (*v as f32 * 1000.0).round() / 1000.0)
            .collect();
        f
    }

    /// Tempo from the autocorrelation of the onset-strength envelope, refined on the loop length.
    fn tempo(&self, onsets: &[usize], dur: f64, fr: f64) -> Estimate<f64> {
        let unknown = |why: &str| Estimate { value: None, confidence: 0.0, source: "unknown".into(), note: Some(why.into()) };
        let n = self.flux.len();
        if dur < 1.5 || n < 64 || onsets.len() < 4 {
            return unknown("too short or too few onsets for a tempo");
        }
        // onset strength: flux minus its local mean, half-wave rectified
        let half = (0.25 * fr) as usize;
        let o: Vec<f64> = (0..n)
            .map(|i| {
                let (a, b) = (i.saturating_sub(half), (i + half).min(n - 1));
                let local = self.flux[a..=b].iter().map(|v| *v as f64).sum::<f64>() / (b - a + 1) as f64;
                (self.flux[i] as f64 - local).max(0.0)
            })
            .collect();
        // smooth so that onsets falling between analysis frames still line up
        let k = [0.1, 0.2, 0.4, 0.2, 0.1];
        let o: Vec<f64> = (0..n).map(|i| (0..5).map(|j| o.get((i + j).wrapping_sub(2)).copied().unwrap_or(0.0) * k[j]).sum()).collect();
        let max_lag = ((60.0 * fr / 50.0) as usize * 4).min(n - 1);
        let ac: Vec<f64> = (0..=max_lag).map(|l| (0..n - l).map(|i| o[i] * o[i + l]).sum::<f64>() / (n - l) as f64).collect();
        let at = |lag: f64| {
            let i = lag.floor() as usize;
            if i + 1 >= ac.len() {
                return 0.0;
            }
            let t = lag - i as f64;
            ac[i] * (1.0 - t) + ac[i + 1] * t
        };
        let mut scores = Vec::new();
        let mut b = 70.0;
        while b <= 180.0 {
            let lag = 60.0 * fr / b;
            scores.push((b, at(lag) + 0.5 * at(2.0 * lag) + 0.25 * at(4.0 * lag)));
            b += 0.1;
        }
        let (best, s) = scores.iter().cloned().max_by(|a, b| a.1.total_cmp(&b.1)).unwrap_or((0.0, 0.0));
        let mut sorted: Vec<f64> = scores.iter().map(|x| x.1).collect();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let median = sorted[sorted.len() / 2];
        let conf = if s > 0.0 { ((s - median) / s).clamp(0.0, 1.0) } else { 0.0 };
        if conf < 0.3 {
            return unknown("no clear periodicity");
        }
        // a loop that is a whole number of bars gives the exact tempo: prefer such a tempo when
        // its periodicity score is close to the best one
        let score_at = |b: f64| {
            let lag = 60.0 * fr / b;
            at(lag) + 0.5 * at(2.0 * lag) + 0.25 * at(4.0 * lag)
        };
        let mut loop_best: Option<(f64, f64, f64)> = None;
        for nb in [1.0, 2.0, 4.0, 8.0, 16.0, 32.0] {
            let exact = nb * 240.0 / dur;
            if (70.0..=180.0).contains(&exact) {
                let sc = score_at(exact).max(score_at(exact * 1.002)).max(score_at(exact * 0.998));
                if sc >= 0.85 * s && loop_best.is_none_or(|l| sc > l.2) {
                    loop_best = Some((nb, exact, sc));
                }
            }
        }
        if let Some((nb, exact, _)) = loop_best {
            return Estimate { value: Some(r2(exact)), confidence: r3((conf + 0.15).min(1.0)), source: "analysis+loop_length".into(), note: Some(format!("{nb} bars")) };
        }
        Estimate { value: Some(r1(best)), confidence: r3(conf), source: "analysis".into(), note: None }
    }
}

fn r1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}
fn r2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}
fn r3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// Timbre similarity 0..1 from the RMS distance of two feature vectors (each dimension is 0..1).
/// Gaussian falloff: distance 0.05 → 0.94, 0.1 → 0.78, 0.2 → 0.37, 0.3 → 0.11. On the test library,
/// sounds of different kinds (kick vs pad, bass vs hat) are 0.27 or more apart, so they score low.
pub fn similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let rms = (a.iter().zip(b).map(|(x, y)| ((x - y) as f64).powi(2)).sum::<f64>() / a.len() as f64).sqrt();
    r3((-(rms / 0.2).powi(2)).exp())
}

#[cfg(test)]
pub mod fixtures {
    //! Synthetic sounds with known properties, shared by the library tests.
    use std::f64::consts::PI;

    pub fn kick(sr: u32) -> Vec<f64> {
        let n = (sr as f64 * 0.45) as usize;
        let mut ph = 0.0;
        (0..n)
            .map(|i| {
                let t = i as f64 / sr as f64;
                let f = 45.0 + 110.0 * (-t * 30.0).exp();
                ph += 2.0 * PI * f / sr as f64;
                0.95 * ph.sin() * (-t * 7.0).exp()
            })
            .collect()
    }
    pub fn noise(seed: &mut u64) -> f64 {
        *seed ^= *seed << 13;
        *seed ^= *seed >> 7;
        *seed ^= *seed << 17;
        (*seed % 20001) as f64 / 10000.0 - 1.0
    }
    pub fn hat(sr: u32) -> Vec<f64> {
        let mut s = 12345u64;
        let mut prev = 0.0;
        (0..(sr as f64 * 0.12) as usize)
            .map(|i| {
                let x = noise(&mut s);
                let hp = x - prev; // crude high-pass
                prev = x;
                0.6 * hp * (-(i as f64) / sr as f64 * 40.0).exp()
            })
            .collect()
    }
    /// Four-on-the-floor kicks for `bars` bars at `bpm`.
    pub fn drum_loop(sr: u32, bpm: f64, bars: u32) -> Vec<f64> {
        let beat = (60.0 / bpm * sr as f64) as usize;
        let total = (bars as f64 * 4.0 * 60.0 / bpm * sr as f64) as usize;
        let k = kick(sr);
        let h = hat(sr);
        let mut out = vec![0.0; total];
        for b in 0..bars as usize * 4 {
            for (i, v) in k.iter().enumerate() {
                if b * beat + i < total {
                    out[b * beat + i] += v;
                }
            }
            for (i, v) in h.iter().enumerate() {
                let at = b * beat + beat / 2 + i;
                if at < total {
                    out[at] += v * 0.7;
                }
            }
        }
        out
    }
    /// Arpeggio A–C–E–G (A minor 7) in 16ths at `bpm`, sine-saw tone.
    pub fn arp_loop(sr: u32, bpm: f64, bars: u32, midi: &[u8]) -> Vec<f64> {
        let step = (15.0 / bpm * sr as f64) as usize;
        let total = (bars as f64 * 4.0 * 60.0 / bpm * sr as f64) as usize;
        (0..total)
            .map(|i| {
                let n = midi[(i / step) % midi.len()] as f64;
                let f = 440.0 * 2f64.powf((n - 69.0) / 12.0);
                let t = (i % step) as f64 / sr as f64;
                let env = (-t * 9.0).exp() * (1.0 - (-t * 400.0).exp());
                let ph = 2.0 * PI * f * i as f64 / sr as f64;
                0.5 * env * (ph.sin() + 0.3 * (2.0 * ph).sin() + 0.15 * (3.0 * ph).sin())
            })
            .collect()
    }
    /// Sustained chord pad (slow attack), stereo-detuned.
    pub fn pad(sr: u32, secs: f64, midi: &[u8]) -> Vec<f64> {
        let n = (sr as f64 * secs) as usize;
        (0..n)
            .flat_map(|i| {
                let t = i as f64 / sr as f64;
                let env = (t / 1.5).min(1.0) * (((secs - t) / 1.0).min(1.0)).max(0.0);
                let mut l = 0.0;
                let mut r = 0.0;
                for (v, &m) in midi.iter().enumerate() {
                    let f = 440.0 * 2f64.powf((m as f64 - 69.0) / 12.0);
                    let ph = v as f64 * 1.7; // voices start at different phases
                    // a centre voice plus a quieter detuned voice per side, as in a chorus/unison patch
                    l += (2.0 * PI * f * t + ph).sin() + 0.35 * (2.0 * PI * f * 1.004 * t + ph).sin();
                    r += (2.0 * PI * f * t + ph + 0.4).sin() + 0.35 * (2.0 * PI * f * 0.996 * t + ph).sin();
                }
                let g = 0.35 * env / midi.len() as f64;
                [l * g, r * g]
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::*;
    use super::*;

    fn run(samples: &[f64], sr: u32, ch: usize) -> (Features, Measurements) {
        let mut a = Analyzer::new(sr, ch);
        let f32s: Vec<f32> = samples.iter().map(|v| *v as f32).collect();
        for chunk in f32s.chunks(4096 * ch) {
            a.push(chunk, ch);
        }
        let m = a.measurements("float", 32);
        let dur = samples.len() as f64 / ch as f64 / sr as f64;
        (a.finish(dur, &m), m)
    }

    #[test]
    fn tempo_of_drum_and_arp_loops() {
        for (bpm, bars) in [(124.0, 2), (128.0, 4), (95.0, 2), (140.0, 4)] {
            let (f, _) = run(&drum_loop(44100, bpm, bars), 44100, 1);
            let got = f.bpm.value.unwrap_or_else(|| panic!("no tempo for {bpm}: {:?}", f.bpm));
            assert!((got - bpm).abs() < 0.5, "drum loop {bpm}: got {got} {:?}", f.bpm);
            assert_eq!(f.loop_bars, Some(bars as f64));
            assert!(f.is_loop && !f.is_one_shot);
        }
        let (f, _) = run(&arp_loop(44100, 124.0, 2, &[57, 60, 64, 67]), 44100, 1);
        assert!((f.bpm.value.unwrap() - 124.0).abs() < 0.5, "{:?}", f.bpm);
    }

    #[test]
    fn key_of_tonal_material() {
        let (f, _) = run(&arp_loop(44100, 124.0, 2, &[57, 60, 64, 69, 72, 76, 74, 71]), 44100, 1);
        assert_eq!(f.key.value.as_deref(), Some("A minor"), "{:?} {:?}", f.key, f.key_candidates);
        let (f, _) = run(&pad(44100, 6.0, &[60, 64, 67]), 44100, 2);
        let k = f.key.value.clone().unwrap_or_default();
        assert!(k == "C major" || k == "A minor" || k == "G major" || k == "F major", "C major triad: {:?} {:?}", f.key, f.key_candidates);
        // a hat has no key and says so
        let (f, _) = run(&hat(44100), 44100, 1);
        assert!(f.key.value.is_none() && f.key.note.is_some());
    }

    #[test]
    fn one_shots_and_descriptors() {
        let (k, mk) = run(&kick(44100), 44100, 1);
        assert!(k.is_one_shot && !k.is_loop && k.bpm.value.is_none(), "kick: {:?}", k.bpm);
        assert!(k.descriptors.deep > 0.8 && k.bands[0] + k.bands[1] > 0.6, "kick is deep: {:?}", k.bands);
        assert!(mk.sample_peak_dbfs > -1.0);
        let (h, _) = run(&hat(44100), 44100, 1);
        assert!(h.descriptors.bright > 0.8 && h.descriptors.noisy > 0.5 && h.descriptors.deep < 0.2, "hat: {:?}", h.descriptors);
        let (p, _) = run(&pad(44100, 6.0, &[57, 60, 64]), 44100, 2);
        assert!(p.descriptors.atmospheric > 0.5 && p.descriptors.tonal > 0.7 && p.attack_ms.unwrap() > 200.0, "pad: {:?} attack {:?}", p.descriptors, p.attack_ms);
        assert!(similarity(&k.vector, &k.vector) > 0.99);
        assert!(similarity(&k.vector, &h.vector) < similarity(&k.vector, &run(&kick(48000), 48000, 1).0.vector), "a kick is closer to a kick than to a hat");
        assert!(similarity(&k.vector, &h.vector) < 0.2, "a kick and a hat are not similar sounds: {}", similarity(&k.vector, &h.vector));
    }
}
