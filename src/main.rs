mod gfx_ctx;
mod pipelines;

use crate::gfx_ctx::GraphicsContext;
use crate::pipelines::{RenderStuff, MAX_PARTICLES};

use wgpu::{
    Color, ComputePassDescriptor, LoadOp, RenderPassColorAttachment, RenderPassDescriptor,
    SurfaceError,
};

use winit::event::Event;

use winit::event_loop::ControlFlow;
use winit::event_loop::*;
use winit::window::Fullscreen;
use winit_input_helper::WinitInputHelper;
use Fullscreen::Borderless;

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
    #[profiling::function]
    pub fn handle_events(&mut self, event: &Event<()>) -> ShouldQuit {
        let has_events = self.input_helper.update(event);

        // if events cleared
        if has_events {
            profiling::scope!("Main Thread");

            self.update();
            self.render();

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

    #[profiling::function]
    fn update(&mut self) {
        self.render_stuff.compute.bind_groups.swap(0, 1);
        self.render_stuff.render.bind_groups.swap(0, 1);
        self.render_stuff.compute.particle_swapchain.swap(0, 1);
    }

    #[profiling::function]
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
            let mut swap_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("swap pass"),
            });
            swap_pass.set_pipeline(&self.render_stuff.compute.swap_pipeline);
            swap_pass.set_bind_group(0, &self.render_stuff.compute.bind_groups[0], &[]);
            swap_pass.set_bind_group(1, &self.render_stuff.shared.compute_bind_group, &[]);
            swap_pass.dispatch(1, 1, 1);
        }

        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("physics compute pass"),
            });
            cpass.set_pipeline(&self.render_stuff.compute.compute_pipeline);
            cpass.set_bind_group(0, &self.render_stuff.compute.bind_groups[0], &[]);
            cpass.set_bind_group(1, &self.render_stuff.shared.compute_bind_group, &[]);
            // cpass.dispatch_indirect(&self.render_stuff.particle_swapchain[0], 0);
            cpass.dispatch(
                ((MAX_PARTICLES + 255) as f32 / 256f32 / 256f32) as u32,
                1,
                1,
            );
            // cpass.dispatch(64, 1, 1);
        }

        {
            let mut emitpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("emission pass"),
            });
            emitpass.set_pipeline(&self.render_stuff.compute.emit_pipeline);
            emitpass.set_bind_group(0, &self.render_stuff.compute.bind_groups[0], &[]);
            emitpass.set_bind_group(1, &self.render_stuff.shared.compute_bind_group, &[]);
            emitpass.dispatch((5255f32 / 256f32) as u32, 1, 1);
            // emitpass.dispatch(5000, 1, 1);
        }

        {
            let view = &frame_tex
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render pass descriptor"),
                color_attachments: &[RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_stuff.render.render_pipeline);
            render_pass.set_bind_group(0, &self.render_stuff.render.bind_groups[0], &[]);
            render_pass.set_bind_group(1, &self.render_stuff.shared.render_bind_group, &[]);
            render_pass.draw(0..MAX_PARTICLES * 3, 0..1);
        }

        self.gc.queue.submit(Some(encoder.finish()));
        frame_tex.present();
    }
}

fn main() {
    profiling::register_thread!("Main Thread");
    env_logger::init();
    let event_loop = EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .with_title("particles!")
        // .with_fullscreen(Some(Borderless(None)))
        .build(&event_loop)
        .unwrap();

    let mut gc = GraphicsContext::new(window);
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
