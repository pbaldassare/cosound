//! Standard MIDI File writer (format 1, 480 PPQ). Track 1 holds name, tempo, 4/4 and key
//! signature; every part gets its own track with name, program change and notes.
use super::generate::{Part, PPQ};
use super::theory::Key;

fn vlq(out: &mut Vec<u8>, mut v: u32) {
    let mut buf = [0u8; 5];
    let mut i = 4;
    buf[i] = (v & 0x7F) as u8;
    v >>= 7;
    while v > 0 {
        i -= 1;
        buf[i] = (v & 0x7F) as u8 | 0x80;
        v >>= 7;
    }
    out.extend_from_slice(&buf[i..]);
}

fn meta(t: &mut Vec<u8>, kind: u8, data: &[u8]) {
    t.extend_from_slice(&[0, 0xFF, kind]);
    vlq(t, data.len() as u32);
    t.extend_from_slice(data);
}

fn chunk(out: &mut Vec<u8>, mut body: Vec<u8>) {
    body.extend_from_slice(&[0, 0xFF, 0x2F, 0]); // end of track
    out.extend_from_slice(b"MTrk");
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(&body);
}

fn ascii(s: &str) -> Vec<u8> {
    s.chars().map(|c| if c.is_ascii() && !c.is_ascii_control() { c as u8 } else { b'?' }).take(120).collect()
}

pub fn write(parts: &[Part], bpm: f64, key: Key, title: &str) -> Vec<u8> {
    let mut out = b"MThd".to_vec();
    out.extend_from_slice(&6u32.to_be_bytes());
    out.extend_from_slice(&1u16.to_be_bytes());
    out.extend_from_slice(&((parts.len() + 1) as u16).to_be_bytes());
    out.extend_from_slice(&(PPQ as u16).to_be_bytes());

    let mut t = Vec::new();
    meta(&mut t, 0x03, &ascii(title));
    meta(&mut t, 0x58, &[4, 2, 24, 8]);
    let us = (60_000_000.0 / bpm).round() as u32;
    meta(&mut t, 0x51, &us.to_be_bytes()[1..]);
    meta(&mut t, 0x59, &[key.signature().clamp(-7, 7) as i8 as u8, key.mode.minor_third() as u8]);
    chunk(&mut out, t);

    for p in parts {
        let mut t = Vec::new();
        meta(&mut t, 0x03, &ascii(p.name));
        if p.channel != 9 {
            t.extend_from_slice(&[0, 0xC0 | p.channel, p.program]);
        }
        let mut ev: Vec<(u32, u8, u8, u8)> = Vec::with_capacity(p.notes.len() * 2); // (tick, on?, pitch, vel)
        for n in &p.notes {
            ev.push((n.start, 1, n.pitch, n.vel.max(1)));
            ev.push((n.start + n.dur, 0, n.pitch, 0));
        }
        ev.sort_by_key(|e| (e.0, e.1, e.2)); // note-offs before note-ons on the same tick
        let mut last = 0;
        for (tick, on, pitch, vel) in ev {
            vlq(&mut t, tick - last);
            last = tick;
            t.extend_from_slice(&[if on == 1 { 0x90 } else { 0x80 } | p.channel, pitch & 0x7F, vel & 0x7F]);
        }
        chunk(&mut out, t);
    }
    out
}

/// A track for the MIDI Generation Engine: any name, channel, optional program change, notes and controller
/// events (tick, CC number, value), e.g. CC74 filter or CC1 modulation automation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Track {
    pub name: String,
    pub channel: u8,
    pub program: Option<u8>,
    pub notes: Vec<super::generate::MidiNote>,
    #[serde(default)]
    pub cc: Vec<(u32, u8, u8)>,
}

/// Format-1 file with a conductor track (title, tempo, time and key signature) and one track per part.
pub fn write_tracks(tracks: &[Track], bpm: f64, key: Key, title: &str, time_sig: (u8, u8)) -> Vec<u8> {
    let mut out = b"MThd".to_vec();
    out.extend_from_slice(&6u32.to_be_bytes());
    out.extend_from_slice(&1u16.to_be_bytes());
    out.extend_from_slice(&((tracks.len() + 1) as u16).to_be_bytes());
    out.extend_from_slice(&(PPQ as u16).to_be_bytes());
    let mut t = Vec::new();
    meta(&mut t, 0x03, &ascii(title));
    let den_pow = (time_sig.1.max(1) as f64).log2().round() as u8;
    meta(&mut t, 0x58, &[time_sig.0, den_pow, 24, 8]);
    let us = (60_000_000.0 / bpm).round() as u32;
    meta(&mut t, 0x51, &us.to_be_bytes()[1..]);
    meta(&mut t, 0x59, &[key.signature().clamp(-7, 7) as i8 as u8, key.mode.minor_third() as u8]);
    chunk(&mut out, t);
    for tr in tracks {
        let ch = tr.channel & 0x0F;
        let mut t = Vec::new();
        meta(&mut t, 0x03, &ascii(&tr.name));
        if let (Some(p), false) = (tr.program, ch == 9) {
            t.extend_from_slice(&[0, 0xC0 | ch, p & 0x7F]);
        }
        // (tick, order, status, a, b): controllers before note-offs before note-ons on the same tick
        let mut ev: Vec<(u32, u8, u8, u8, u8)> = Vec::with_capacity(tr.notes.len() * 2 + tr.cc.len());
        for (tick, cc, v) in &tr.cc {
            ev.push((*tick, 0, 0xB0 | ch, cc & 0x7F, v & 0x7F));
        }
        for n in &tr.notes {
            ev.push((n.start, 2, 0x90 | ch, n.pitch & 0x7F, n.vel.clamp(1, 127)));
            ev.push((n.start + n.dur.max(1), 1, 0x80 | ch, n.pitch & 0x7F, 0));
        }
        ev.sort_by_key(|e| (e.0, e.1, e.3));
        let mut last = 0;
        for (tick, _, st, a, b) in ev {
            vlq(&mut t, tick - last);
            last = tick;
            t.extend_from_slice(&[st, a, b]);
        }
        chunk(&mut out, t);
    }
    out
}

