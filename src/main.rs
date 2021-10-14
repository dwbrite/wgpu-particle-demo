mod gfx_ctx;
mod pipelines;

use crate::gfx_ctx::GraphicsContext;
use crate::pipelines::{RenderStuff, MAX_PARTICLES};

use wgpu::ComputePassDescriptor;

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
        self.render_stuff.compute.particle_swapchain.swap(0, 1);
    }

    #[profiling::function]
    fn render(&self) {
        let mut encoder = self.gc.device.create_command_encoder(&Default::default());

        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("physics compute pass"),
            });
            cpass.set_pipeline(&self.render_stuff.compute.compute_pipeline);
            cpass.set_bind_group(0, &self.render_stuff.compute.bind_groups[0], &[]);
            cpass.set_bind_group(1, &self.render_stuff.shared.bind_group, &[]);
            // cpass.dispatch_indirect(&self.render_stuff.particle_swapchain[0], 0);
            cpass.dispatch(((MAX_PARTICLES + 63) as f32 / 64f32) as u32, 1, 1);
        }

        {
            let mut emitpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("emission pass"),
            });
            emitpass.set_pipeline(&self.render_stuff.compute.emit_pipeline);
            emitpass.set_bind_group(0, &self.render_stuff.compute.bind_groups[0], &[]);
            emitpass.set_bind_group(1, &self.render_stuff.shared.bind_group, &[]);
            emitpass.dispatch((5000f32 / 64f32) as u32, 1, 1);
        }

        self.gc.queue.submit(Some(encoder.finish()));
    }
}

fn main() {
    profiling::register_thread!("Main Thread");
    env_logger::init();
    let event_loop = EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .with_title("particles!")
        .with_fullscreen(Some(Borderless(None)))
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
