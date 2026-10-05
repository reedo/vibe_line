//! Original development music and sample-accurate chart. Run from the repository root.
//! No dependencies or samples. See assets/audio/PROVENANCE.txt.
use std::{f32::consts::TAU, fmt::Write as _, fs, io::Write as _, path::Path};

const RATE: usize = 48_000;
const BEAT: usize = RATE / 2; // 120 BPM
const BAR: usize = BEAT * 4;
const COUNT_IN: usize = BAR * 2;
const MUSIC_END: usize = COUNT_IN + BAR * 32;
const FRAMES: usize = MUSIC_END + RATE * 2;
const SEED: u32 = 0x51A7_2026;

#[derive(Clone, Copy)]
struct Event {
    frame: usize,
    mask: u8,
}

struct Noise(u32);
impl Noise {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f64 / u32::MAX as f64 * 2.0 - 1.0) as f32
    }
}

fn hz(note: i32) -> f32 {
    440.0 * 2.0f32.powf((note - 69) as f32 / 12.0)
}

fn add<F: FnMut(f32) -> f32>(out: &mut [f32], at: usize, seconds: f32, mut voice: F) {
    let count = ((seconds * RATE as f32) as usize).min(out.len().saturating_sub(at));
    for i in 0..count {
        let t = i as f32 / RATE as f32;
        // A short release also makes drum and percussion truncation click-free.
        let release = ((count - i) as f32 / (RATE as f32 * 0.008)).min(1.0);
        out[at + i] += voice(t) * release;
    }
}

fn pluck(out: &mut [f32], at: usize, note: i32, length: f32, gain: f32, bright: bool) {
    let frequency = hz(note);
    add(out, at, length, |t| {
        let phase = TAU * frequency * t;
        let harmonics = if bright {
            phase.sin() + 0.25 * (2.0 * phase).sin() + 0.12 * (3.0 * phase).sin()
        } else {
            phase.sin() + 0.12 * (2.0 * phase).sin()
        };
        let attack = (t / 0.004).min(1.0);
        gain * attack * (-t * 6.0 / length).exp() * harmonics
    });
}

fn kick(out: &mut [f32], at: usize, gain: f32) {
    add(out, at, 0.24, |t| {
        // Integral of a falling frequency, rather than frequency multiplied by time.
        let phase = TAU * (47.0 * t + 70.0 / 32.0 * (1.0 - (-32.0 * t).exp()));
        gain * phase.sin() * (-19.0 * t).exp() * (t / 0.002).min(1.0)
    });
}

fn snare(out: &mut [f32], at: usize, gain: f32, noise: &mut Noise) {
    let mut previous = 0.0;
    add(out, at, 0.16, |t| {
        let white = noise.next();
        let high = white - previous * 0.65;
        previous = white;
        gain * (0.65 * high + 0.35 * (TAU * 185.0 * t).sin())
            * (-30.0 * t).exp()
            * (t / 0.001).min(1.0)
    });
}

fn hat(out: &mut [f32], at: usize, gain: f32, noise: &mut Noise) {
    let mut previous = 0.0;
    add(out, at, 0.055, |t| {
        let white = noise.next();
        let high = white - previous;
        previous = white;
        gain * high * (-85.0 * t).exp() * (t / 0.001).min(1.0)
    });
}

fn accent(out: &mut [f32], event: Event) {
    // Four recognisable timbres are layered when a chart event needs two buttons.
    if event.mask & 1 != 0 {
        pluck(out, event.frame, 86, 0.16, 0.30, true);
    }
    if event.mask & 2 != 0 {
        add(out, event.frame, 0.19, |t| {
            0.27 * (TAU * (650.0 * t + 1500.0 * t * t)).sin()
                * (-22.0 * t).exp()
                * (t / 0.003).min(1.0)
        });
    }
    if event.mask & 4 != 0 {
        add(out, event.frame, 0.21, |t| {
            0.27 * (TAU * 370.0 * t + 1.4 * (TAU * 23.0 * t).sin()).sin()
                * (-20.0 * t).exp()
                * (t / 0.003).min(1.0)
        });
    }
    if event.mask & 8 != 0 {
        pluck(out, event.frame, 50, 0.24, 0.34, true);
    }
}

