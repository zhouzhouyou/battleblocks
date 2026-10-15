(() => {
  let volume = 1;
  let music, context, effect, wanted = false, muted = false, musicEnabled = true, effectsEnabled = true, primed = false;
  function apply() {
    if (!music) return;
    music.muted = muted;
    music.volume = .2 * volume;
    if (wanted && musicEnabled) music.play().catch(e => console.warn('BGM playback pending', e.message));
    else music.pause();
  }
  function init(hasMusic, hasEffects) {
    musicEnabled = !!hasMusic; effectsEnabled = !!hasEffects;
    if (musicEnabled) { music = document.createElement('audio');music.id='bb-bgm';music.src='bgm.ogg';music.preload='none';music.loop=true;music.volume=.2;document.body.append(music); }
    if (effectsEnabled) { context=new AudioContext();fetch('crush.wav').then(r=>{if(!r.ok)throw new Error('Effect download failed');return r.arrayBuffer();}).then(b=>context.decodeAudioData(b)).then(b=>effect=b).catch(e=>console.warn(e)); }
    const unlock = () => {
      context?.resume().catch(console.warn);
      if (music && !primed) { primed=true;music.muted=true;music.play().then(()=>apply()).catch(()=>{primed=false;}); }
      else if (wanted) apply();
    };
    document.addEventListener('pointerdown',unlock);document.addEventListener('keydown',unlock);
    window.addEventListener('pagehide',()=>{music?.pause();context?.close();});
  }
  miniquad_add_plugin({name:'battleblocks_audio',version:1,register_plugin(imports){
    Object.assign(imports.env,{bb_audio_init:init,bb_audio_playing(value){const next=!!value;if(next!==wanted){wanted=next;apply();}},bb_audio_muted(value){muted=!!value;apply();},bb_audio_volume(value){volume=Math.max(0,Math.min(100,value))/100;apply();},bb_audio_crush(){if(muted||!effect||!context)return;const source=context.createBufferSource();const gain=context.createGain();source.buffer=effect;gain.gain.value=.35*volume;source.connect(gain);gain.connect(context.destination);source.onended=()=>{source.disconnect();gain.disconnect();};source.start();}});
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
