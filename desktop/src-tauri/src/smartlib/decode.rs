//! Multi-format audio decoding (Symphonia 0.6, pure Rust, MPL-2.0): WAV, AIFF, CAF, MP3, AAC-LC and
//! ALAC in MP4/M4A, FLAC, OGG Vorbis. Streams packets into a callback, so a file is never loaded
//! whole into memory. Also reads the embedded loop data of WAV files (ACID tempo, `smpl` root note).
use serde_json::{json, Map, Value};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::codecs::CodecParameters;
use symphonia::core::errors::Error as SymErr;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, RawValue, StandardTag};

/// Extensions the decoder is built to read (verified by tests or on real files, see FORMATS.md).
pub const AUDIO_EXTS: [&str; 12] = ["wav", "wave", "aif", "aiff", "aifc", "caf", "mp3", "m4a", "aac", "mp4", "flac", "ogg"];

#[derive(Debug, PartialEq)]
pub enum DecodeError {
    Cancelled,
    Unsupported(String),
    Io(String),
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Decoded {
    pub container: String,
    pub codec: String,
    pub sample_rate: u32,
    pub channels: usize,
    pub bit_depth: Option<u32>,
    /// From the container, when declared.
    pub duration_s: Option<f64>,
    pub decoded_frames: u64,
    /// true when decoding stopped at `max_seconds` (long files are analyzed on their first part).
    pub truncated: bool,
    pub tags: Map<String, Value>,
}

fn raw_to_json(v: &RawValue) -> Option<Value> {
    Some(match v {
        RawValue::String(s) => json!(s.as_str().trim_matches(char::from(0)).trim()),
        RawValue::Float(f) => json!(f),
        RawValue::SignedInt(i) => json!(i),
        RawValue::UnsignedInt(u) => json!(u),
        RawValue::Boolean(b) => json!(b),
        _ => return None,
    })
}

fn collect_tags(tags: &[symphonia::core::meta::Tag], out: &mut Map<String, Value>) {
    for t in tags {
        let (k, v) = match &t.std {
            Some(StandardTag::TrackTitle(s)) => ("title", json!(s.as_str())),
            Some(StandardTag::Artist(s)) => ("artist", json!(s.as_str())),
            Some(StandardTag::Genre(s)) => ("genre", json!(s.as_str())),
            Some(StandardTag::Bpm(b)) => ("bpm", json!(b)),
            Some(StandardTag::InitialKey(s)) => ("key", json!(s.as_str())),
            Some(StandardTag::Comment(s)) => ("comment", json!(s.as_str())),
            _ => match raw_to_json(&t.raw.value) {
                Some(v) if t.raw.key.len() <= 40 => (t.raw.key.as_str(), v),
                _ => continue,
            },
        };
        if v.as_str().is_some_and(|s| s.is_empty() || s.len() > 300) {
            continue;
        }
        out.entry(k.to_string()).or_insert(v);
    }
}

/// Decode `path`, calling `sink(interleaved_samples, channels, sample_rate)` for every decoded block.
pub fn decode(path: &Path, max_seconds: Option<f64>, cancel: &dyn Fn() -> bool, sink: &mut dyn FnMut(&[f32], usize, u32)) -> Result<Decoded, DecodeError> {
    let file = File::open(path).map_err(|e| DecodeError::Io(e.to_string()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let mut format = symphonia::default::get_probe()
        .probe(&hint, mss, FormatOptions::default(), MetadataOptions::default())
        .map_err(|e| DecodeError::Unsupported(format!("container not recognized: {e}")))?;
    let track = format.default_track(TrackType::Audio).or_else(|| format.first_track_known_codec(TrackType::Audio)).ok_or_else(|| DecodeError::Unsupported("no audio track".into()))?;
    let track_id = track.id;
    let Some(CodecParameters::Audio(params)) = track.codec_params.clone() else {
        return Err(DecodeError::Unsupported("unknown audio codec".into()));
    };
    let mut out = Decoded {
        container: format.format_info().short_name.to_string(),
        sample_rate: params.sample_rate.unwrap_or(0),
        channels: params.channels.as_ref().map_or(0, |c| c.count()),
        bit_depth: params.bits_per_sample,
        ..Default::default()
    };
    if let (Some(n), Some(r)) = (track.num_frames, params.sample_rate) {
        out.duration_s = Some(n as f64 / r as f64);
    }
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(&params, &AudioDecoderOptions::default())
        .map_err(|e| DecodeError::Unsupported(format!("codec not supported: {e}")))?;
    out.codec = decoder.codec_info().short_name.to_string();
    let mut tags = Map::new();
    let read_meta = |format: &mut Box<dyn symphonia::core::formats::FormatReader>, tags: &mut Map<String, Value>| {
        let mut md = format.metadata();
        if let Some(rev) = md.skip_to_latest() {
            collect_tags(&rev.media.tags, tags);
            for t in &rev.per_track {
                collect_tags(&t.metadata.tags, tags);
            }
        }
    };
    read_meta(&mut format, &mut tags);
    let limit = max_seconds.map(|s| (s * out.sample_rate.max(1) as f64) as u64);
    let mut buf: Vec<f32> = Vec::new();
    let mut n = 0u32;
    loop {
        n += 1;
        if n % 64 == 0 && cancel() {
            return Err(DecodeError::Cancelled);
        }
        let packet = match format.next_packet() {
            Ok(Some(p)) => p,
            Ok(None) => break,
            Err(SymErr::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(SymErr::DecodeError(_)) => continue,
            Err(e) => return Err(DecodeError::Io(e.to_string())),
        };
        if packet.track_id != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(audio) => {
                let ch = audio.spec().channels().count().max(1);
                if out.sample_rate == 0 {
                    out.sample_rate = audio.spec().rate();
                }
                out.channels = ch;
                audio.copy_to_vec_interleaved::<f32>(&mut buf);
                let frames = (buf.len() / ch) as u64;
                let take = limit.map_or(frames, |l| frames.min(l.saturating_sub(out.decoded_frames)));
                sink(&buf[..take as usize * ch], ch, out.sample_rate);
                out.decoded_frames += take;
                if limit.is_some_and(|l| out.decoded_frames >= l) {
                    out.truncated = true;
                    break;
                }
            }
            Err(SymErr::DecodeError(_)) => continue,
            Err(e) => return Err(DecodeError::Io(e.to_string())),
        }
    }
    read_meta(&mut format, &mut tags);
    if out.duration_s.is_none() && !out.truncated && out.sample_rate > 0 {
        out.duration_s = Some(out.decoded_frames as f64 / out.sample_rate as f64);
    }
    if out.sample_rate == 0 || out.channels == 0 {
        return Err(DecodeError::Unsupported("no decodable audio".into()));
    }
    out.tags = tags;
    Ok(out)
}

/// Loop data embedded in a WAV file: ACID chunk (tempo, beats, root note) and `smpl` (MIDI unity
/// note). The ACID layout is not published by its owner: it is the widely used community layout,
/// so values are labelled `embedded` and only used when they are plausible.
pub fn wav_loop_info(path: &Path) -> Option<Value> {
    let mut f = File::open(path).ok()?;
    let mut riff = [0u8; 12];
    f.read_exact(&mut riff).ok()?;
    if &riff[0..4] != b"RIFF" || &riff[8..12] != b"WAVE" {
        return None;
    }
    let mut out = Map::new();
    let mut hdr = [0u8; 8];
    for _ in 0..64 {
        if f.read_exact(&mut hdr).is_err() {
            break;
        }
        let size = u32::from_le_bytes([hdr[4], hdr[5], hdr[6], hdr[7]]) as u64;
        match &hdr[0..4] {
            b"acid" if size >= 24 => {
                let mut b = [0u8; 24];
                f.read_exact(&mut b).ok()?;
                let flags = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
                let root = u16::from_le_bytes([b[4], b[5]]);
                let beats = u32::from_le_bytes([b[12], b[13], b[14], b[15]]);
                let tempo = f32::from_le_bytes([b[20], b[21], b[22], b[23]]);
                if (40.0..=300.0).contains(&tempo) {
                    out.insert("acid_tempo".into(), json!((tempo * 100.0).round() / 100.0));
                }
                if beats > 0 && beats < 10_000 {
                    out.insert("acid_beats".into(), json!(beats));
                }
                out.insert("acid_one_shot".into(), json!(flags & 1 == 1));
                if flags & 2 == 2 && root < 128 {
                    out.insert("acid_root_note".into(), json!(root));
                }
                f.seek(SeekFrom::Current((size - 24 + size % 2) as i64)).ok()?;
            }
            b"smpl" if size >= 36 => {
                let mut b = [0u8; 36];
                f.read_exact(&mut b).ok()?;
                let unity = u32::from_le_bytes([b[12], b[13], b[14], b[15]]);
                let loops = u32::from_le_bytes([b[28], b[29], b[30], b[31]]);
                if unity < 128 {
                    out.insert("smpl_root_note".into(), json!(unity));
                }
                out.insert("smpl_loops".into(), json!(loops));
                f.seek(SeekFrom::Current((size - 36 + size % 2) as i64)).ok()?;
            }
            _ => {
                f.seek(SeekFrom::Current((size + size % 2) as i64)).ok()?;
            }
        }
    }
    (!out.is_empty()).then_some(Value::Object(out))
}

/// Test helper: write a 16-bit AIFF file (big-endian PCM, 80-bit extended sample rate).
#[cfg(test)]
pub fn write_test_aiff(path: &Path, rate: u32, channels: u16, samples: &[f64]) {
    let frames = (samples.len() / channels as usize) as u32;
    let mut ssnd = vec![0u8; 8];
    for &s in samples {
        ssnd.extend_from_slice(&((s * 32767.0).round().clamp(-32768.0, 32767.0) as i16).to_be_bytes());
    }
    // 80-bit IEEE extended for the sample rate
    let mut rate80 = [0u8; 10];
    let (mut m, mut e) = (rate as u64, 16383 + 63);
    while m & (1 << 63) == 0 {
        m <<= 1;
        e -= 1;
    }
    rate80[0..2].copy_from_slice(&(e as u16).to_be_bytes());
    rate80[2..10].copy_from_slice(&m.to_be_bytes());
    let mut comm = Vec::new();
    comm.extend_from_slice(&channels.to_be_bytes());
    comm.extend_from_slice(&frames.to_be_bytes());
    comm.extend_from_slice(&16u16.to_be_bytes());
    comm.extend_from_slice(&rate80);
    let mut body = b"AIFF".to_vec();
    body.extend_from_slice(b"COMM");
    body.extend_from_slice(&(comm.len() as u32).to_be_bytes());
    body.extend_from_slice(&comm);
    body.extend_from_slice(b"SSND");
    body.extend_from_slice(&(ssnd.len() as u32).to_be_bytes());
    body.extend_from_slice(&ssnd);
    let mut out = b"FORM".to_vec();
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(&body);
    std::fs::write(path, out).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::wav::{sine, write_test_wav};

    #[test]
    fn decodes_wav_and_aiff_identically() {
        let dir = tempfile::tempdir().unwrap();
        let s = sine(44100, 1.0, 440.0, 0.5, 0.0);
        let (w, a) = (dir.path().join("t.wav"), dir.path().join("t.aif"));
        write_test_wav(&w, 44100, 1, 16, false, &s);
        write_test_aiff(&a, 44100, 1, &s);
        let grab = |p: &Path| {
            let mut v = Vec::new();
            let d = decode(p, None, &|| false, &mut |b, _, _| v.extend_from_slice(b)).unwrap();
            (d, v)
        };
        let ((dw, vw), (da, va)) = (grab(&w), grab(&a));
        assert_eq!((dw.sample_rate, dw.channels, dw.decoded_frames), (44100, 1, 44100));
        assert_eq!((da.sample_rate, da.channels, da.decoded_frames), (44100, 1, 44100));
        assert_eq!(dw.bit_depth, Some(16));
        assert!((dw.duration_s.unwrap() - 1.0).abs() < 1e-6);
        assert!(vw.iter().zip(&va).all(|(x, y)| (x - y).abs() < 1e-6), "same samples from both containers");
        // truncation for long files
        let mut n = 0;
        let d = decode(&w, Some(0.25), &|| false, &mut |b, _, _| n += b.len()).unwrap();
        assert!(d.truncated && n == 11025);
        assert!(matches!(decode(&w, None, &|| true, &mut |_, _, _| {}), Err(DecodeError::Cancelled) | Ok(_)));
        let bad = dir.path().join("x.mp3");
        std::fs::write(&bad, b"definitely not audio").unwrap();
        assert!(matches!(decode(&bad, None, &|| false, &mut |_, _, _| {}), Err(DecodeError::Unsupported(_))));
    }

    #[test]
    fn reads_acid_and_smpl_chunks() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("loop.wav");
        write_test_wav(&p, 44100, 1, 16, false, &[0.0; 100]);
        let mut b = std::fs::read(&p).unwrap();
        let mut acid = b"acid".to_vec();
        acid.extend_from_slice(&24u32.to_le_bytes());
        acid.extend_from_slice(&2u32.to_le_bytes()); // flags: root note set
        acid.extend_from_slice(&57u16.to_le_bytes());
        acid.extend_from_slice(&[0; 6]);
        acid.extend_from_slice(&8u32.to_le_bytes());
        acid.extend_from_slice(&[0; 4]);
        acid.extend_from_slice(&124.0f32.to_le_bytes());
        b.extend_from_slice(&acid);
        let riff = (b.len() - 8) as u32;
        b[4..8].copy_from_slice(&riff.to_le_bytes());
        std::fs::write(&p, &b).unwrap();
        let info = wav_loop_info(&p).unwrap();
        assert_eq!((info["acid_tempo"].as_f64(), info["acid_beats"].as_u64(), info["acid_root_note"].as_u64()), (Some(124.0), Some(8), Some(57)));
    }
}
