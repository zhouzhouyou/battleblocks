(() => {
  let volume = 1;
  let channelVolumes = [1, 1, 1];
  let effectsGain;
  let music, context, effect, wanted = false, muted = false, musicEnabled = true, effectsEnabled = true, primed = false, musicPrimed = false;
  const stopSpeech = () => { window.speechSynthesis?.cancel(); };
  function speak(ptr, len, zh) {
    if (!effectsEnabled || muted || volume === 0 || channelVolumes[2] === 0 || !window.speechSynthesis || !primed) return;
    const synth = window.speechSynthesis;
    synth.cancel();
    const utterance = new SpeechSynthesisUtterance(new TextDecoder().decode(new Uint8Array(wasm_memory.buffer, ptr, len)));
    utterance.lang = zh ? 'zh-CN' : 'en-US';
    const voices = synth.getVoices().filter(v => v.lang.toLowerCase().startsWith(zh ? 'zh' : 'en'));
    utterance.voice = voices.find(v => v.localService && /Samantha|Tingting|婷婷/i.test(v.name)) || voices.find(v => v.localService) || voices[0] || null;
    utterance.rate = .9; utterance.pitch = .85; utterance.volume = volume * channelVolumes[2];
    try { synth.speak(utterance); } catch (error) { console.warn('System speech unavailable', error); }
  }
  function portal() {
    if (!effectsEnabled || muted || volume === 0 || channelVolumes[1] === 0 || !context || context.state !== 'running') return;
    const rate = context.sampleRate;
    const buffer = context.createBuffer(1, Math.floor(rate * .28), rate);
    const data = buffer.getChannelData(0);
    for (let n = 0; n < data.length; n++) {
      const t = n / rate, phase = Math.PI * 2 * (260 * t + 1800 * t * t);
      data[n] = Math.min(t / .015, 1) * Math.max(0, 1 - t / .28) ** 2 * (Math.sin(phase) + .3 * Math.sin(phase * 1.5)) * .2;
    }
    const source = context.createBufferSource(), gain = context.createGain();
    source.buffer = buffer; gain.gain.value = 1;
    source.connect(gain); gain.connect(effectsGain);
    source.onended = () => { source.disconnect(); gain.disconnect(); }; source.start();
  }
  function apply() {
    if (effectsGain) effectsGain.gain.value = muted ? 0 : volume * channelVolumes[1];
    if (!music) return;
    music.muted = muted;
    music.volume = .2 * volume * channelVolumes[0];
    if (wanted && musicEnabled) music.play().catch(e => console.warn('BGM playback pending', e.message));
    else music.pause();
  }
  function init(hasMusic, hasEffects) {
    musicEnabled = !!hasMusic; effectsEnabled = !!hasEffects;
    if (musicEnabled) { music = document.createElement('audio');music.id='bb-bgm';music.src='bgm.ogg';music.preload='none';music.loop=true;music.volume=.2;document.body.append(music); }
    if (effectsEnabled) { context=new AudioContext();effectsGain=context.createGain();effectsGain.connect(context.destination);apply();fetch('crush.wav').then(r=>{if(!r.ok)throw new Error('Effect download failed');return r.arrayBuffer();}).then(b=>context.decodeAudioData(b)).then(b=>effect=b).catch(e=>console.warn(e)); }
    const unlock = () => {
      primed = true;
      context?.resume().catch(console.warn);
      if (music && !musicPrimed) { musicPrimed=true;music.muted=true;music.play().then(()=>apply()).catch(()=>{musicPrimed=false;}); }
      else if (wanted) apply();
    };
    document.addEventListener('pointerdown',unlock);document.addEventListener('keydown',unlock);
    window.addEventListener('pagehide',()=>{stopSpeech();music?.pause();context?.close();});
    document.addEventListener('visibilitychange',()=>{if(document.hidden)stopSpeech();});
  }
  miniquad_add_plugin({name:'battleblocks_audio',version:1,register_plugin(imports){
    Object.assign(imports.env, {
      bb_audio_init: init,
      bb_audio_speak: speak,
      bb_audio_stop_speech: stopSpeech,
      bb_audio_portal: portal,
      bb_audio_playing(value) {
        const next = !!value;
        if (next !== wanted) { wanted = next; apply(); }
      },
      bb_audio_muted(value) {
        muted = !!value;
        if (muted) stopSpeech();
        apply();
      },
      bb_audio_volume(value) {
        const next = Math.max(0, Math.min(100, value)) / 100;
        if (next !== volume) stopSpeech();
        volume = next;
        apply();
      },
      bb_audio_channels(musicLevel, effectsLevel, voiceLevel) {
        const next = [musicLevel, effectsLevel, voiceLevel].map(v => Math.max(0, Math.min(100, v)) / 100);
        if (next[2] !== channelVolumes[2]) stopSpeech();
        channelVolumes = next;
        apply();
      },
      bb_audio_crush() {
        if (muted || channelVolumes[1] === 0 || !effect || !context) return;
        const source = context.createBufferSource();
        const gain = context.createGain();
        source.buffer = effect;
        gain.gain.value = .35;
        source.connect(gain);
        gain.connect(effectsGain);
        source.onended = () => { source.disconnect(); gain.disconnect(); };
        source.start();
      }
    });
  }});
})();

miniquad_add_plugin({name:'battleblocks_debug',version:1,register_plugin(imports){
  imports.env.bb_debug_download=function(ptr,len){
    const bytes=new Uint8Array(wasm_memory.buffer,ptr,len).slice();
    const url=URL.createObjectURL(new Blob([bytes],{type:'application/x-ndjson'}));
    const link=document.createElement('a');link.href=url;link.download='battleblocks-session.jsonl';
    document.body.appendChild(link);link.click();link.remove();setTimeout(()=>URL.revokeObjectURL(url),1000);
  };
}});
