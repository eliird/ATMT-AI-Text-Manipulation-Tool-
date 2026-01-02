use winit::event_loop::{ControlFlow, EventLoop, ActiveEventLoop};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::window::WindowId;


struct App;

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        println!("Application resumed");
    }

    fn window_event(
            &mut self,
            _event_loop: &ActiveEventLoop,
            _id: WindowId,
            _event: WindowEvent,
        ) {
        // we do nothing for now will update later
    }
}
fn main() {

    let event_loop = EventLoop::new().unwrap();

    // keep running until explicitly told to stop
    event_loop.set_control_flow(ControlFlow::Wait);

    println!("Event loop started. Press Ctrl+C to exit.");

    let mut app = App;
    event_loop.run_app(&mut app).unwrap();
}
