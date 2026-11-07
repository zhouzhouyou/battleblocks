/* Browser audio bridge contract without a window or external speech service. */
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const path = require('node:path');
function browser({speech = true, effects = true} = {}) {
  const handlers = {}, imports = {env: {}}, spoken = [], buffers = [];
  let cancelled = 0;
  const synth = {speaking: false, pending: false,
    getVoices: () => [{name: 'Samantha', lang: 'en-US', localService: true}, {name: 'Tingting', lang: 'zh-CN', localService: true}],
    speak: u => spoken.push(u), cancel: () => { cancelled++; synth.speaking = false; }};
  class Context {
    constructor() { this.sampleRate = 48000; this.state = 'running'; }
    resume() { return Promise.resolve(); }
    close() {}
    decodeAudioData() { return Promise.resolve({}); }
    createBuffer(_, size) { const data = new Float32Array(size); buffers.push(data); return {getChannelData: () => data}; }
    createBufferSource() { return {connect() {}, disconnect() {}, start() {}}; }
    createGain() { return {gain: {}, connect() {}, disconnect() {}}; }
  }
  const bytes = new TextEncoder().encode('空间校准完成。');
  const window = {speechSynthesis: speech ? synth : undefined, addEventListener: (k, fn) => handlers[k] = fn};
  const sandbox = {window, AudioContext: Context, SpeechSynthesisUtterance: function(text) { this.text = text; },
    document: {addEventListener: (k, fn) => handlers[k] = fn},
    miniquad_add_plugin: p => p.register_plugin(imports),
    fetch: () => Promise.resolve({ok: true, arrayBuffer: () => Promise.resolve(new ArrayBuffer(1))}),
    wasm_memory: {buffer: bytes.buffer}, TextDecoder, Uint8Array, console};
  vm.runInNewContext(fs.readFileSync(path.join(__dirname, '../src/audio/web.js'), 'utf8'), sandbox);
  const api = imports.env;
  api.bb_audio_init(0, effects ? 1 : 0);
  return {api, spoken, buffers, handlers, synth, length: bytes.length, cancelled: () => cancelled};
}
const b = browser();
b.api.bb_audio_speak(0, b.length, 1);
assert.equal(b.spoken.length, 0, 'speech waits for a user gesture');
b.handlers.pointerdown();
b.api.bb_audio_speak(0, b.length, 1);
assert.equal(b.spoken[0].text, '空间校准完成。');
assert.equal(b.spoken[0].lang, 'zh-CN');
assert.equal(b.spoken[0].voice.name, 'Tingting');
b.api.bb_audio_speak(0, b.length, 0);
assert.equal(b.spoken.length, 2, 'each result starts its cue');
b.synth.speaking = true; const beforeReplacement = b.cancelled();
b.api.bb_audio_speak(0, b.length, 0);
assert.equal(b.spoken.length, 3, 'a new result replaces the current cue');
assert.equal(b.cancelled(), beforeReplacement + 1);
b.api.bb_audio_muted(1); b.api.bb_audio_portal();
assert.equal(b.buffers.length, 0);
assert.ok(b.cancelled() > 0);
b.api.bb_audio_muted(0); b.api.bb_audio_volume(35);
b.api.bb_audio_speak(0, b.length, 0);
assert.equal(b.spoken[3].volume, .35);
b.api.bb_audio_portal();
assert.equal(b.buffers[0].length, 13440);
assert.ok(b.buffers[0].every(v => Number.isFinite(v) && Math.abs(v) <= .26));
b.api.bb_audio_volume(0); b.api.bb_audio_speak(0, b.length, 0);
assert.equal(b.spoken.length, 4);
b.handlers.pagehide();
for (const options of [{speech: false}, {effects: false}]) {
  const disabled = browser(options); disabled.handlers.pointerdown();
  disabled.api.bb_audio_speak(0, disabled.length, 1);
  assert.equal(disabled.spoken.length, 0);
}
console.log('PASS: browser speech, gesture, language, mute, volume, result replacement, fallback and portal PCM');
