//! Optional system synthesis on a worker; generated PCM never enters the repository.
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
#[derive(Default)]
pub struct Output {
    pub generation: AtomicUsize,
    pub ready: AtomicBool,
    pub samples: Mutex<Option<(usize, Vec<f32>)>>,
    shutdown: AtomicBool,
}
struct Request {
    text: String,
    zh: bool,
    generation: usize,
}
pub struct Speech {
    pub output: Arc<Output>,
    request: Arc<Mutex<Option<Request>>>,
    worker: Option<std::thread::JoinHandle<()>>,
    rate: u32,
}
impl Speech {
    pub fn new(rate: u32) -> Self {
        let output = Arc::new(Output::default());
        let request = Arc::new(Mutex::new(None));
        Self {
            output,
            request,
            worker: None,
            rate,
        }
    }
    pub fn stop(&mut self) {
        self.output.generation.fetch_add(1, Ordering::Relaxed);
        *self.request.lock().unwrap() = None;
    }
    pub fn speak(&mut self, text: &str, zh: bool) {
        #[cfg(target_os = "macos")]
        if self.worker.is_none() {
            let state = self.output.clone();
            let queue = self.request.clone();
            let rate = self.rate;
            self.worker = Some(std::thread::spawn(move || run(rate, state, queue)));
        }
        #[cfg(not(target_os = "macos"))]
        let _ = self.rate;
        if self.worker.is_none() || text.is_empty() {
            return;
        }
        self.stop();
        let generation = self.output.generation.load(Ordering::Relaxed);
        *self.request.lock().unwrap() = Some(Request {
            text: text.to_owned(),
            zh,
            generation,
        });
    }
}
impl Drop for Speech {
    fn drop(&mut self) {
        self.stop();
        self.output.shutdown.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
#[cfg(target_os = "macos")]
fn run(rate: u32, state: Arc<Output>, queue: Arc<Mutex<Option<Request>>>) {
    use std::{
        io::Write,
        process::{Command, Stdio},
        time::{Duration, Instant},
    };
    let listing = Command::new("/usr/bin/say")
        .args(["-v", "?"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    let voices = [("en_US", "Samantha"), ("zh_CN", "Tingting")].map(|(language, preferred)| {
        let candidates: Vec<_> = listing
            .lines()
            .filter_map(|line| {
                let fields: Vec<_> = line.split_whitespace().collect();
                let at = fields.iter().position(|&field| field == language)?;
                Some(fields[..at].join(" "))
            })
            .collect();
        candidates
            .iter()
            .find(|name| name.starts_with(preferred))
            .or(candidates.first())
            .cloned()
    });
    // A private temporary directory keeps synthesized dialogue isolated from other processes.
    let directory = std::env::temp_dir().join(format!(
        "battleblocks-voice-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    use std::os::unix::fs::DirBuilderExt;
    if std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&directory)
        .is_err()
    {
        return;
    }
    let path = directory.join("speech.wav");
    let mut cache: std::collections::VecDeque<((String, bool), Vec<f32>)> = Default::default();
    // Eight MiB of mono PCM at most, independent of the output device's rate.
    const CACHE_SAMPLES: usize = 2 * 1024 * 1024;
    while !state.shutdown.load(Ordering::Relaxed) {
        let request = queue.lock().unwrap().take();
        let Some(request) = request else {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        };
        let cancelled = || {
            state.shutdown.load(Ordering::Relaxed)
                || state.generation.load(Ordering::Relaxed) != request.generation
        };
        let result = (|| -> Option<Vec<f32>> {
            if cancelled() {
                return None;
            }
            if let Some((_, samples)) = cache
                .iter()
                .find(|((text, zh), _)| *text == request.text && *zh == request.zh)
            {
                return Some(samples.clone());
            }
            let voice = voices[request.zh as usize].as_ref()?;
            let mut child = Command::new("/usr/bin/say")
                .args(["-v", voice, "-r", "165", "-o"])
                .arg(&path)
                .args(["--file-format=WAVE", "--data-format=LEI16@22050"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .ok()?;
            if let Some(mut input) = child.stdin.take() {
                let _ = input.write_all(request.text.as_bytes());
            }
            let start = Instant::now();
            loop {
                if cancelled() || start.elapsed() > Duration::from_secs(15) {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                match child.try_wait() {
                    Ok(Some(status)) if status.success() => break,
                    Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                    _ => {
                        let _ = child.kill();
                        let _ = child.wait();
                        return None;
                    }
                }
            }
            let bytes = std::fs::read(&path).ok()?;
            let pcm = super::native::decode_effect(&bytes, rate).ok()?;
            let samples = super::voice_filter::mechanical(&pcm, rate);
            if samples.len() <= CACHE_SAMPLES && !cancelled() {
                while cache.iter().map(|(_, v)| v.len()).sum::<usize>() + samples.len()
                    > CACHE_SAMPLES
                {
                    cache.pop_front();
                }
                cache.push_back(((request.text.clone(), request.zh), samples.clone()));
            }
            Some(samples)
        })();
        let _ = std::fs::remove_file(&path);
        if !cancelled() {
            if let Some(samples) = result {
                *state.samples.lock().unwrap() = Some((request.generation, samples));
                state.ready.store(true, Ordering::Release);
            }
        }
    }
    let _ = std::fs::remove_dir_all(directory);
}
#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Requires installed macOS system voices"]
    fn system_voice_worker_produces_pcm_and_cancels() {
        let mut speech = Speech::new(48000);
        let text = crate::content::role_lore(12)
            .unwrap()
            .voice("victory", false)
            .unwrap();
        speech.speak(text, false);
        let start = std::time::Instant::now();
        while !speech.output.ready.load(Ordering::Acquire) {
            assert!(
                start.elapsed().as_secs() < 20,
                "system voice did not produce PCM"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let (generation, samples) = speech.output.samples.lock().unwrap().take().unwrap();
        assert!(samples.len() > 48000);
        assert!(samples.iter().all(|v| v.is_finite() && v.abs() <= 0.65));
        assert!(samples.iter().any(|v| v.abs() > 0.01));
        speech.stop();
        assert_ne!(speech.output.generation.load(Ordering::Relaxed), generation);
        speech.output.ready.store(false, Ordering::Release);
        speech.speak(text, false);
        let cached_start = std::time::Instant::now();
        while !speech.output.ready.load(Ordering::Acquire) {
            assert!(
                cached_start.elapsed().as_secs() < 5,
                "cached cue did not arrive"
            );
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        let (_, cached) = speech.output.samples.lock().unwrap().take().unwrap();
        assert_eq!(samples, cached);
        println!(
            "Cached voice ready in {:.2} ms",
            cached_start.elapsed().as_secs_f64() * 1000.
        );
        speech.stop();
        speech.speak(text, true);
        speech.stop(); // cancellation is also safe during synthesis
    }
}