/// Minimal reader used by the tests to prove the file is well formed (supports running status).
#[cfg(test)]
pub fn read(bytes: &[u8]) -> Result<(u16, u16, Vec<Vec<(u32, u8, u8, u8)>>, Option<u32>), String> {
    let be16 = |b: &[u8]| u16::from_be_bytes([b[0], b[1]]);
    let be32 = |b: &[u8]| u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
    if bytes.len() < 14 || &bytes[0..4] != b"MThd" || be32(&bytes[4..8]) != 6 {
        return Err("bad header".into());
    }
    let (format, ntrks, division) = (be16(&bytes[8..10]), be16(&bytes[10..12]), be16(&bytes[12..14]));
    let mut pos = 14;
    let mut tracks = Vec::new();
    let mut tempo = None;
    for _ in 0..ntrks {
        if &bytes[pos..pos + 4] != b"MTrk" {
            return Err("bad track".into());
        }
        let len = be32(&bytes[pos + 4..pos + 8]) as usize;
        let body = &bytes[pos + 8..pos + 8 + len];
        pos += 8 + len;
        let (mut i, mut tick, mut status, mut evs, mut ended) = (0usize, 0u32, 0u8, Vec::new(), false);
        while i < body.len() {
            let mut d = 0u32;
            loop {
                let b = body[i];
                i += 1;
                d = (d << 7) | (b & 0x7F) as u32;
                if b & 0x80 == 0 {
                    break;
                }
            }
            tick += d;
            if body[i] & 0x80 != 0 {
                status = body[i];
                i += 1;
            }
            match status {
                0xFF => {
                    let kind = body[i];
                    let l = body[i + 1] as usize;
                    if kind == 0x51 {
                        tempo = Some(u32::from_be_bytes([0, body[i + 2], body[i + 3], body[i + 4]]));
                    }
                    if kind == 0x2F {
                        ended = true;
                    }
                    i += 2 + l;
                }
                s if s & 0xF0 == 0xC0 => i += 1,
                s if s & 0xF0 == 0xB0 => {
                    evs.push((tick, s, body[i], body[i + 1]));
                    i += 2;
                }
                s if s & 0xF0 == 0x80 || s & 0xF0 == 0x90 => {
                    evs.push((tick, s, body[i], body[i + 1]));
                    i += 2;
                }
                s => return Err(format!("unexpected status {s:#x}")),
            }
        }
        if !ended {
            return Err("missing end of track".into());
        }
        tracks.push(evs);
    }
    if pos != bytes.len() {
        return Err("trailing bytes".into());
    }
    let _ = format;
    Ok((format, division, tracks, tempo))
}

#[cfg(test)]
mod tests {
    use super::super::generate::*;
    use super::super::theory::{progression, Key};
    use super::*;

    #[test]
    fn writes_a_valid_format_1_file() {
        let key = Key::parse("A minor").unwrap();
        let ch = progression(key, "i-VI-III-VII", false).unwrap();
        let spec = Spec { key, chords: &ch, bars: 8, seed: 3, swing: 20 };
        let parts = vec![chords(&spec, "sustain").unwrap(), bass(&spec, "rolling").unwrap(), melody(&spec), drums(&spec, "melodic_techno", true).unwrap()];
        let bytes = write(&parts, 124.0, key, "COSOUND test – àccent");
        let (format, division, tracks, tempo) = read(&bytes).unwrap();
        assert_eq!((format, division, tracks.len()), (1, 480, 5));
        assert_eq!(tempo, Some(483_871)); // 60e6 / 124
        for (t, p) in tracks[1..].iter().zip(&parts) {
            let on = t.iter().filter(|e| e.1 & 0xF0 == 0x90).count();
            let off = t.iter().filter(|e| e.1 & 0xF0 == 0x80).count();
            assert_eq!((on, off), (p.notes.len(), p.notes.len()), "{}", p.id);
            assert!(t.iter().all(|e| e.1 & 0x0F == p.channel), "{} channel", p.id);
            // every note-on is closed by a note-off on the same pitch before it sounds again
            let mut open = std::collections::HashSet::new();
            for e in t {
                if e.1 & 0xF0 == 0x90 {
                    assert!(open.insert(e.2), "{} pitch {} re-triggered while sounding", p.id, e.2);
                } else {
                    assert!(open.remove(&e.2), "{} note-off without note-on", p.id);
                }
            }
            assert!(open.is_empty());
        }
    }

    #[test]
    fn variable_length_quantities() {
        for (v, enc) in [(0u32, vec![0x00]), (0x7F, vec![0x7F]), (0x80, vec![0x81, 0x00]), (0x3FFF, vec![0xFF, 0x7F]), (0x0FFF_FFFF, vec![0xFF, 0xFF, 0xFF, 0x7F])] {
            let mut o = Vec::new();
            vlq(&mut o, v);
            assert_eq!(o, enc, "{v:#x}");
        }
    }
}
