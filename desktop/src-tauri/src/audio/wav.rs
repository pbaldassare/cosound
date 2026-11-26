//! Tool `audio.wav_meter`: streaming WAV reader and measurements.
//! - Integrated loudness per ITU-R BS.1770 (K-weighting, 400 ms blocks, 75 % overlap, -70 LKFS
//!   absolute and -10 LU relative gates), mono and stereo.
//! - True peak with 4x oversampling (65-tap windowed-sinc interpolator, BS.1770 Annex 2 style).
//! - Sample peak, RMS, crest factor, DC offset, clipped samples, stereo and low-band correlation,
//!   coarse band energy, leading/trailing silence.
//! Long files are processed in chunks and never loaded whole into memory.
use serde::Serialize;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

pub const VERSION: &str = "cosound-wav-meter/1";

#[derive(Debug, PartialEq)]
pub enum MeterError {
    Cancelled,
    Invalid(String),
}

impl From<std::io::Error> for MeterError {
    fn from(e: std::io::Error) -> Self {
        MeterError::Invalid(e.to_string())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Enc {
    Int,
    Float,
}

struct Header {
    rate: u32,
    channels: u16,
    bits: u16,
    enc: Enc,
    data_bytes: u64,
}

fn u16le(b: &[u8]) -> u16 {
    u16::from_le_bytes([b[0], b[1]])
}
fn u32le(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

fn header(r: &mut (impl Read + Seek), file_len: u64) -> Result<Header, MeterError> {
    let bad = |m: &str| MeterError::Invalid(m.to_string());
    let mut riff = [0u8; 12];
    r.read_exact(&mut riff).map_err(|_| bad("not a WAV file (too short)"))?;
    if &riff[0..4] != b"RIFF" || &riff[8..12] != b"WAVE" {
        return Err(bad("not a RIFF/WAVE file"));
    }
    let mut fmt: Option<(u16, u16, u32, u16)> = None;
    loop {
        let mut ch = [0u8; 8];
        r.read_exact(&mut ch).map_err(|_| bad("WAV file has no data chunk"))?;
        let size = u32le(&ch[4..8]) as u64;
        match &ch[0..4] {
            b"fmt " => {
                let mut b = vec![0u8; size.min(64) as usize];
                r.read_exact(&mut b)?;
                if size > 64 {
                    r.seek(SeekFrom::Current((size - 64) as i64))?;
                }
                if b.len() < 16 {
                    return Err(bad("WAV fmt chunk too short"));
                }
                let mut format = u16le(&b[0..2]);
                if format == 0xFFFE && b.len() >= 26 {
                    format = u16le(&b[24..26]); // WAVE_FORMAT_EXTENSIBLE: sub-format GUID starts with the format code
                }
                fmt = Some((format, u16le(&b[2..4]), u32le(&b[4..8]), u16le(&b[14..16])));
                if size % 2 == 1 {
                    r.seek(SeekFrom::Current(1))?;
                }
            }
            b"data" => {
                let (format, channels, rate, bits) = fmt.ok_or_else(|| bad("WAV data chunk before fmt chunk"))?;
                let enc = match (format, bits) {
                    (1, 16 | 24 | 32) => Enc::Int,
                    (3, 32 | 64) => Enc::Float,
                    _ => return Err(MeterError::Invalid(format!("unsupported WAV encoding (format code {format}, {bits} bit)"))),
                };
                if channels == 0 || rate < 8000 {
                    return Err(bad("WAV header has no channels or an invalid sample rate"));
                }
                let pos = r.stream_position()?;
                // some writers leave the size at 0 or 0xFFFFFFFF: trust the file length instead
                let data_bytes = if size == 0 || size == u32::MAX as u64 { file_len.saturating_sub(pos) } else { size.min(file_len.saturating_sub(pos)) };
                return Ok(Header { rate, channels, bits, enc, data_bytes });
            }
            _ => {
                r.seek(SeekFrom::Current((size + size % 2) as i64))?;
            }
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Biquad {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    z1: f64,
    z2: f64,
}

impl Biquad {
    fn run(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
    /// RBJ cookbook low/high-pass, Q = 1/sqrt(2).
    fn pass(fs: f64, f0: f64, high: bool) -> Biquad {
        let w = 2.0 * std::f64::consts::PI * f0 / fs;
        let (c, alpha) = (w.cos(), w.sin() / std::f64::consts::SQRT_2);
        let a0 = 1.0 + alpha;
        let (b0, b1) = if high { ((1.0 + c) / 2.0, -(1.0 + c)) } else { ((1.0 - c) / 2.0, 1.0 - c) };
        Biquad { b0: b0 / a0, b1: b1 / a0, b2: b0 / a0, a1: -2.0 * c / a0, a2: (1.0 - alpha) / a0, ..Default::default() }
    }
    /// BS.1770 K-weighting stage 1 (high shelf) and stage 2 (RLB high-pass), for any sample rate.
    fn k_weighting(fs: f64) -> (Biquad, Biquad) {
        let pi = std::f64::consts::PI;
        let (f0, g, q) = (1681.974450955533, 3.999843853973347, 0.7071752369554196);
        let k = (pi * f0 / fs).tan();
        let vh = 10f64.powf(g / 20.0);
        let vb = vh.powf(0.4996667741545416);
        let a0 = 1.0 + k / q + k * k;
        let s1 = Biquad {
            b0: (vh + vb * k / q + k * k) / a0,
            b1: 2.0 * (k * k - vh) / a0,
            b2: (vh - vb * k / q + k * k) / a0,
            a1: 2.0 * (k * k - 1.0) / a0,
            a2: (1.0 - k / q + k * k) / a0,
            ..Default::default()
        };
        let (f0, q) = (38.13547087602444, 0.5003270373238773);
        let k = (pi * f0 / fs).tan();
        let d = 1.0 + k / q + k * k;
        let s2 = Biquad { b0: 1.0, b1: -2.0, b2: 1.0, a1: 2.0 * (k * k - 1.0) / d, a2: (1.0 - k / q + k * k) / d, ..Default::default() };
        (s1, s2)
    }
}

const TP_TAPS: usize = 65; // 4 phases; phase 0 reproduces the original samples exactly
const TP_HIST: usize = 17;

fn tp_filter() -> [f64; TP_TAPS] {
    let mut h = [0.0; TP_TAPS];
    let c = (TP_TAPS - 1) as f64 / 2.0;
    for (n, v) in h.iter_mut().enumerate() {
        let t = (n as f64 - c) / 4.0;
        let sinc = if t == 0.0 { 1.0 } else { (std::f64::consts::PI * t).sin() / (std::f64::consts::PI * t) };
        let w = 0.42 - 0.5 * (2.0 * std::f64::consts::PI * n as f64 / (TP_TAPS - 1) as f64).cos() + 0.08 * (4.0 * std::f64::consts::PI * n as f64 / (TP_TAPS - 1) as f64).cos();
        *v = sinc * w;
    }
    h
}

#[derive(Serialize, Debug, Clone)]
pub struct BandEnergy {
    pub low_pct: f64,
    pub mid_pct: f64,
    pub high_pct: f64,
}

#[derive(Serialize, Debug, Clone)]
pub struct Measurements {
    pub analyzer: &'static str,
    pub encoding: &'static str,
    pub sample_rate: u32,
    pub channels: u16,
    pub bits: u16,
    pub frames: u64,
    pub duration_s: f64,
    pub sample_peak_dbfs: f64,
    pub channel_peaks_dbfs: Vec<f64>,
    pub true_peak_dbtp: f64,
    pub rms_dbfs: f64,
    pub crest_factor_db: f64,
    /// None when the file is shorter than one 400 ms block, silent, or has more than 2 channels.
    pub integrated_lufs: Option<f64>,
    /// Peak-to-loudness ratio = true peak − integrated loudness.
    pub plr_db: Option<f64>,
    pub dc_offset: Vec<f64>,
    pub clipped_samples: u64,
    pub stereo_correlation: Option<f64>,
    /// Correlation of the content below 120 Hz (2nd-order filter): low values mean a wide low end.
    pub low_band_correlation: Option<f64>,
    /// Share of energy below 120 Hz, 120 Hz–5 kHz, above 5 kHz (2nd-order filters: coarse).
    pub band_energy: Option<BandEnergy>,
    pub leading_silence_s: f64,
    pub trailing_silence_s: f64,
}

pub fn db(x: f64) -> f64 {
    if x <= 1e-6 {
        -120.0
    } else {
        20.0 * x.log10()
    }
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Streaming meter: feed frames (one value per channel, −1..1) and get the measurements.
/// Shared by the WAV reader below and by the multi-format decoder of the Smart Library.
pub struct Meter {
    nch: usize,
    fs: f64,
    kw: Vec<(Biquad, Biquad)>,
    lp: Vec<Biquad>,
    hp: Vec<Biquad>,
    tpf: [f64; TP_TAPS],
    hist: Vec<[f64; TP_HIST]>,
    hpos: usize,
    peak: Vec<f64>,
    tp: f64,
    sumsq: Vec<f64>,
    dc: Vec<f64>,
    low_e: f64,
    high_e: f64,
    all_e: f64,
    clipped: u64,
    corr: [f64; 6],
    first_loud: Option<u64>,
    last_loud: u64,
    hop: u64,
    sub_acc: f64,
    sub_n: u64,
    subs: Vec<f64>,
    frame: u64,
    low: Vec<f64>,
}

impl Meter {
    pub fn new(rate: u32, channels: usize) -> Meter {
        let fs = rate as f64;
        let nch = channels.max(1);
        let (k1, k2) = Biquad::k_weighting(fs);
        Meter {
            nch,
            fs,
            kw: vec![(k1, k2); nch],
            lp: vec![Biquad::pass(fs, 120.0, false); nch],
            hp: vec![Biquad::pass(fs, 5000.0, true); nch],
            tpf: tp_filter(),
            hist: vec![[0.0; TP_HIST]; nch],
            hpos: 0,
            peak: vec![0.0; nch],
            tp: 0.0,
            sumsq: vec![0.0; nch],
            dc: vec![0.0; nch],
            low_e: 0.0,
            high_e: 0.0,
            all_e: 0.0,
            clipped: 0,
            corr: [0.0; 6],
            first_loud: None,
            last_loud: 0,
            hop: (fs / 10.0).round().max(1.0) as u64, // 100 ms sub-blocks
            sub_acc: 0.0,
            sub_n: 0,
            subs: Vec::new(),
            frame: 0,
            low: vec![0.0; nch],
        }
    }

    /// One frame: `x.len()` must equal the channel count.
    pub fn push(&mut self, x: &[f64]) {
        let silence = 0.001; // −60 dBFS
        self.hpos = (self.hpos + 1) % TP_HIST;
        let mut loud = false;
        let mut block_power = 0.0;
        for c in 0..self.nch {
            let v = x[c];
            let a = v.abs();
            self.peak[c] = self.peak[c].max(a);
            self.sumsq[c] += v * v;
            self.dc[c] += v;
            if a >= 0.999 {
                self.clipped += 1;
            }
            if a > silence {
                loud = true;
            }
            let (s1, s2) = &mut self.kw[c];
            let k = s2.run(s1.run(v));
            block_power += k * k;
            self.low[c] = self.lp[c].run(v);
            let hi = self.hp[c].run(v);
            self.low_e += self.low[c] * self.low[c];
            self.high_e += hi * hi;
            self.all_e += v * v;
            // 4x oversampled true peak: phases 1..3 interpolate between samples
            self.hist[c][self.hpos] = v;
            for p in 1..4 {
                let mut y = 0.0;
                let mut k = 0;
                while p + 4 * k < TP_TAPS {
                    y += self.tpf[p + 4 * k] * self.hist[c][(self.hpos + TP_HIST - k) % TP_HIST];
                    k += 1;
                }
                self.tp = self.tp.max(y.abs());
            }
        }
        if self.nch == 2 {
            let (l, r, ll, lr) = (x[0], x[1], self.low[0], self.low[1]);
            for (acc, v) in self.corr.iter_mut().zip([l * r, l * l, r * r, ll * lr, ll * ll, lr * lr]) {
                *acc += v;
            }
        }
        if loud {
            self.first_loud.get_or_insert(self.frame);
            self.last_loud = self.frame;
        }
        self.sub_acc += block_power;
        self.sub_n += 1;
        if self.sub_n == self.hop {
            self.subs.push(self.sub_acc / self.hop as f64);
            self.sub_acc = 0.0;
            self.sub_n = 0;
        }
        self.frame += 1;
    }

    pub fn finish(&self, encoding: &'static str, bits: u16) -> Measurements {
        let (nch, fs, total_frames) = (self.nch, self.fs, self.frame);
        let frames = total_frames.max(1) as f64;
        let sample_peak = self.peak.iter().cloned().fold(0.0, f64::max);
        let mean_sq = self.sumsq.iter().sum::<f64>() / (frames * nch as f64);
        let rms = db(mean_sq.sqrt());
        let integrated = if nch <= 2 && self.subs.len() >= 4 {
            let blocks: Vec<f64> = self.subs.windows(4).map(|w| w.iter().sum::<f64>() / 4.0).collect();
            let lk = |p: f64| -0.691 + 10.0 * p.log10();
            let abs: Vec<f64> = blocks.iter().copied().filter(|&p| p > 0.0 && lk(p) > -70.0).collect();
            if abs.is_empty() {
                None
            } else {
                let rel = lk(abs.iter().sum::<f64>() / abs.len() as f64) - 10.0;
                let gated: Vec<f64> = abs.into_iter().filter(|&p| lk(p) > rel).collect();
                Some(lk(gated.iter().sum::<f64>() / gated.len() as f64))
            }
        } else {
            None
        };
        let true_peak = db(self.tp.max(sample_peak));
        let corr = |a: f64, b: f64, ab: f64| if a > 0.0 && b > 0.0 { Some(round2(ab / (a * b).sqrt())) } else { None };
        let sec = |f: u64| f as f64 / fs;
        let [lr, ll, rr, llr, lll, lrr] = self.corr;
        Measurements {
            analyzer: VERSION,
            encoding,
            sample_rate: fs as u32,
            channels: nch as u16,
            bits,
            frames: total_frames,
            duration_s: (sec(total_frames) * 1000.0).round() / 1000.0,
            sample_peak_dbfs: round2(db(sample_peak)),
            channel_peaks_dbfs: self.peak.iter().map(|&p| round2(db(p))).collect(),
            true_peak_dbtp: round2(true_peak),
            rms_dbfs: round2(rms),
            crest_factor_db: round2(db(sample_peak) - rms),
            integrated_lufs: integrated.map(round2),
            plr_db: integrated.map(|l| round2(true_peak - l)),
            dc_offset: self.dc.iter().map(|&d| (d / frames * 1e5).round() / 1e5).collect(),
            clipped_samples: self.clipped,
            stereo_correlation: if nch == 2 { corr(ll, rr, lr) } else { None },
            low_band_correlation: if nch == 2 { corr(lll, lrr, llr) } else { None },
            band_energy: (self.all_e > 0.0).then(|| {
                let (l, hi) = ((self.low_e / self.all_e * 100.0).min(100.0), (self.high_e / self.all_e * 100.0).min(100.0));
                BandEnergy { low_pct: round2(l), mid_pct: round2((100.0 - l - hi).max(0.0)), high_pct: round2(hi) }
            }),
            leading_silence_s: round2(sec(self.first_loud.unwrap_or(total_frames))),
            trailing_silence_s: round2(self.first_loud.map_or(0.0, |_| sec(total_frames.saturating_sub(1 + self.last_loud)))),
        }
    }
}

/// Measure a WAV file. `cancelled` is polled regularly so long analyses can be stopped.
pub fn analyze(path: &Path, cancelled: &dyn Fn() -> bool) -> Result<Measurements, MeterError> {
    let file = File::open(path)?;
    let len = file.metadata()?.len();
    let mut r = BufReader::with_capacity(1 << 16, file);
    let h = header(&mut r, len)?;
    let nch = h.channels as usize;
    let bps = (h.bits / 8) as usize;
    let frame_bytes = bps * nch;
    let total_frames = h.data_bytes / frame_bytes as u64;
    let mut meter = Meter::new(h.rate, nch);
    let chunk_frames = 8192usize;
    let mut buf = vec![0u8; chunk_frames * frame_bytes];
    let mut frame = 0u64;
    let mut x = vec![0.0f64; nch];
    while frame < total_frames {
        if cancelled() {
            return Err(MeterError::Cancelled);
        }
        let n = ((total_frames - frame) as usize).min(chunk_frames);
        let bytes = &mut buf[..n * frame_bytes];
        r.read_exact(bytes).map_err(|_| MeterError::Invalid("WAV data ends early".into()))?;
        for f in 0..n {
            let base = f * frame_bytes;
            for (c, xv) in x.iter_mut().enumerate() {
                let s = &bytes[base + c * bps..base + (c + 1) * bps];
                *xv = match (h.enc, h.bits) {
                    (Enc::Int, 16) => i16::from_le_bytes([s[0], s[1]]) as f64 / 32768.0,
                    (Enc::Int, 24) => (((s[0] as i32) << 8 | (s[1] as i32) << 16 | (s[2] as i32) << 24) >> 8) as f64 / 8_388_608.0,
                    (Enc::Int, _) => i32::from_le_bytes([s[0], s[1], s[2], s[3]]) as f64 / 2_147_483_648.0,
                    (Enc::Float, 32) => f32::from_le_bytes([s[0], s[1], s[2], s[3]]) as f64,
                    _ => f64::from_le_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]]),
                };
            }
            meter.push(&x);
        }
        frame += n as u64;
    }
    Ok(meter.finish(if h.enc == Enc::Int { "pcm_int" } else { "float" }, h.bits))
}

/// Test helper: write a WAV file (16/24-bit int or 32-bit float) from interleaved samples.
#[cfg(test)]
pub fn write_test_wav(path: &Path, rate: u32, channels: u16, bits: u16, float: bool, samples: &[f64]) {
    use std::io::Write;
    let bps = (bits / 8) as u32;
    let data_len = samples.len() as u32 * bps;
    let mut out = Vec::new();
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&(if float { 3u16 } else { 1 }).to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * bps * channels as u32).to_le_bytes());
    out.extend_from_slice(&((bps * channels as u32) as u16).to_le_bytes());
    out.extend_from_slice(&bits.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for &s in samples {
        match (float, bits) {
            (true, _) => out.extend_from_slice(&(s as f32).to_le_bytes()),
            (false, 16) => out.extend_from_slice(&((s * 32767.0).round().clamp(-32768.0, 32767.0) as i16).to_le_bytes()),
            _ => out.extend_from_slice(&((s * 8_388_607.0).round() as i32).to_le_bytes()[..3]),
        }
    }
    std::fs::File::create(path).unwrap().write_all(&out).unwrap();
}

#[cfg(test)]
pub fn sine(rate: u32, secs: f64, freq: f64, amp: f64, phase: f64) -> Vec<f64> {
    let n = (rate as f64 * secs) as usize;
    (0..n).map(|i| amp * (2.0 * std::f64::consts::PI * freq * i as f64 / rate as f64 + phase).sin()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn never() -> bool {
        false
    }

    #[test]
    fn bs1770_reference_tones() {
        let dir = tempfile::tempdir().unwrap();
        // 0 dBFS 997 Hz sine on one channel reads -3.01 LKFS (BS.1770 definition)
        let p = dir.path().join("mono.wav");
        write_test_wav(&p, 48000, 1, 24, false, &sine(48000, 3.0, 997.0, 1.0, 0.0));
        let m = analyze(&p, &never).unwrap();
        assert!((m.integrated_lufs.unwrap() + 3.01).abs() < 0.05, "{:?}", m.integrated_lufs);
        assert!((m.sample_peak_dbfs).abs() < 0.01);
        assert!((m.rms_dbfs + 3.01).abs() < 0.02, "sine RMS = peak - 3.01 dB");
        assert!((m.crest_factor_db - 3.01).abs() < 0.02);
        // the same tone at -20 dBFS on both channels: -20 dB, +3.01 dB for two channels
        let s = sine(44100, 3.0, 997.0, 0.1, 0.0);
        let st: Vec<f64> = s.iter().flat_map(|&v| [v, v]).collect();
        let p = dir.path().join("stereo.wav");
        write_test_wav(&p, 44100, 2, 16, false, &st);
        let m = analyze(&p, &never).unwrap();
        assert!((m.integrated_lufs.unwrap() + 20.0).abs() < 0.06, "{:?}", m.integrated_lufs);
        assert_eq!(m.stereo_correlation, Some(1.0));
        assert_eq!((m.channels, m.sample_rate, m.bits, m.encoding), (2, 44100, 16, "pcm_int"));
        assert!((m.duration_s - 3.0).abs() < 0.001);
    }

    #[test]
    fn gating_ignores_silence() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = vec![0.0; 48000 * 2];
        s.extend(sine(48000, 2.0, 997.0, 1.0, 0.0));
        s.extend(vec![0.0; 48000]);
        let p = dir.path().join("gap.wav");
        write_test_wav(&p, 48000, 1, 24, false, &s);
        let m = analyze(&p, &never).unwrap();
        // 2 s of tone = 20 sub-blocks: 17 full 400 ms blocks plus 3 partial ones at each edge
        // (3/4, 1/2, 1/4 of the power), all above the -10 LU gate: mean power 20/23 -> -3.62 LKFS.
        // Without the absolute gate the silent blocks would count too: 20/47 -> -6.72 LKFS.
        assert!((m.integrated_lufs.unwrap() + 3.62).abs() < 0.05, "silent blocks are gated out: {:?}", m.integrated_lufs);
        assert!((m.leading_silence_s - 2.0).abs() < 0.01 && (m.trailing_silence_s - 1.0).abs() < 0.01, "{} {}", m.leading_silence_s, m.trailing_silence_s);
    }

    #[test]
    fn true_peak_finds_inter_sample_overs() {
        // fs/4 sine at 45°: every sample sits at ±0.707 (-3.01 dBFS) but the waveform reaches 1.0
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("isp.wav");
        write_test_wav(&p, 48000, 1, 32, true, &sine(48000, 1.0, 12000.0, 1.0, std::f64::consts::FRAC_PI_4));
        let m = analyze(&p, &never).unwrap();
        assert!((m.sample_peak_dbfs + 3.01).abs() < 0.02, "{}", m.sample_peak_dbfs);
        assert!(m.true_peak_dbtp > -0.3 && m.true_peak_dbtp < 0.2, "{}", m.true_peak_dbtp);
        assert_eq!(m.encoding, "float");
    }

    #[test]
    fn stereo_correlation_and_clipping() {
        let dir = tempfile::tempdir().unwrap();
        let s = sine(48000, 1.0, 440.0, 1.2, 0.0);
        let anti: Vec<f64> = s.iter().flat_map(|&v| [v.clamp(-1.0, 1.0), -v.clamp(-1.0, 1.0)]).collect();
        let p = dir.path().join("anti.wav");
        write_test_wav(&p, 48000, 2, 16, false, &anti);
        let m = analyze(&p, &never).unwrap();
        assert_eq!(m.stereo_correlation, Some(-1.0));
        assert!(m.clipped_samples > 0);
        assert!(matches!(analyze(&p, &|| true), Err(MeterError::Cancelled)));
    }

    #[test]
    fn rejects_non_wav_and_unsupported() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("x.wav");
        std::fs::write(&p, b"ID3 this is an mp3").unwrap();
        assert!(matches!(analyze(&p, &never), Err(MeterError::Invalid(_))));
        write_test_wav(&p, 48000, 1, 16, false, &[0.0; 10]);
        let mut b = std::fs::read(&p).unwrap();
        b[20] = 2; // ADPCM format code
        std::fs::write(&p, b).unwrap();
        assert!(matches!(analyze(&p, &never), Err(MeterError::Invalid(m)) if m.contains("unsupported")));
    }
}
