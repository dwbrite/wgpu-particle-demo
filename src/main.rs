mod gfx_ctx;
mod pipelines;

use crate::gfx_ctx::GraphicsContext;
use crate::pipelines::{RenderStuff, MAX_PARTICLES};
use core::mem;
use std::thread::sleep;
use std::time::Duration;
use std::{ptr, thread};
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, CommandEncoderDescriptor, ComputePassDescriptor,
    ComputePipeline, PresentMode, RenderPipeline, RequestAdapterOptions, ShaderModule,
    SurfaceConfiguration, TextureUsages,
};
use winit::dpi::PhysicalSize;
use winit::event::Event;
use winit::event::*;
use winit::event_loop::ControlFlow;
use winit::event_loop::*;
use winit_input_helper::WinitInputHelper;

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
        unsafe {
            let pa: *mut BindGroup = &mut self.render_stuff.bind_groups[0];
            let pb: *mut BindGroup = &mut self.render_stuff.bind_groups[1];
            ptr::swap(pa, pb);
        }
        unsafe {
            let pa: *mut Buffer = &mut self.render_stuff.particle_swapchain[0];
            let pb: *mut Buffer = &mut self.render_stuff.particle_swapchain[1];
            ptr::swap(pa, pb);
        }

        thread::sleep(Duration::from_millis(10));
    }

    #[profiling::function]
    fn render(&self) {
        let mut encoder = self.gc.device.create_command_encoder(&Default::default());

        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("physics compute pass"),
            });
            cpass.set_pipeline(&self.render_stuff.compute_pipeline);
            cpass.set_bind_group(0, &self.render_stuff.bind_groups[0], &[]);
            cpass.dispatch_indirect(&self.render_stuff.particle_swapchain[0], 0);
        }

        {
            let mut emitpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("emission pass"),
            });
            emitpass.set_pipeline(&self.render_stuff.emit_pipeline);
            emitpass.set_bind_group(0, &self.render_stuff.bind_groups[0], &[]);
            emitpass.dispatch((5000f32 / 64f32) as u32, 1, 1);
        }

        self.gc.queue.submit(Some(encoder.finish()));
    }
}

fn main() {
    profiling::register_thread!("Main Thread");
    env_logger::init();
    let event_loop = EventLoop::new();

    let title = "void";
    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        .with_fullscreen(None)
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
