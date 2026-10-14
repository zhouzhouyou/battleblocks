use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use lewton::inside_ogg::OggStreamReader;
use std::{
    io::Cursor,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
const MUSIC: &[u8] = include_bytes!("../../assets/bgm.ogg");
const CAPACITY: usize = 32768;
struct Ring {
    slots: Box<[AtomicU32]>,
    read: AtomicUsize,
    write: AtomicUsize,
}
impl Ring {
    fn new() -> Self {
        Self {
            slots: (0..CAPACITY).map(|_| AtomicU32::new(0)).collect(),
            read: AtomicUsize::new(0),
            write: AtomicUsize::new(0),
        }
    }
    fn push(&self, pair: [f32; 2]) -> bool {
        let w = self.write.load(Ordering::Relaxed);
        let r = self.read.load(Ordering::Acquire);
        if w.wrapping_sub(r) > CAPACITY - 2 {
            return false;
        }
        for (i, v) in pair.iter().enumerate() {
            self.slots[(w + i) % CAPACITY].store(v.to_bits(), Ordering::Relaxed);
        }
        self.write.store(w.wrapping_add(2), Ordering::Release);
        true
    }
    fn pop(&self) -> Option<[f32; 2]> {
        let r = self.read.load(Ordering::Relaxed);
        let w = self.write.load(Ordering::Acquire);
        if w.wrapping_sub(r) < 2 {
            return None;
        }
        let pair = std::array::from_fn(|i| {
            f32::from_bits(self.slots[(r + i) % CAPACITY].load(Ordering::Relaxed))
        });
        self.read.store(r.wrapping_add(2), Ordering::Release);
        Some(pair)
    }
    fn len(&self) -> usize {
        self.write
            .load(Ordering::Acquire)
            .wrapping_sub(self.read.load(Ordering::Acquire))
    }
}
struct Decoder {
    reader: OggStreamReader<Cursor<&'static [u8]>>,
    packet: Vec<i16>,
    at: usize,
    loops: Arc<AtomicUsize>,
    rewind_headers: usize,
}
impl Decoder {
    fn new(loops: Arc<AtomicUsize>) -> Result<Self, String> {
        Ok(Self {
            reader: OggStreamReader::new(Cursor::new(MUSIC)).map_err(|e| format!("{e:?}"))?,
            packet: Vec::new(),
            at: 0,
            loops,
            rewind_headers: 0,
        })
    }
    fn next(&mut self) -> Result<[f32; 2], String> {
        while self.at >= self.packet.len() {
            let decoded = match self.reader.read_dec_packet_itl() {
                // Lewton 0.9 seeks to the first Ogg page, which contains headers.
                // After rewind only, skip the three recognized Vorbis headers.
                Err(lewton::VorbisError::BadAudio(
                    lewton::audio::AudioReadError::AudioIsHeader,
                )) if self.rewind_headers > 0 => {
                    self.rewind_headers -= 1;
                    continue;
                }
                result => result.map_err(|e| format!("{e:?}"))?,
            };
            match decoded {
                Some(packet) => {
                    self.rewind_headers = 0;
                    self.packet = packet;
                    self.at = 0;
                }
                None => {
                    self.reader.seek_absgp_pg(0).map_err(|e| format!("{e:?}"))?;
                    self.rewind_headers = 3;
                    self.loops.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        let channels = self.reader.ident_hdr.audio_channels as usize;
        let left = self.packet[self.at] as f32 / 32768.;
        let right = self.packet[self.at + channels - 1] as f32 / 32768.;
        self.at += channels;
        Ok([left, right])
    }
}

struct Control {
    stop: AtomicBool,
    playing: AtomicBool,
    muted: AtomicBool,
    crush: AtomicBool,
    underruns: AtomicUsize,
}
pub struct Audio {
    stream: Option<cpal::Stream>,
    worker: Option<thread::JoinHandle<()>>,
    control: Arc<Control>,
}
impl Audio {
    pub fn new(music: bool, effects: bool) -> Result<Self, String> {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or("No audio output device")?;
        let supported = device.default_output_config().map_err(|e| e.to_string())?;
        let rate = supported.sample_rate();
        let channels = supported.channels() as usize;
        let control = Arc::new(Control {
            stop: AtomicBool::new(false),
            playing: AtomicBool::new(false),
            muted: AtomicBool::new(false),
            crush: AtomicBool::new(false),
            underruns: AtomicUsize::new(0),
        });
        let ring = Arc::new(Ring::new());
        // Original effect is a mono, 44.1 kHz, 16-bit PCM WAV. Decode once, outside the callback.
        let effect = if effects {
            decode_effect(include_bytes!("../../assets/crush.wav"), rate)?
        } else {
            Vec::new()
        };
        let mut worker = None;
        if music {
            let mut decoder = Decoder::new(Arc::new(AtomicUsize::new(0)))?;
            let ratio = decoder.reader.ident_hdr.audio_sample_rate as f64 / rate as f64;
            let queue = ring.clone();
            let state = control.clone();
            worker = Some(thread::spawn(move || {
                let result = (|| -> Result<(), String> {
                    let mut a = decoder.next()?;
                    let mut b = decoder.next()?;
                    let mut phase = 0.;
                    while !state.stop.load(Ordering::Relaxed) {
                        if !queue.push(std::array::from_fn(|c| a[c] + (b[c] - a[c]) * phase as f32))
                        {
                            thread::sleep(Duration::from_millis(2));
                            continue;
                        }
                        phase += ratio;
                        while phase >= 1. {
                            a = b;
                            b = decoder.next()?;
                            phase -= 1.;
                        }
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    eprintln!("Music decoder: {error}");
                }
            }));
            let started = std::time::Instant::now();
            while ring.len() < 8192 && started.elapsed() < Duration::from_secs(2) {
                thread::sleep(Duration::from_millis(1));
            }
        }
        let state = control.clone();
        let mut voices = [usize::MAX; 8];
        let mut next = move || {
            if state.crush.swap(false, Ordering::Relaxed) && !effect.is_empty() {
                let voice = voices.iter().position(|&p| p == usize::MAX).unwrap_or(0);
                voices[voice] = 0;
            }
            let mut pair = [0., 0.];
            if music && state.playing.load(Ordering::Relaxed) {
                pair = ring
                    .pop()
                    .unwrap_or_else(|| {
                        state.underruns.fetch_add(1, Ordering::Relaxed);
                        [0., 0.]
                    })
                    .map(|v| v * 0.2);
            }
            for position in &mut voices {
                if *position != usize::MAX {
                    let sample = effect[*position] * 0.35;
                    pair[0] += sample;
                    pair[1] += sample;
                    *position += 1;
                    if *position == effect.len() {
                        *position = usize::MAX;
                    }
                }
            }
            if state.muted.load(Ordering::Relaxed) {
                [0., 0.]
            } else {
                pair.map(|v| v.clamp(-1., 1.))
            }
        };
        let config = cpal::StreamConfig::from(supported.clone());
        let errors = |e| eprintln!("Audio output: {e}");
        let built = match supported.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                config,
                move |out: &mut [f32], _| {
                    for frame in out.chunks_mut(channels) {
                        let p = next();
                        for (i, v) in frame.iter_mut().enumerate() {
                            *v = p[i % 2];
                        }
                    }
                },
                errors,
                None,
            ),
            cpal::SampleFormat::I16 => device.build_output_stream(
                config,
                move |out: &mut [i16], _| {
                    for frame in out.chunks_mut(channels) {
                        let p = next();
                        for (i, v) in frame.iter_mut().enumerate() {
                            *v = (p[i % 2] * i16::MAX as f32) as i16;
                        }
                    }
                },
                errors,
                None,
            ),
            cpal::SampleFormat::U16 => device.build_output_stream(
                config,
                move |out: &mut [u16], _| {
                    for frame in out.chunks_mut(channels) {
                        let p = next();
                        for (i, v) in frame.iter_mut().enumerate() {
                            *v = ((p[i % 2] * 0.5 + 0.5) * u16::MAX as f32) as u16;
                        }
                    }
                },
                errors,
                None,
            ),
            other => {
                control.stop.store(true, Ordering::Relaxed);
                if let Some(w) = worker {
                    let _ = w.join();
                }
                return Err(format!("Unsupported audio format: {other:?}"));
            }
        };
        let mut audio = Self {
            stream: None,
            worker,
            control,
        };
        let stream = built.map_err(|e| e.to_string())?;
        stream.play().map_err(|e| e.to_string())?;
        audio.stream = Some(stream);
        Ok(audio)
    }
    pub fn playing(&self, playing: bool) {
        self.control.playing.store(playing, Ordering::Relaxed);
    }
    pub fn muted(&self, muted: bool) {
        self.control.muted.store(muted, Ordering::Relaxed);
    }
    pub fn crush(&self) {
        self.control.crush.store(true, Ordering::Relaxed);
    }
}
impl Drop for Audio {
    fn drop(&mut self) {
        self.stream.take();
        self.control.stop.store(true, Ordering::Relaxed);
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
        eprintln!(
            "Audio PCM underrun frames: {}",
            self.control.underruns.load(Ordering::Relaxed)
        );
    }
}
fn decode_effect(bytes: &[u8], rate: u32) -> Result<Vec<f32>, String> {
    if bytes.get(..4) != Some(b"RIFF") || bytes.get(8..12) != Some(b"WAVE") {
        return Err("Invalid effect WAV".into());
    }
    let mut at = 12;
    let mut format = None;
    let mut pcm = None;
    while at + 8 <= bytes.len() {
        let n = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap()) as usize;
        let end = at + 8 + n;
        if end > bytes.len() {
            return Err("Truncated WAV".into());
        }
        let chunk = &bytes[at + 8..end];
        match &bytes[at..at + 4] {
            b"fmt " if n >= 16 => {
                format = Some((
                    u16::from_le_bytes(chunk[..2].try_into().unwrap()),
                    u16::from_le_bytes(chunk[2..4].try_into().unwrap()),
                    u32::from_le_bytes(chunk[4..8].try_into().unwrap()),
                    u16::from_le_bytes(chunk[14..16].try_into().unwrap()),
                ))
            }
            b"data" => pcm = Some(chunk),
            _ => {}
        }
        at = end + n % 2;
    }
    let (encoding, channels, source_rate, bits) = format.ok_or("Missing WAV format")?;
    if encoding != 1 || channels != 1 || bits != 16 || source_rate == 0 {
        return Err("Unsupported effect WAV format".into());
    }
    let input: Vec<f32> = pcm
        .ok_or("Missing WAV samples")?
        .chunks_exact(2)
        .map(|p| i16::from_le_bytes([p[0], p[1]]) as f32 / 32768.)
        .collect();
    if input.is_empty() {
        return Err("Empty effect".into());
    }
    let count = (input.len() as u64 * rate as u64 / source_rate as u64) as usize;
    Ok((0..count)
        .map(|n| {
            let pos = n as f64 * source_rate as f64 / rate as f64;
            let i = pos as usize;
            let f = (pos - i as f64) as f32;
            input[i] + (input[(i + 1).min(input.len() - 1)] - input[i]) * f
        })
        .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn effect_is_finite_and_has_expected_duration() {
        let data = decode_effect(include_bytes!("../../assets/crush.wav"), 48000).unwrap();
        assert!((data.len() as f64 / 48000. - 1.708).abs() < 0.002);
        assert!(data.iter().all(|v| v.is_finite() && v.abs() <= 1.));
    }
    #[test]
    fn ring_capacity_and_stereo_order_are_bounded() {
        let ring = Ring::new();
        for n in 0..CAPACITY / 2 {
            assert!(ring.push([n as f32, -(n as f32)]));
        }
        assert!(!ring.push([0., 0.]));
        for n in 0..CAPACITY / 2 {
            assert_eq!(ring.pop(), Some([n as f32, -(n as f32)]));
        }
        assert!(ring.pop().is_none());
    }
    #[test]
    fn original_music_rewinds_without_changing_opening_samples() {
        let loops = Arc::new(AtomicUsize::new(0));
        let mut decoder = Decoder::new(loops.clone()).unwrap();
        let first: Vec<_> = (0..512).map(|_| decoder.next().unwrap()).collect();
        let mut second = Vec::new();
        for _ in 0..600_000 {
            let sample = decoder.next().unwrap();
            if loops.load(Ordering::Relaxed) > 0 {
                second.push(sample);
                break;
            }
        }
        assert_eq!(loops.load(Ordering::Relaxed), 1);
        for _ in 1..512 {
            second.push(decoder.next().unwrap());
        }
        assert_eq!(first, second);
    }
    #[test]
    #[ignore = "Requires a real audio output device"]
    fn native_output_stream_effect_pause_resume() {
        let audio = Audio::new(true, true).unwrap();
        audio.muted(true);
        audio.playing(true);
        for _ in 0..8 {
            audio.crush();
            thread::sleep(Duration::from_millis(250));
        }
        thread::sleep(Duration::from_secs(12));
        audio.playing(false);
        thread::sleep(Duration::from_millis(200));
        audio.playing(true);
        thread::sleep(Duration::from_secs(2));
        assert_eq!(audio.control.underruns.load(Ordering::Relaxed), 0);
    }
    #[test]
    fn malformed_effect_is_rejected() {
        assert!(decode_effect(b"wrong", 44100).is_err());
    }
}
