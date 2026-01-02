mod config;

use arboard::Clipboard;
use config::Config;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use std::{thread, time::Duration};
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuId, MenuItem},
};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use reqwest::blocking::Client as HttpClient;

struct App {
    tray_icon: Option<TrayIcon>,
    quit_id: MenuId,
    config_id: MenuId,
    hotkey_manager: GlobalHotKeyManager,
    config: Config,
}

impl App {
    fn open_config(&self) {
        // Ensure config file exists with defaults
        if let Err(e) = self.config.save() {
            println!("Failed to save config: {}", e);
            return;
        }

        if let Some(path) = Config::config_path() {
            println!("Opening config file: {:?}", path);
            if let Err(e) = open::that(&path) {
                println!("Failed to open config: {}", e);
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        println!("Application resumed");
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, _event: WindowEvent) {
        // No windows
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Check for menu events
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.quit_id {
                println!("Quit clicked, exiting...");
                event_loop.exit();
            } else if event.id == self.config_id {
                self.open_config();
            }
        }

        // Check for hotkey events
        if let Ok(_event) = GlobalHotKeyEvent::receiver().try_recv() {
            println!("Hotkey pressed! Capturing text...");

            if let Some(text) = capture_text() {
                println!("--- Captured Text ---");
                println!("{}", text);
                println!("--- Translating... ---");

                match translate_text(&text, &self.config) {
                    Some(translated) => {
                        println!("--- Translation ---");
                        println!("{}", translated);
                        println!("--- End ---");

                        // Paste translation back
                        if let Err(e) = paste_text(&translated) {
                            println!("Failed to paste text: {:?}", e);
                        }
                    }
                    None => {
                        println!("Translation failed");
                    }
                }
            } else {
                println!("No text captured");
            }
        }
    }
}

fn translate_text(text: &str, config: &Config) -> Option<String> {
    let client = HttpClient::new();

    let full_prompt = format!("{}\n\n{}", config.prompt, text);

    let body = serde_json::json!({
        "model": config.model,
        "messages": [
            {
                "role": "user",
                "content": full_prompt
            }
        ]
    });

    let mut request = client
        .post(format!("{}/chat/completions", config.api_url))
        .header("Content-Type", "application/json")
        .json(&body);

    // Only add auth header if api_key is not empty
    if !config.api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", config.api_key));
    }

    let response = request.send().ok()?;
    let json: serde_json::Value = response.json().ok()?;

    json["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
}

fn paste_text(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::new()?;
    let mut enigo = Enigo::new(&Settings::default())?;

    // Put translated text in clipboard
    clipboard.set_text(text)?;

    thread::sleep(Duration::from_millis(50));

    // Simulate Ctrl+V
    enigo.key(Key::Control, Direction::Press)?;
    enigo.key(Key::Unicode('v'), Direction::Click)?;
    enigo.key(Key::Control, Direction::Release)?;

    Ok(())
}

fn capture_text() -> Option<String> {
    let mut clipboard = Clipboard::new().ok()?;
    let mut enigo = Enigo::new(&Settings::default()).ok()?;

    // Release any held modifier keys first
    enigo.key(Key::Shift, Direction::Release).ok()?;
    enigo.key(Key::Control, Direction::Release).ok()?;
    enigo.key(Key::Alt, Direction::Release).ok()?;

    // Save original clipboard
    let original = clipboard.get_text().unwrap_or_default();

    // Small delay to ensure we don't interfere with the hotkey release
    thread::sleep(Duration::from_millis(50));

    // Simulate Ctrl+C
    enigo.key(Key::Control, Direction::Press).ok()?;
    enigo.key(Key::Unicode('c'), Direction::Click).ok()?;
    enigo.key(Key::Control, Direction::Release).ok()?;

    // Wait for clipboard to update
    thread::sleep(Duration::from_millis(100));

    // Check clipboard
    let new_text = clipboard.get_text().unwrap_or_default();

    if !new_text.is_empty() && new_text != original {
        return Some(new_text);
    }

    // Nothing selected - release modifiers again and try Ctrl+A then Ctrl+C
    enigo.key(Key::Shift, Direction::Release).ok()?;
    enigo.key(Key::Control, Direction::Release).ok()?;
    enigo.key(Key::Alt, Direction::Release).ok()?;

    thread::sleep(Duration::from_millis(50));

    // Select all
    enigo.key(Key::Control, Direction::Press).ok()?;
    enigo.key(Key::Unicode('a'), Direction::Click).ok()?;
    enigo.key(Key::Control, Direction::Release).ok()?;

    thread::sleep(Duration::from_millis(100));

    // Copy
    enigo.key(Key::Control, Direction::Press).ok()?;
    enigo.key(Key::Unicode('c'), Direction::Click).ok()?;
    enigo.key(Key::Control, Direction::Release).ok()?;

    thread::sleep(Duration::from_millis(100));

    clipboard.get_text().ok()
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    // Create tray menu
    let menu = Menu::new();
    let config_item = MenuItem::new("Edit Config", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    menu.append(&config_item).unwrap();
    menu.append(&quit_item).unwrap();

    let quit_id = quit_item.id().clone();
    let config_id = config_item.id().clone();

    // Create tray icon
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Translate Tool")
        .build()
        .unwrap();

    // Register global hotkey: Ctrl+Shift+Q
    let hotkey_manager = GlobalHotKeyManager::new().unwrap();
    let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyQ);
    hotkey_manager.register(hotkey).unwrap();

    println!("Tray icon created. Press Ctrl+Shift+Q to trigger.");

    let mut app = App {
        tray_icon: Some(tray_icon),
        quit_id,
        config_id,
        hotkey_manager,
        config: Config::load(),
    };

    event_loop.run_app(&mut app).unwrap();
}
