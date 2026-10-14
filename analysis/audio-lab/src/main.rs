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
const MUSIC: &[u8] = include_bytes!("../../../assets/bgm.ogg");
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
struct Synth {
    frame: u64,
    rate: f64,
    step: u64,
    melody: Vec<f64>,
    bass: Vec<f64>,
    noise: u32,
    noise_value: f64,
}
impl Synth {
    fn new(rate: u32) -> Self {
        let score: serde_json::Value = serde_json::from_str(include_str!("../score.json")).unwrap();
        let frequencies = |key: &str| {
            score[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| {
                    let n = n.as_u64().unwrap();
                    if n == 0 {
                        0.
                    } else {
                        440. * 2f64.powf((n as f64 - 69.) / 12.)
                    }
                })
                .collect()
        };
        Self {
            frame: 0,
            rate: rate as f64,
            step: (rate as f64 * score["tick_seconds"].as_f64().unwrap()).round() as u64,
            melody: frequencies("melody"),
            bass: frequencies("bass"),
            noise: 0x12345678,
            noise_value: 0.,
        }
    }
    fn next(&mut self) -> [f32; 2] {
        let tick = self.frame / self.step;
        let local = (self.frame % self.step) as f64 / self.rate;
        let t = self.frame as f64 / self.rate;
        let hz = self.melody[tick as usize % self.melody.len()];
        let envelope = (local / 0.006).min(1.) * ((0.25 - local) / 0.06).clamp(0., 1.);
        let square = if hz == 0. {
            0.
        } else if (t * hz).fract() < 0.25 {
            1.
        } else {
            -1.
        };
        let bass = self.bass[(tick as usize / 8) % self.bass.len()];
        let triangle = 4. * ((t * bass).fract() - 0.5).abs() - 1.;
        // Fixed-rate sample-and-hold noise gives matching timbre across device rates.
        if (self.frame as f64 * 8000. / self.rate).floor()
            != ((self.frame.saturating_sub(1)) as f64 * 8000. / self.rate).floor()
        {
            self.noise ^= self.noise << 13;
            self.noise ^= self.noise >> 17;
            self.noise ^= self.noise << 5;
            self.noise_value = self.noise as f64 / u32::MAX as f64 * 2. - 1.;
        }
        let hat = self.noise_value * (-local * 80.).exp() * 0.07;
        let kick = if tick % 4 == 0 {
            (std::f64::consts::TAU * (55. * local + 35. * (1. - (-local * 30.).exp()) / 30.)).sin()
                * (-local * 22.).exp()
                * 0.18
        } else {
            0.
        };
        self.frame += 1;
        let out = (square * envelope * 0.11 + triangle * 0.09 + hat + kick) as f32;
        [out, out]
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "stream".into());
    if !["buffer", "stream", "synth", "idle", "render"].contains(&mode.as_str()) {
        return Err("Use buffer | stream | synth | idle | render".into());
    }
    if mode == "render" {
        let rate = 44100;
        let mut synth = Synth::new(rate);
        let mut bytes = Vec::new();
        for _ in 0..rate * 16 {
            let x = (synth.next()[0] * i16::MAX as f32) as i16;
            bytes.extend(x.to_le_bytes());
        }
        let mut wav = Vec::new();
        wav.extend(b"RIFF");
        wav.extend((36 + bytes.len() as u32).to_le_bytes());
        wav.extend(b"WAVEfmt ");
        wav.extend(16u32.to_le_bytes());
        wav.extend(1u16.to_le_bytes());
        wav.extend(1u16.to_le_bytes());
        wav.extend(rate.to_le_bytes());
        wav.extend((rate * 2).to_le_bytes());
        wav.extend(2u16.to_le_bytes());
        wav.extend(16u16.to_le_bytes());
        wav.extend(b"data");
        wav.extend((bytes.len() as u32).to_le_bytes());
        wav.extend(bytes);
        std::fs::write("synth-preview.wav", wav)?;
        return Ok(());
    }
    let seconds = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(30u64);
    let device = cpal::default_host()
        .default_output_device()
        .ok_or("No output device")?;
    let supported = device.default_output_config()?;
    let rate = supported.sample_rate();
    let channels = supported.channels() as usize;
    let ring = Arc::new(if mode == "stream" {
        Ring::new()
    } else {
        Ring {
            slots: Box::new([]),
            read: AtomicUsize::new(0),
            write: AtomicUsize::new(0),
        }
    });
    let stop = Arc::new(AtomicBool::new(false));
    let loops = Arc::new(AtomicUsize::new(0));
    let underruns = Arc::new(AtomicUsize::new(0));
    let failed = Arc::new(AtomicBool::new(false));
    let mut pcm = Vec::new();
    let mut producer = None;
    if mode == "buffer" {
        let mut reader = OggStreamReader::new(Cursor::new(MUSIC))?;
        let input_rate = reader.ident_hdr.audio_sample_rate;
        let input_channels = reader.ident_hdr.audio_channels as usize;
        let mut input = Vec::new();
        while let Some(packet) = reader.read_dec_packet_itl()? {
            for frame in packet.chunks_exact(input_channels) {
                input.push([
                    frame[0] as f32 / 32768.,
                    frame[input_channels - 1] as f32 / 32768.,
                ]);
            }
        }
        let frames = (input.len() as u64 * rate as u64 / input_rate as u64) as usize;
        pcm.reserve_exact(frames);
        for n in 0..frames {
            let position = n as f64 * input_rate as f64 / rate as f64;
            let i = position as usize;
            let fraction = (position - i as f64) as f32;
            let a = input[i];
            let b = input[(i + 1).min(input.len() - 1)];
            pcm.push(std::array::from_fn(|c| a[c] + (b[c] - a[c]) * fraction));
        }
    }
    if mode == "stream" {
        let mut decoder = Decoder::new(loops.clone())?;
        let ratio = decoder.reader.ident_hdr.audio_sample_rate as f64 / rate as f64;
        let worker_ring = ring.clone();
        let stop = stop.clone();
        let worker_failed = failed.clone();
        producer = Some(thread::spawn(move || {
            let result = (|| -> Result<(), String> {
                let mut a = decoder.next()?;
                let mut b = decoder.next()?;
                let mut phase = 0.;
                while !stop.load(Ordering::Relaxed) {
                    let pair = std::array::from_fn(|c| a[c] + (b[c] - a[c]) * phase as f32);
                    if !worker_ring.push(pair) {
                        thread::sleep(Duration::from_millis(1));
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
            if let Err(e) = result {
                eprintln!("Decoder failed: {e}");
                worker_failed.store(true, Ordering::Release);
            }
        }));
        let started = std::time::Instant::now();
        while ring.len() < 8192
            && started.elapsed() < Duration::from_secs(3)
            && !failed.load(Ordering::Acquire)
        {
            thread::sleep(Duration::from_millis(1));
        }
        if ring.len() < 8192 {
            return Err("Stream failed to prefill".into());
        }
    }
    let pcm_bytes = pcm.capacity() * std::mem::size_of::<[f32; 2]>();
    let mut at = 0;
    let mut synth = Synth::new(rate);
    let callback_ring = ring.clone();
    let callback_underruns = underruns.clone();
    let callback_loops = loops.clone();
    let callback_mode = mode.clone();
    let gain = std::env::var("AUDIO_LAB_GAIN")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.25)
        .clamp(0., 1.);
    let mut next = move || -> [f32; 2] {
        let pair = match callback_mode.as_str() {
            "stream" => callback_ring.pop().unwrap_or_else(|| {
                callback_underruns.fetch_add(1, Ordering::Relaxed);
                [0., 0.]
            }),
            "buffer" => {
                let pair = pcm[at];
                at += 1;
                if at == pcm.len() {
                    at = 0;
                    callback_loops.fetch_add(1, Ordering::Relaxed);
                }
                pair
            }
            "synth" => synth.next(),
            _ => [0., 0.],
        };
        pair.map(|x| x * gain)
    };
    let config = cpal::StreamConfig::from(supported.clone());
    let errors = |e| eprintln!("Output error: {e}");
    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => device.build_output_stream(
            config,
            move |data: &mut [f32], _| {
                for frame in data.chunks_mut(channels) {
                    let pair = next();
                    for (i, s) in frame.iter_mut().enumerate() {
                        *s = pair[i % 2];
                    }
                }
            },
            errors,
            None,
        )?,
        cpal::SampleFormat::I16 => device.build_output_stream(
            config,
            move |data: &mut [i16], _| {
                for frame in data.chunks_mut(channels) {
                    let pair = next();
                    for (i, s) in frame.iter_mut().enumerate() {
                        *s = (pair[i % 2].clamp(-1., 1.) * i16::MAX as f32) as i16;
                    }
                }
            },
            errors,
            None,
        )?,
        cpal::SampleFormat::U16 => device.build_output_stream(
            config,
            move |data: &mut [u16], _| {
                for frame in data.chunks_mut(channels) {
                    let pair = next();
                    for (i, s) in frame.iter_mut().enumerate() {
                        *s = ((pair[i % 2].clamp(-1., 1.) * 0.5 + 0.5) * u16::MAX as f32) as u16;
                    }
                }
            },
            errors,
            None,
        )?,
        other => return Err(format!("Unsupported output format {other:?}").into()),
    };
    println!(
        "{}",
        serde_json::json!({"mode":mode,"rate":rate,"channels":channels,"pcm_capacity_bytes":pcm_bytes,"ring_capacity_bytes":if mode=="stream"{CAPACITY*4}else{0},"gain":gain})
    );
    stream.play()?;
    thread::sleep(Duration::from_secs(seconds));
    drop(stream);
    stop.store(true, Ordering::Relaxed);
    if let Some(producer) = producer {
        producer.join().map_err(|_| "Producer panicked")?;
    }
    println!(
        "{}",
        serde_json::json!({"loops":loops.load(Ordering::Relaxed),"underrun_frames":underruns.load(Ordering::Relaxed),"decoder_failed":failed.load(Ordering::Relaxed)})
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ring_is_bounded_and_preserves_stereo() {
        let r = Ring::new();
        for n in 0..CAPACITY / 2 {
            assert!(r.push([n as f32, -(n as f32)]));
        }
        assert!(!r.push([0., 0.]));
        for n in 0..CAPACITY / 2 {
            assert_eq!(r.pop(), Some([n as f32, -(n as f32)]));
        }
        assert!(r.pop().is_none());
        for _ in 0..3 * CAPACITY {
            assert!(r.push([1., 2.]));
            assert_eq!(r.pop(), Some([1., 2.]));
        }
    }
    #[test]
    fn ring_cross_thread_publication_is_ordered() {
        let ring = Arc::new(Ring::new());
        let producer_ring = ring.clone();
        let producer = thread::spawn(move || {
            for n in 0..100_000 {
                while !producer_ring.push([n as f32, -(n as f32)]) {
                    thread::yield_now();
                }
            }
        });
        for n in 0..100_000 {
            let pair = loop {
                if let Some(pair) = ring.pop() {
                    break pair;
                }
                thread::yield_now();
            };
            assert_eq!(pair, [n as f32, -(n as f32)]);
        }
        producer.join().unwrap();
    }
    #[test]
    fn stream_loop_reuses_decoder_and_preserves_start() {
        let loops = Arc::new(AtomicUsize::new(0));
        let mut decoder = Decoder::new(loops.clone()).unwrap();
        let first: Vec<_> = (0..512).map(|_| decoder.next().unwrap()).collect();
        let mut next_loop = Vec::new();
        for _ in 0..600_000 {
            let pair = decoder.next().unwrap();
            if loops.load(Ordering::Relaxed) > 0 {
                next_loop.push(pair);
                break;
            }
        }
        assert_eq!(loops.load(Ordering::Relaxed), 1);
        for _ in 1..512 {
            next_loop.push(decoder.next().unwrap());
        }
        assert_eq!(first, next_loop);
    }
    #[test]
    fn synth_outputs_bounded_finite_samples() {
        let mut s = Synth::new(48000);
        let mut energy = 0.;
        for _ in 0..48000 * 8 {
            let pair = s.next();
            assert_eq!(pair[0], pair[1]);
            assert!(pair[0].is_finite() && pair[0].abs() < 1.);
            energy += pair[0].abs();
        }
        assert!(energy > 1000.);
    }
}
