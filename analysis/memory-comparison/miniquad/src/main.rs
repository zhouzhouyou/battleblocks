use miniquad::*;
struct Window{ctx:Box<dyn RenderingBackend>,frames:u64}
impl EventHandler for Window {
 fn update(&mut self){}
 fn draw(&mut self){self.ctx.begin_default_pass(PassAction::clear_color(0.,0.,0.,1.));self.ctx.end_render_pass();self.ctx.commit_frame();self.frames+=1;if self.frames==60 {println!("DISPLAY {:?} dpi {}",window::screen_size(),window::dpi_scale());}}
}
fn main(){let mut c=conf::Conf{window_title:"Memory comparison Miniquad".into(),window_width:1024,window_height:720,high_dpi:false,sample_count:1,..Default::default()};if std::env::var("BB_PHYSICAL_NORMALIZED").as_deref()==Ok("1"){c.window_width=512;c.window_height=360;}c.platform.swap_interval=Some(1);if std::env::var("BB_BACKEND").as_deref()==Ok("metal"){c.platform.apple_gfx_api=conf::AppleGfxApi::Metal;}
miniquad::start(c,||Box::new(Window{ctx:window::new_rendering_backend(),frames:0}) as Box<dyn EventHandler>);}
