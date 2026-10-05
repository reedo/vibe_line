# Vibe String

An open-source, Rust rhythm game with a wool-string theme, inspired by Vib-Ribbon. This
repository contains a Rust workspace, a minimal Bevy window, an MVP roadmap and
a reproducible development music/chart fixture. Gameplay is not implemented yet.

## Development workspace

| Package | Location | Purpose |
| --- | --- | --- |
| `vibe-string` | `crates/vibe-string` | Bevy application; the default package for `cargo run` |
| `vibe-string-core` | `crates/vibe-string-core` | Engine-independent library reserved for chart and judgment rules |
| `vibe-string-tools` | `tools` | Development tools, currently `generate-dev-track` |

Install stable Rust with rustup. The workspace uses Rust 2024 and requires Rust
1.95 or newer; `rust-toolchain.toml` selects stable with rustfmt and Clippy. On
Windows, install the Visual Studio C++ Build Tools and Windows SDK as described
in [Bevy's setup instructions](https://bevy.org/learn/quick-start/getting-started/setup/).
The first build downloads and compiles the engine dependencies.

Run these commands from the repository root:

```sh
cargo run --locked
cargo check --locked --workspace --all-targets
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

`cargo run` currently opens an empty 1280x720 window titled **Vibe String** with
a 2D camera. The shared `assets/` folder is resolved through `BEVY_ASSET_ROOT` in
`.cargo/config.toml` when running through Cargo, including from a member directory.
A future standalone distribution will need to ship its assets alongside the
executable. Bevy's 2D, audio and WAV support are enabled; its 3D feature set is
disabled. Dependency versions are recorded in `Cargo.lock`.

Regenerate the development WAV and chart from the repository root:

```sh
cargo run --locked --release -p vibe-string-tools --bin generate-dev-track
```

This replaces the two generated fixture files. The core library starts empty;
add gameplay tests as chart and judgment behavior is implemented. All build
artifacts go in the ignored `target/` directory.

## What we are trying to preserve

The terrain is the music chart: one character travels along a continuous strand
of wool, responding to four obstacle shapes and their two-action
combinations. Successful inputs produce expressive traversal animations;
mistakes visibly degrade the character, with a route to recovery. Players can
eventually generate courses from their own local music.

## Wool-string direction

The project name is **Vibe String**. The proposed character is a small walking
knot with dangling thread legs. Success tightens it into a tidy, lively shape;
mistakes loosen and unravel it; recovery gathers the loose strands again.

Colour is part of each level's identity. Both the wool and background can use
light, dark or colourful palettes, and can change within a track to reflect its
intensity or mood. Block, loop, wave and pit obstacles are bends and coils of the
same continuous strand. Keep their silhouettes distinct and preserve a clearly
readable point of contact throughout palette changes.

For the MVP, give each level a base palette and optional timed palette cues for
musical sections. Blend between palettes using song time so pause and restart
remain consistent. Author mood changes explicitly; smoothed musical energy can
drive intensity variations without claiming to infer emotion from loudness.
Keep palette cues separate from required-input events. Use contrasting outlines
or backing where needed during transitions, and retain shape-based action cues
so colour changes do not change the meaning of an obstacle.

For the MVP, make wool from simple thick strokes, a few offset fibres and modest
animated wobble. Tie movement to the rhythm, with slight squash/stretch when an
obstacle is passed. Add richer fibre shading, cloth backgrounds and decorative
stitches after input timing and shape readability are established. Decorative
fibres and wobble must not move the logical hit point or hide a combination.

Use independent code, an original character, an original title and new music.
The premise that Vib-Ribbon has no identifiable owner is not supported by the
available evidence: [MoMA's collection record](https://www.moma.org/collection/works/162460)
credits Sony Computer Entertainment's copyright and trademark, and
[Sony announced its rerelease in 2014](https://blog.playstation.com/2014/10/06/vib-ribbon-finally-releases-in-north-america-tomorrow/).
These sources are not a complete audit of present contractual rights. The
project's MIT license does not grant rights to the original game's assets.

## Two useful finish lines

**Gameplay MVP:** one complete, enjoyable track; all four actions and six pairs;
keyboard and controller support; calibration; an original animated knot
character; level palettes and section colour transitions; score, combo, failure
and recovery; tutorial, results and retry.

**Faithful product MVP:** everything above plus local WAV import and deterministic
automatic chart generation. Restricting the first importer to WAV keeps decoder
and timing issues manageable. Wider format support can follow.

Do not describe the single-song build as reproducing the original's complete
play-your-own-music feature.

## Build order

Estimates below are rough focused development days for one experienced developer,
not commitments. Audio/device behavior and chart quality are the main unknowns.

| Stage | Work | Exit criterion | Estimate |
| --- | --- | --- | --- |
| 1. Prove timing | Windows Bevy app, fixture playback, beat marker, input log, latency offset, pause/restart | Marker stays aligned over a full track; offset is adjustable; no stale input or audio survives restart | 1-2 days |
| 2. Make one obstacle fun, then four | Continuous strand, one avatar, approach preview, timestamp judgment, four actions | All four shapes are readable; early/late/wrong/missed inputs resolve once; held keys do not clear future notes | 2-4 days |
| 3. Complete the gameplay MVP | Six pairs, animation, level palettes and section transitions, degradation/recovery, score, tutorial, remapping, controller, results/retry | A new player can learn, complete or fail the fixture, and restart without developer help; obstacles stay readable through colour changes | 3-5 days |
| 4. Play local music | WAV decode, analysis, chart rules, caching, local file selection | Same file/settings/version produce the same playable chart; silence and dense audio behave sensibly | 4-7 days |
| 5. Package the product MVP | Regression checks, device checks, release build, asset notices and build instructions | Windows release runs on another machine; development and imported tracks work through the complete loop | 2-4 days |

This suggests roughly 1-2 focused weeks for the single-song game and 3-5 weeks
for the constrained music-import MVP. Validate stage 1 before committing to dates.

During the timing stage, keep a short reference log from original-game footage
or an available copy: obstacle geometry, input mapping, traversal animations,
pair behavior and success/failure feedback. Separate observed behavior from our
new tuning; exact historical scoring and timing remain unverified.

## Technical decisions

- Start with Rust and **Bevy 0.19.1**, native Windows first. Pin the initial engine
  release and commit Cargo.lock when the application is created. Current
  [Bevy documentation](https://bevy.org/learn/quick-start/getting-started/) uses the
  0.19 series.
- Use Bevy's built-in audio for the timing experiment. Its
  [AudioSinkPlayback API](https://docs.rs/bevy/0.19.1/bevy/audio/trait.AudioSinkPlayback.html)
  exposes position, pause and resume. Load the whole short fixture before play.
- Keep chart data, timestamped input and judgment in `vibe-string-core`, with no
  Bevy dependency. The `vibe-string` application owns audio, rendering, device
  input and menus; `vibe-string-tools` contains development asset generation.
- Store chart times as integer source-audio frame indices with an explicit
  sample rate. Convert to the playback timeline at the boundary; if audio is
  resampled, preserve the same time rather than reinterpreting frame numbers.
- Represent required actions with a four-bit mask: block=1, loop=2, wave=4,
  pit=8. A pair is one event whose mask has two bits set. Stable note IDs allow
  one judgment per event.
- Begin rendering with Bevy line gizmos/polylines and a simple original knot
  character. Layer thin fibres over the main stroke to suggest wool. Replace
  the stroke renderer only if thickness or joins require it.
  Rhythm judgments drive obstacle traversal; a physics engine is unnecessary.

Keep an audio transport interface so another backend can be substituted if
measurements justify it. The maintained
[bevy_kira_audio compatibility table](https://github.com/NiklasEi/bevy_kira_audio)
maps Bevy 0.19 to plugin 0.26. Kira provides scheduled clocks, but adopting it
does not eliminate speaker/device latency.

### Timing contract

The authoritative song time comes from audio playback, not accumulated render
frame deltas. At normal playback speed, place each approaching event using:

```text
distance_to_hit = scroll_speed * (event_time - visual_song_time)
judgment_error = calibrated_input_song_time - event_time
```

Read backend position and use a bounded, monotonic estimate between observations
if needed to make movement smooth. Explicitly reset that estimate on pause,
seek, focus loss and restart. Keep playback speed at 1.0 for the MVP.
Clear pending chords and queued gameplay inputs on those transitions, and
require a fresh press after release when resuming. Decide explicitly whether
resume uses a short countdown or restarts the phrase.

Backend position is not a measurement of the sound at the listener's ears.
Measure output latency and frame scheduling effects, provide a user timing
offset, and keep visual alignment separately adjustable if playtesting needs it.
Document offset signs so calibration is understandable.

Collect key/button press edges, ignore keyboard repeats, and timestamp as early
as the input integration allows. Bevy's
[KeyboardInput message](https://docs.rs/bevy/0.19.1/bevy/input/keyboard/struct.KeyboardInput.html)
has no operating-system event timestamp: reading it once per frame introduces
timing uncertainty. Test real input at 60/120/144 Hz and under frame stalls.

Suggested initial tuning: 2 seconds of approach visibility, a +/-100 ms hit
window and up to 40 ms between the presses of a two-button chord. These are our
starting values, not verified original-game rules. Require each chord press
inside the hit window, allow only the required mask, consume an input once, and
give explicit feedback for wrong or extra presses. Merge input events and
note/chord deadlines into chronological order; at equal timestamps, process
eligible presses before expiry. This keeps misses, combo changes and recovery
in the same order when several events arrive during a slow frame.

### Automatic chart generation

Do this after the authored chart feels good. Analyse a track before gameplay:

1. Decode to a known PCM timeline; retain exact duration/sample-rate metadata.
2. Measure an onset envelope (for example spectral flux), local energy and
   broad frequency balance. Preserve quiet sections.
3. Pick salient peaks using adaptive thresholds and minimum spacing. Tempo
   estimation may help grouping, but never force every song onto a fixed grid.
4. Convert events into musical phrases using constrained patterns for the four
   actions. Add pairs only where difficulty and spacing allow them. Include
   repetition, rests, a lead-in and sensible transitions between patterns.
5. Cache the chart by audio-content hash, generator version, difficulty and seed.

Use density limits per time window as well as a minimum event gap; a fast song
must not automatically create an impossible chart. Validate silent, sustained,
percussive and tempo-changing inputs. Check the importer against the known
fixture, then manually assess several different styles of music. The fixture
alone cannot establish automatic chart quality. This is a new generator, not a
claim to reproduce Sony's original analysis algorithm.

## Development track

The fixture is an AI-composed, programmatically synthesized electronic track.
It is generated by ordinary Rust code, not a neural audio-generation service.
It uses oscillators and seeded noise rather than imported samples. Exact musical
event times are available by construction, which makes it useful for initial
synchronization and judgment work.

- Audio: [assets/audio/line_test_120.wav](assets/audio/line_test_120.wav)
- Reference chart: [assets/charts/line_test_120.json](assets/charts/line_test_120.json)
- Generator: [tools/generate_dev_track.rs](tools/generate_dev_track.rs)
- Generation instructions and provenance:
  [assets/audio/PROVENANCE.txt](assets/audio/PROVENANCE.txt)

This chart is authored alongside the music, not produced by the future music
analyser. Use its exact timestamps as the known reference for early development.

## Verification and scope boundaries

Test the pure judgment module with timestamped replays, including hit-window
boundaries, every pair, incorrect masks, repeated/held keys, missed events and
restart. The same input timeline must produce the same results under different
update schedules. Separately test real input/audio: replay tests cannot prove
audible synchronization.

Check audio startup, pause/resume, focus changes and repeated retry on actual
hardware. Test calibration and controller input independently of the keyboard.
Imported tracks remain local; they do not need uploading or bundling with the
open-source repository.

Leave browser builds, online features, a full chart editor, exact original
scoring, original character/song recreation, procedural singing, complex camera
choreography and broad codec support until after the MVP. Track any unverified
historical behavior in observation notes rather than labeling proposed tuning
as original fidelity.

The next implementation task is to finish stage 1: play the fixture in the Bevy window,
draw a beat marker, log presses against the audio timeline and calibrate the
offset before building the full visual game.
