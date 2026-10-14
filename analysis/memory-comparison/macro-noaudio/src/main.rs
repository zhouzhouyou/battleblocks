use macroquad::prelude::*;
#[cfg(feature="audio")]
use macroquad::audio::{load_sound_from_bytes,play_sound,PlaySoundParams};
fn conf()->macroquad::conf::Conf{let mut c=Conf{window_title:"Memory comparison Macroquad".into(),window_width:1024,window_height:720,high_dpi:false,sample_count:1,..Default::default()};if std::env::var("BB_PHYSICAL_NORMALIZED").as_deref()==Ok("1"){c.window_width=512;c.window_height=360;}c.platform.swap_interval=Some(1);if std::env::var("BB_BACKEND").as_deref()==Ok("metal"){c.platform.apple_gfx_api=miniquad::conf::AppleGfxApi::Metal;}
 let small=std::env::var("BB_SMALL_BUFFERS").as_deref()==Ok("1");macroquad::conf::Conf{miniquad_conf:c,draw_call_vertex_capacity:if small{512}else{10000},draw_call_index_capacity:if small{768}else{5000},..Default::default()}}
#[macroquad::main(conf)]
async fn main(){
 let mode=std::env::var("BB_MODE").unwrap_or_else(|_|"empty".into());
 let font=if mode.contains("font"){Some(load_ttf_font_from_bytes(include_bytes!("../../assets/ui.ttf")).unwrap())}else{None};
 #[cfg(feature="audio")]
 let music=if mode.contains("music"){Some(load_sound_from_bytes(include_bytes!("../../assets/bgm.ogg")).await.unwrap())}else{None};
 #[cfg(feature="audio")]
 if mode.contains("play"){if let Some(s)=&music{play_sound(s,PlaySoundParams{looped:true,volume:0.});}}
 if mode.contains("prewarm"){if let Some(f)=&font{let chars="方块战斗 BattleBlocks生命 700 / 700魔力 300 / 300选择两个相邻方块技能：剑刃斩击P：暂停 R：重开 M：静音".chars().collect::<Vec<_>>();f.populate_font_cache(&chars,30);}}
 let mut frames=0;
 loop{clear_background(BLACK);
 if mode.contains("framewarm") {if let Some(f)=&font {let size=if mode.contains("bucket"){[16u16,24,32,48,64].into_iter().find(|&x|x>=16+(frames%65) as u16).unwrap_or(64)}else{16+(frames%65) as u16};let chars="方块战斗 BattleBlocks生命 700 / 700魔力 300 / 300选择两个相邻方块技能：剑刃斩击P：暂停 R：重开 M：静音".chars().collect::<Vec<_>>();f.populate_font_cache(&chars,size);}}
 if mode.contains("draw"){for (i,text) in ["方块战斗 BattleBlocks","生命 700 / 700","魔力 300 / 300","选择两个相邻方块","技能：剑刃斩击","P：暂停 R：重开 M：静音"].iter().enumerate(){draw_text_ex(text,30.,60.+i as f32*60.,TextParams{font:font.as_ref(),font_size:if mode.contains("resize") {let requested=16+(frames%65) as u16;if mode.contains("bucket"){[16u16,24,32,48,64].into_iter().find(|&x|x>=requested).unwrap_or(64)}else{requested}} else {30},color:WHITE,..Default::default()});}}
 frames+=1;if frames==60{println!("DISPLAY {:?} dpi {}",miniquad::window::screen_size(),miniquad::window::dpi_scale());}
 next_frame().await;}
}
