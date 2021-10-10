mod gfx_ctx;
mod pipelines;

use crate::gfx_ctx::GraphicsContext;
use crate::pipelines::RenderStuff;

use std::thread;
use std::time::Duration;

use wgpu::ComputePassDescriptor;

use winit::event::Event;

use log::LevelFilter;
use winit::event_loop::ControlFlow;
use winit::event_loop::*;
use winit_input_helper::WinitInputHelper;

struct State {
    gc: GraphicsContext,
    render_stuff: RenderStuff,
    input_helper: WinitInputHelper,
}

impl State {
    #[profiling::function]
    pub fn handle_events(&mut self, event: &Event<()>) -> bool {
        let has_events = self.input_helper.update(event);

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

        input_helper.quit()
    }

    #[profiling::function]
    fn update(&mut self) {
        self.render_stuff.bind_group_swapchain.swap(0, 1);
        self.render_stuff.particle_swapchain.swap(0, 1);

        // sleep just to make sure our loop isn't too hot for the gpu
        thread::sleep(Duration::from_millis(10));
    }

    #[profiling::function]
    fn render(&self) {
        let mut encoder = self.gc.device.create_command_encoder(&Default::default());

        // using a new buffer every frame has no problems
        // (despite the extreme frametimes, since the buffer is 64MB)

        // let new_buf = self.gc.device.create_buffer_init(&BufferInitDescriptor {
        //     label: Some("New/Empty Particle Buffer"),
        //     contents: &vec![0u8; MAX_PARTICLES as usize * 64],
        //     usage: BufferUsages::STORAGE
        //         | BufferUsages::INDIRECT,
        // });

        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("physics compute pass"),
            });
            cpass.set_pipeline(&self.render_stuff.compute_pipeline);
            cpass.set_bind_group(0, &self.render_stuff.bind_group_swapchain[0], &[]);
            // cpass.dispatch_indirect(&new_buf, 0);
            // println!("{:?}", &self.render_stuff.particle_swapchain[0]);
            cpass.dispatch_indirect(&self.render_stuff.particle_swapchain[0], 0);
        }

        {
            let mut emitpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("emission pass"),
            });
            emitpass.set_pipeline(&self.render_stuff.emit_pipeline);
            emitpass.set_bind_group(0, &self.render_stuff.bind_group_swapchain[0], &[]);
            emitpass.dispatch((5000f32 / 64f32) as u32, 1, 1);
        }

        self.gc.queue.submit(Some(encoder.finish()));
    }
}

fn main() {
    profiling::register_thread!("Main Thread");
    env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .init();

    let event_loop = EventLoop::new();

    let title = "particle test";
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

        if state.handle_events(&event) {
            *control_flow = ControlFlow::Exit
        }
    });
}
