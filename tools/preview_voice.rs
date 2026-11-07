//! Shared runtime filter, driven by preview_voice.py; raw mono i16 input/output.
#[path = "../src/audio/voice_filter.rs"]
mod voice_filter;
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let rate: u32 = args[1].parse().expect("sample rate");
    let bytes = std::fs::read(&args[2]).expect("input PCM");
    let pcm: Vec<_> = bytes.chunks_exact(2)
        .map(|p| i16::from_le_bytes([p[0], p[1]]) as f32 / 32768.).collect();
    let start = std::time::Instant::now();
    let output = voice_filter::mechanical(&pcm, rate);
    println!("Filter: {:.2} ms for {:.2} s of speech", start.elapsed().as_secs_f64() * 1000., pcm.len() as f64 / rate as f64);
    let bytes: Vec<_> = output.iter().flat_map(|v| ((v.clamp(-1., 1.) * 32767.) as i16).to_le_bytes()).collect();
    std::fs::write(&args[3], bytes).expect("output PCM");
}
