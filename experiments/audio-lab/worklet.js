class ChipProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super(); const score = options.processorOptions.score;
    this.step = Math.round(sampleRate * score.tick_seconds); this.frame = 0;
    const hz = n => n === 0 ? 0 : 440 * 2 ** ((n - 69) / 12);
    this.melody = score.melody.map(hz); this.bass = score.bass.map(hz);
    this.noise = 0x12345678; this.noiseValue = 0; this.active = true;
    this.port.onmessage = event => { if (event.data === 'stop') this.active = false; };
  }
  process(inputs, outputs) {
    if (!this.active) return false;
    const output = outputs[0];
    for (let i = 0; i < output[0].length; i++) {
      const tick = Math.floor(this.frame / this.step), local = (this.frame % this.step) / sampleRate, t = this.frame / sampleRate;
      const hz = this.melody[tick % this.melody.length];
      const env = Math.min(local / .006, 1) * Math.max(0, Math.min((.25 - local) / .06, 1));
      const square = hz === 0 ? 0 : (t * hz % 1 < .25 ? 1 : -1);
      const bass = this.bass[Math.floor(tick / 8) % this.bass.length];
      const triangle = 4 * Math.abs(t * bass % 1 - .5) - 1;
      if (Math.floor(this.frame * 8000 / sampleRate) !== Math.floor(Math.max(0, this.frame - 1) * 8000 / sampleRate)) {
        this.noise ^= this.noise << 13; this.noise ^= this.noise >>> 17; this.noise ^= this.noise << 5;
        this.noiseValue = (this.noise >>> 0) / 0xffffffff * 2 - 1;
      }
      const hat = this.noiseValue * Math.exp(-local * 80) * .07;
      const kick = tick % 4 === 0 ? Math.sin(2 * Math.PI * (55 * local + 35 * (1 - Math.exp(-local * 30)) / 30)) * Math.exp(-local * 22) * .18 : 0;
      const sample = square * env * .11 + triangle * .09 + hat + kick;
      for (const channel of output) channel[i] = sample;
      this.frame++;
    }
    const second = Math.floor(this.frame / sampleRate);
    if (second !== this.reportedSecond) { this.reportedSecond = second; this.port.postMessage({ seconds: second, frames: this.frame }); }
    return true;
  }
}
registerProcessor('battleblocks-chip', ChipProcessor);