fn chart() -> Vec<Event> {
    let mut events = Vec::new();
    for bar in 0..16 {
        for beat in [0, 2] {
            events.push(Event {
                frame: COUNT_IN + bar * BAR + beat * BEAT,
                mask: 1 << (bar / 4),
            });
        }
    }
    let singles = [1, 4, 2, 8, 4, 1, 8, 2];
    for bar in 16..24 {
        for (i, eighth) in [0, 3, 6].iter().enumerate() {
            events.push(Event {
                frame: COUNT_IN + bar * BAR + eighth * BEAT / 2,
                mask: singles[((bar - 16) * 3 + i) % singles.len()],
            });
        }
    }
    for (i, mask) in [3, 5, 9, 6, 10, 12].iter().enumerate() {
        for beat in [0, 2] {
            events.push(Event {
                frame: COUNT_IN + (26 + i) * BAR + beat * BEAT,
                mask: *mask,
            });
        }
    }
    events
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let events = chart();
    assert_eq!(events.len(), 68);
    assert!(events.windows(2).all(|e| e[0].frame < e[1].frame));
    assert!(events.iter().all(|e| e.frame >= COUNT_IN
        && e.frame < MUSIC_END
        && e.mask > 0
        && e.mask < 16
        && e.mask.count_ones() <= 2));
    let mut out = vec![0.0f32; FRAMES];
    let mut noise = Noise(SEED);
    for beat in 0..8 {
        pluck(
            &mut out,
            beat * BEAT,
            if beat % 4 == 0 { 86 } else { 81 },
            0.065,
            0.15,
            false,
        );
    }
    // Original D-major / B-minor / G-major / A-major progression and motif.
    let chords = [[50, 54, 57], [47, 50, 54], [43, 47, 50], [45, 49, 52]];
    let melody = [
        74, 78, 81, 78, 76, 74, 69, 73, 74, 81, 78, 76, 73, 69, 71, 73,
    ];
    for bar in 0..32 {
        let start = COUNT_IN + bar * BAR;
        let chord = chords[(bar / 2) % chords.len()];
        let is_rest = (24..26).contains(&bar);
        // The rest section keeps quiet harmony and pulse but has no required inputs.
        let energy = if is_rest { 0.45 } else { 1.0 };
        for (voice, note) in chord.iter().enumerate() {
            pluck(
                &mut out,
                start + voice * 180,
                note + 12,
                1.6,
                0.085 * energy,
                false,
            );
            if !is_rest {
                pluck(
                    &mut out,
                    start + BEAT * 2 + voice * 180,
                    note + 12,
                    0.8,
                    0.055,
                    false,
                );
            }
        }
        for beat in 0..4 {
            let at = start + beat * BEAT;
            if beat % 2 == 0 {
                kick(&mut out, at, 0.34 * energy);
            }
            if beat % 2 == 1 && !is_rest {
                snare(&mut out, at, 0.15, &mut noise);
            }
            let bass = if beat == 3 {
                chord[2] - 12
            } else {
                chord[0] - 12
            };
            pluck(&mut out, at, bass, 0.39, 0.18 * energy, false);
        }
        if !is_rest {
            for eighth in 0..8 {
                hat(
                    &mut out,
                    start + eighth * BEAT / 2,
                    if eighth % 2 == 0 { 0.030 } else { 0.045 },
                    &mut noise,
                );
            }
            for (i, eighth) in [1, 3, 5, 7].iter().enumerate() {
                let note = melody[(bar * 4 + i) % melody.len()];
                // The answering phrase moves to the lower register every four bars.
                let octave = if (bar / 4) % 2 == 0 { 0 } else { -12 };
                pluck(
                    &mut out,
                    start + eighth * BEAT / 2,
                    note + octave,
                    0.27,
                    0.11,
                    true,
                );
            }
        }
    }
    for &event in &events {
        accent(&mut out, event);
    }
    for note in [50, 62, 66, 69, 74] {
        pluck(&mut out, MUSIC_END, note, 1.8, 0.1, false);
    }
    // A very quiet slapback makes the dry synthesizer easier to listen to.
    let delay = BEAT * 3 / 4;
    for i in (delay..FRAMES).rev() {
        out[i] += out[i - delay] * 0.11;
    }
    let fade = RATE / 10;
    for i in 0..fade {
        out[FRAMES - fade + i] *= (fade - i - 1) as f32 / fade as f32;
    }
    let peak = out.iter().copied().map(f32::abs).fold(0.0, f32::max);
    assert!(peak.is_finite() && peak > 0.0);
    let gain = 0.88 / peak;
    let pcm: Vec<i16> = out
        .iter()
        .map(|s| (s * gain * i16::MAX as f32).round() as i16)
        .collect();
    let rms = (pcm
        .iter()
        .map(|s| (*s as f64 / i16::MAX as f64).powi(2))
        .sum::<f64>()
        / FRAMES as f64)
        .sqrt();
    assert!(rms > 0.01 && rms < 0.4);
    assert_eq!(*pcm.last().unwrap(), 0);
    fs::create_dir_all("assets/audio")?;
    fs::create_dir_all("assets/charts")?;
    let mut wav = fs::File::create("assets/audio/line_test_120.wav")?;
    let bytes = (pcm.len() * 2) as u32;
    wav.write_all(b"RIFF")?;
    wav.write_all(&(36 + bytes).to_le_bytes())?;
    wav.write_all(b"WAVEfmt ")?;
    wav.write_all(&16u32.to_le_bytes())?;
    for value in [1u16, 1u16] {
        wav.write_all(&value.to_le_bytes())?;
    }
    for value in [RATE as u32, RATE as u32 * 2] {
        wav.write_all(&value.to_le_bytes())?;
    }
    for value in [2u16, 16u16] {
        wav.write_all(&value.to_le_bytes())?;
    }
    wav.write_all(b"data")?;
    wav.write_all(&bytes.to_le_bytes())?;
    let mut payload = Vec::with_capacity(bytes as usize);
    for sample in &pcm {
        payload.extend_from_slice(&sample.to_le_bytes());
    }
    wav.write_all(&payload)?;
    let sections = [
        ("count_in", 0, COUNT_IN),
        ("block", COUNT_IN, COUNT_IN + BAR * 4),
        ("loop", COUNT_IN + BAR * 4, COUNT_IN + BAR * 8),
        ("wave", COUNT_IN + BAR * 8, COUNT_IN + BAR * 12),
        ("pit", COUNT_IN + BAR * 12, COUNT_IN + BAR * 16),
        ("mixed_singles", COUNT_IN + BAR * 16, COUNT_IN + BAR * 24),
        ("rest", COUNT_IN + BAR * 24, COUNT_IN + BAR * 26),
        ("pairs", COUNT_IN + BAR * 26, MUSIC_END),
        ("tail", MUSIC_END, FRAMES),
    ];
    let mut json = format!(
        "{{\n  \"version\": 1,\n  \"title\": \"Vibe String - Thread Test 120\",\n  \"sample_rate\": {RATE},\n  \"bpm\": 120,\n  \"beats_per_bar\": 4,\n  \"audio\": \"../audio/line_test_120.wav\",\n  \"audio_reference_base\": \"chart_directory\",\n  \"total_frames\": {FRAMES},\n  \"music_start_frame\": {COUNT_IN},\n  \"seed\": {SEED},\n  \"timing\": \"Frames are zero-based decoded audio sample frames; section ends are exclusive. Each event is the required impact frame, not its spawn frame.\",\n  \"masks\": {{\"block\": 1, \"loop\": 2, \"wave\": 4, \"pit\": 8}},\n  \"sections\": [\n"
    );
    for (i, (name, start, end)) in sections.iter().enumerate() {
        writeln!(
            json,
            "    {{\"name\": \"{name}\", \"start_frame\": {start}, \"end_frame\": {end}}}{}",
            if i + 1 == sections.len() { "" } else { "," }
        )?;
    }
    json.push_str("  ],\n  \"events\": [\n");
    for (i, event) in events.iter().enumerate() {
        writeln!(
            json,
            "    {{\"frame\": {}, \"mask\": {}}}{}",
            event.frame,
            event.mask,
            if i + 1 == events.len() { "" } else { "," }
        )?;
    }
    json.push_str("  ]\n}\n");
    fs::write("assets/charts/line_test_120.json", json)?;
    println!(
        "Generated {FRAMES} mono frames at {RATE} Hz (70 seconds), {} events; peak {:.3}, RMS {rms:.3}.",
        events.len(),
        0.88
    );
    Ok(())
}
