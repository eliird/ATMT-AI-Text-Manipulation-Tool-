use winit::event_loop::{ControlFlow, EventLoop, ActiveEventLoop};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::window::WindowId;

use tray_icon::{
    menu::{Menu, MenuItem, MenuEvent, MenuId},
    TrayIcon, TrayIconBuilder,
};

use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};

struct App{
    tray_icon: Option<TrayIcon>,
    quit_id: MenuId,
    hotkey_manager: GlobalHotKeyManager,
}

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
        // No window events to handle
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // check for menu events
        if let Ok(event) = MenuEvent::receiver().try_recv(){
            if event.id == self.quit_id {
                println!("Quit menu item clicked. Exiting...");
                event_loop.exit();
            }
        };

        // check for the hotkey events
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            println!("Hotkey event received: {:?}", event);
        }
    }
}
fn main() {

    let event_loop = EventLoop::new().unwrap();
    // keep running until explicitly told to stop
    event_loop.set_control_flow(ControlFlow::Wait);

    //create a tray menu
    let menu = Menu::new();
    let quite_item = MenuItem::new("Quit", true, None);
    menu.append(&quite_item).unwrap();
    let quit_id = quite_item.id().clone();

    // create tray icon, empty for now
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .build()
        .unwrap();
    
    // Register a global hotkey Ctrl+Shift+H
    let hotkey_manager = GlobalHotKeyManager::new().unwrap();
    let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyH);
    hotkey_manager.register(hotkey).unwrap();

    println!("Tray icon created. right click to see the menu.");
    

    let mut app = App { 
        tray_icon: Some(tray_icon),
        quit_id: quit_id,
        hotkey_manager: hotkey_manager,
    };
       // listen for menu events
    println!("Event loop started. Press Ctrl+C to exit.");
    event_loop.run_app(&mut app).unwrap();
}
