mod gfx_ctx;
mod pipelines;

use winit::dpi::PhysicalSize;
use winit::event_loop::ControlFlow;
use wgpu::{SurfaceConfiguration, TextureUsages, PresentMode, RequestAdapterOptions, ShaderModule, RenderPipeline, BindGroupLayout, BindGroup, ComputePipeline};
use crate::gfx_ctx::GraphicsContext;
use winit::event::Event;
use winit::event::*;
use winit::event_loop::*;
use winit_input_helper::WinitInputHelper;

pub enum ShouldQuit {
    True,
    False,
}

struct State {
    gc: GraphicsContext,
    input_helper: WinitInputHelper,
}

impl State {
    #[profiling::function]
    pub fn handle_events(&mut self, event: &Event<()>) -> ShouldQuit {
        let has_events = self.input_helper.update(event);

        // if events cleared
        if has_events {
            profiling::scope!("Main Thread");
            //
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

    let mut state = State {
        gc: GraphicsContext::new(window),
        input_helper: WinitInputHelper::new(),
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if matches!(state.handle_events(&event), ShouldQuit::True) {
            *control_flow = ControlFlow::Exit
        }
    });
}
