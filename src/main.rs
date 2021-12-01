// mod framework;
mod gfx_ctx;
mod pipelines;

use crate::gfx_ctx::GraphicsContext;
use crate::pipelines::{RenderStuff, Uniforms, MAX_PARTICLES};

use wgpu::{
    Color, ComputePassDescriptor, LoadOp, RenderBundle, RenderPassColorAttachment,
    RenderPassDescriptor, SurfaceError,
};

use winit::event::Event;
use winit::event::VirtualKeyCode::P;

use std::panic;

#[cfg(target_arch = "wasm32")]
use web_sys::window;

use winit::event_loop::ControlFlow;
use winit::event_loop::*;
use winit::window::Fullscreen;
use winit::window::Fullscreen::Borderless;
use winit_input_helper::WinitInputHelper;
// extern crate console_error_panic_hook;
// use std::panic;

pub enum ShouldQuit {
    True,
    False,
}

struct State {
    gc: GraphicsContext,
    render_stuff: RenderStuff,
    input_helper: WinitInputHelper,
}

impl State {
    #[cfg_attr(feature = "tracy", profiling::function)]
    pub fn handle_events(&mut self, event: &Event<()>) -> ShouldQuit {
        let has_events = self.input_helper.update(event);

        // if events cleared
        if has_events {
            #[cfg(feature = "tracy")]
            profiling::scope!("Main Thread");

            self.update();
            self.render();

            #[cfg(feature = "tracy")]
            profiling::finish_frame!();
        }

        let input_helper = &mut self.input_helper;

        if let Some(size) = input_helper.window_resized() {
            self.gc.resize(size);
        }

        if input_helper.quit() {
            ShouldQuit::True
        } else {
            ShouldQuit::False
        }
    }

    #[cfg_attr(feature = "tracy", profiling::function)]
    fn update(&mut self) {
        if let Some(mouse) = self.input_helper.mouse() {
            let mouse = (
                mouse.0 / self.gc.size.width as f32,
                mouse.1 / self.gc.size.height as f32,
            );

            let mut uniforms = Uniforms {
                paused: 0,
                mouse_down: 0,
                mouse_pos_last: [(mouse.0 * 2.0) - 1.0, mouse.1 * (-2.0) + 1.0],
            };

            if self.input_helper.mouse_pressed(0) || self.input_helper.mouse_held(0) {
                uniforms.mouse_down = 1;
            }

            self.gc.queue.write_buffer(
                &self.render_stuff.shared.uniforms,
                0,
                bytemuck::cast_slice(&[uniforms]),
            );
        }
    }

    #[cfg_attr(feature = "tracy", profiling::function)]
    fn render(&self) {
        let frame_tex = {
            let frame = self.gc.surface.get_current_texture();

            match frame {
                Ok(_f) => _f,
                Err(SurfaceError::Outdated) => {
                    self.gc.surface.configure(&self.gc.device, &self.gc.config);
                    self.gc
                        .surface
                        .get_current_texture()
                        .expect("swapchain failed to get current frame (twice)")
                }
                Err(SurfaceError::Timeout) => {
                    return; /*assume gpu is asleep?*/
                }
                _ => frame.expect("swapchain failed to get current frame"),
            }
        };

        let mut encoder = self.gc.device.create_command_encoder(&Default::default());

        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("physics compute pass"),
            });
            cpass.set_pipeline(&self.render_stuff.compute.compute_pipeline);
            cpass.set_bind_group(0, &self.render_stuff.compute.bind_group, &[]);
            cpass.set_bind_group(1, &self.render_stuff.shared.compute_bind_group, &[]);
            cpass.dispatch(
                ((MAX_PARTICLES + 255) as f32 / 256f32 / 256f32) as u32,
                1,
                1,
            );
        }

        {
            let mut emitpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("emission pass"),
            });
            emitpass.set_pipeline(&self.render_stuff.compute.emit_pipeline);
            emitpass.set_bind_group(0, &self.render_stuff.compute.bind_group, &[]);
            emitpass.set_bind_group(1, &self.render_stuff.shared.compute_bind_group, &[]);
            emitpass.dispatch(1, 1, 1);
        }

        {
            let view = &frame_tex
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let ops = wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: true,
            };

            let rpass_color_attachment = if self.gc.sample_count == 1 {
                wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops,
                }
            } else {
                wgpu::RenderPassColorAttachment {
                    view: &self.gc.msaa_framebuffer,
                    resolve_target: Some(view),
                    ops,
                }
            };

            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render pass descriptor"),
                color_attachments: &[rpass_color_attachment],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_stuff.render.render_pipeline);
            render_pass.set_bind_group(0, &self.render_stuff.render.bind_group, &[]);
            render_pass.set_bind_group(1, &self.render_stuff.shared.render_bind_group, &[]);
            render_pass.set_bind_group(2, &self.render_stuff.render.texture_bind_group, &[]);
            render_pass.draw(0..((MAX_PARTICLES) * 3), 0..1);
        }

        self.gc.queue.submit(Some(encoder.finish()));
        frame_tex.present();
    }
}

async fn async_main() {
    #[cfg(feature = "tracy")]
    profiling::register_thread!("Main Thread");

    // env_logger::init();
    let event_loop = EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .with_title("particles!")
        // .with_fullscreen(Some(Borderless(None)))
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        panic::set_hook(Box::new(console_error_panic_hook::hook));

        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
    }

    let mut gc = GraphicsContext::new(window, 1).await;
    let render_stuff = RenderStuff::new(&mut gc);

    let mut state = State {
        gc,
        render_stuff,
        input_helper: WinitInputHelper::new(),
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if matches!(state.handle_events(&event), ShouldQuit::True) {
            *control_flow = ControlFlow::Exit
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    futures::executor::block_on(async_main());
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::{prelude::*, JsCast};
    wasm_bindgen_futures::spawn_local(async_main());
}
