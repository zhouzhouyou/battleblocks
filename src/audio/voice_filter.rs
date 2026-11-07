/// Steady female pitch, double tracking, metallic modulation and a speaker band.
/// This suggests a mechanical female AI; it does not recreate the actor or game's DSP.
pub fn mechanical(input: &[f32], rate: u32) -> Vec<f32> {
    let flattened = flatten_pitch(input, rate);
    let mut low = 0.;
    let mut previous = 0.;
    let mut high = 0.;
    let high_a = (-std::f32::consts::TAU * 150. / rate as f32).exp();
    let low_a = 1. - (-std::f32::consts::TAU * 4200. / rate as f32).exp();
    let delay = (rate as f32 * 0.018) as usize;
    flattened
        .iter()
        .enumerate()
        .map(|(n, &dry)| {
            let doubled = dry + flattened.get(n.wrapping_sub(delay)).copied().unwrap_or(0.) * 0.18;
            let carrier = (std::f32::consts::TAU * 90. * n as f32 / rate as f32).sin();
            let sample = doubled * (0.72 + 0.28 * carrier);
            high = high_a * (high + sample - previous);
            previous = sample;
            low += low_a * (high - low);
            (low * 1.7).tanh() * 0.65
        })
        .collect()
}

// Pitch-synchronous overlap/add changes the spacing of voiced pulses while
// retaining each pulse's formants. Unvoiced consonants and pauses pass through.
const VOICE_PITCH: f32 = 196.;

fn pitch_period(samples: &[f32], rate: u32) -> Option<usize> {
    let stride = (rate / 8000).max(1) as usize;
    let window: Vec<_> = samples.iter().step_by(stride).copied().collect();
    let analysis_rate = rate as f32 / stride as f32;
    let first_lag = (analysis_rate / 330.) as usize;
    let last_lag = (analysis_rate / 130.) as usize;
    if window.len() <= last_lag * 2 || window.iter().map(|v| v * v).sum::<f32>() < 0.005 {
        return None;
    }
    let correlations: Vec<_> = (first_lag..=last_lag)
        .map(|lag| {
            let mut cross = 0.;
            let mut left_energy = 0.;
            let mut right_energy = 0.;
            for (&left, &right) in window.iter().zip(window.iter().skip(lag)) {
                cross += left * right;
                left_energy += left * left;
                right_energy += right * right;
            }
            cross / (left_energy * right_energy).sqrt().max(1e-9)
        })
        .collect();
    let best = correlations.iter().copied().fold(0., f32::max);
    if best < 0.7 {
        return None;
    }
    // Prefer the earliest strong peak to avoid octave errors from longer lags.
    correlations
        .windows(3)
        .position(|p| p[1] >= best * 0.93 && p[1] > p[0] && p[1] >= p[2])
        .map(|index| (first_lag + index + 1) * stride)
}

fn flatten_pitch(input: &[f32], rate: u32) -> Vec<f32> {
    let hop = (rate / 100).max(1) as usize;
    let window = (rate as f32 * 0.035) as usize;
    let periods: Vec<_> = (0..input.len())
        .step_by(hop)
        .map(|at| pitch_period(&input[at..(at + window).min(input.len())], rate))
        .collect();
    let mut output = input.to_vec();
    let mut pulses = vec![0.; input.len()];
    let mut weights = vec![0.; input.len()];
    let target_period = rate as f32 / VOICE_PITCH;
    let mut frame = 0;
    while frame < periods.len() {
        if periods[frame].is_none() {
            frame += 1;
            continue;
        }
        let start_frame = frame;
        while frame < periods.len() && periods[frame].is_some() {
            frame += 1;
        }
        let start = start_frame * hop;
        let end = (frame * hop).min(input.len());
        if end - start < hop * 6 {
            continue;
        }
        let period = periods[start_frame].unwrap();
        let mut mark = (start..(start + period).min(end))
            .max_by(|&a, &b| input[a].total_cmp(&input[b]))
            .unwrap();
        let mut marks = Vec::new();
        while mark < end {
            let period = periods[(mark / hop).min(frame - 1)].unwrap();
            marks.push((mark, period));
            let predicted = mark + period;
            let left = predicted.saturating_sub(period / 5);
            let right = (predicted + period / 5 + 1).min(end);
            if left >= right {
                break;
            }
            mark = (left..right)
                .max_by(|&a, &b| input[a].total_cmp(&input[b]))
                .unwrap();
        }
        let mut source = 0;
        let mut target = marks[0].0 as f32;
        while target < end as f32 {
            while source + 1 < marks.len()
                && (marks[source + 1].0 as f32 - target).abs()
                    < (marks[source].0 as f32 - target).abs()
            {
                source += 1;
            }
            let (center, period) = marks[source];
            for offset in -(period as isize)..=period as isize {
                let from = center as isize + offset;
                let to = target.round() as isize + offset;
                if from < 0
                    || from >= input.len() as isize
                    || to < start as isize
                    || to >= end as isize
                {
                    continue;
                }
                let weight =
                    0.5 + 0.5 * (std::f32::consts::PI * offset as f32 / period as f32).cos();
                pulses[to as usize] += input[from as usize] * weight;
                weights[to as usize] += weight;
            }
            target += target_period;
        }
        for at in start..end {
            if weights[at] > 0.01 {
                let fade = ((at - start).min(end - at) as f32 / (hop * 2) as f32).min(1.);
                let amount = fade * 0.85;
                output[at] = input[at] * (1. - amount) + pulses[at] / weights[at] * amount;
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn voiced_pitch_converges_without_changing_duration() {
        let rate = 22050;
        for frequency in [155., 245.] {
            let input: Vec<_> = (0..rate)
                .map(|n| (std::f32::consts::TAU * frequency * n as f32 / rate as f32).sin() * 0.4)
                .collect();
            let output = flatten_pitch(&input, rate);
            assert_eq!(output.len(), input.len());
            let center = rate as usize / 2;
            let period = pitch_period(&output[center..center + 1000], rate).unwrap();
            let actual = rate as f32 / period as f32;
            assert!(
                (actual - VOICE_PITCH).abs() < 12.,
                "{frequency} Hz became {actual} Hz"
            );
        }
    }
    #[test]
    fn filter_preserves_duration_silence_and_finite_bounds() {
        for rate in [22050, 44100, 48000] {
            assert!(mechanical(&vec![0.; 100], rate).iter().all(|&v| v == 0.));
            let input: Vec<_> = (0..rate).map(|n| (n as f32 * 0.07).sin() * 0.8).collect();
            let output = mechanical(&input, rate);
            assert_eq!(input.len(), output.len());
            assert!(output.iter().all(|v| v.is_finite() && v.abs() <= 0.65));
            assert!(output.iter().any(|v| v.abs() > 0.1));
            assert_ne!(input, output);
        }
    }
}
