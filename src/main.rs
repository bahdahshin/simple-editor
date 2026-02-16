use rfd::FileDialog;
use softbuffer::{Context, Surface};
use std::fs;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

const BG: u32 = 0x00F6F6F6;
const FG: u32 = 0x001E1E1E;
const STATUS_BG: u32 = 0x00E6E6E6;
const STATUS_H: u32 = 22;
const CHAR_W: u32 = 8;
const CHAR_H: u32 = 8;
const PADDING: u32 = 8;

fn main() {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = EditorApp::default();
    event_loop.run_app(&mut app).expect("event loop failed");
}

#[derive(Default)]
struct EditorApp {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    context: Option<Context<Rc<Window>>>,
    text: String,
    current_file: Option<PathBuf>,
    status: String,
    modifiers: ModifiersState,
}

impl EditorApp {
    fn title(&self) -> String {
        let name = self
            .current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");
        format!("Simple Rust Text Editor — {name}")
    }

    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn save_file(&mut self) {
        if let Some(path) = self.current_file.clone() {
            self.save_to(path);
        } else {
            self.save_file_as();
        }
    }

    fn save_file_as(&mut self) {
        if let Some(path) = FileDialog::new().save_file() {
            self.save_to(path);
        }
    }

    fn save_to(&mut self, path: PathBuf) {
        match fs::write(&path, &self.text) {
            Ok(_) => {
                self.current_file = Some(path.clone());
                self.status = format!("Saved {}", path.display());
            }
            Err(err) => {
                self.status = format!("Save failed: {err}");
            }
        }
        if let Some(window) = &self.window {
            window.set_title(&self.title());
        }
    }

    fn open_file(&mut self) {
        if let Some(path) = FileDialog::new().pick_file() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    self.text = content;
                    self.current_file = Some(path.clone());
                    self.status = format!("Opened {}", path.display());
                    if let Some(window) = &self.window {
                        window.set_title(&self.title());
                    }
                }
                Err(err) => {
                    self.status = format!("Open failed: {err}");
                }
            }
        }
    }

    fn new_file(&mut self) {
        self.text.clear();
        self.current_file = None;
        self.status = "Created a new file".to_string();
        if let Some(window) = &self.window {
            window.set_title(&self.title());
        }
    }

    fn draw(&mut self, width: u32, height: u32) {
        let Some(surface) = self.surface.as_mut() else {
            return;
        };

        if width == 0 || height == 0 {
            return;
        }

        let Some(w_nz) = NonZeroU32::new(width) else {
            return;
        };
        let Some(h_nz) = NonZeroU32::new(height) else {
            return;
        };
        if surface.resize(w_nz, h_nz).is_err() {
            return;
        }

        let Ok(mut buffer) = surface.buffer_mut() else {
            return;
        };

        for px in buffer.iter_mut() {
            *px = BG;
        }

        let status_top = height.saturating_sub(STATUS_H);
        fill_rect(
            &mut buffer,
            width,
            0,
            status_top,
            width,
            STATUS_H,
            STATUS_BG,
        );

        let text_height = status_top.saturating_sub(PADDING);
        draw_multiline_text(
            &mut buffer,
            width,
            PADDING,
            PADDING,
            &self.text,
            FG,
            text_height,
        );

        let status = if self.status.is_empty() {
            "Shortcuts: Ctrl+N New | Ctrl+O Open | Ctrl+S Save"
        } else {
            &self.status
        };
        draw_text(
            &mut buffer,
            width,
            PADDING,
            status_top + 7,
            status,
            FG,
            width.saturating_sub(PADDING),
        );

        let _ = buffer.present();
    }
}

impl ApplicationHandler for EditorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = WindowAttributes::default()
            .with_title(self.title())
            .with_inner_size(LogicalSize::new(900.0, 650.0));
        let window = Rc::new(
            event_loop
                .create_window(attrs)
                .expect("failed to create window"),
        );

        let context = Context::new(window.clone()).expect("failed to create softbuffer context");
        let surface =
            Surface::new(&context, window.clone()).expect("failed to create softbuffer surface");

        self.context = Some(context);
        self.surface = Some(surface);
        self.window = Some(window);
        self.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    let size = window.inner_size();
                    self.draw(size.width, size.height);
                }
            }
            WindowEvent::Resized(_) => self.request_redraw(),
            WindowEvent::ModifiersChanged(m) => self.modifiers = m.state(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    let ctrl = self.modifiers.control_key();
                    match &event.logical_key {
                        Key::Character(ch) if ctrl && ch.eq_ignore_ascii_case("s") => {
                            self.save_file();
                            self.request_redraw();
                            return;
                        }
                        Key::Character(ch) if ctrl && ch.eq_ignore_ascii_case("o") => {
                            self.open_file();
                            self.request_redraw();
                            return;
                        }
                        Key::Character(ch) if ctrl && ch.eq_ignore_ascii_case("n") => {
                            self.new_file();
                            self.request_redraw();
                            return;
                        }
                        Key::Named(NamedKey::Backspace) => {
                            self.text.pop();
                        }
                        Key::Named(NamedKey::Enter) => self.text.push('\n'),
                        _ => {
                            if !ctrl {
                                if let Some(text) = &event.text {
                                    for ch in text.chars() {
                                        if !ch.is_control() {
                                            self.text.push(ch);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    self.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn fill_rect(buffer: &mut [u32], width: u32, x: u32, y: u32, w: u32, h: u32, color: u32) {
    let max_y = y.saturating_add(h);
    let max_x = x.saturating_add(w);
    for yy in y..max_y {
        for xx in x..max_x {
            let idx = (yy * width + xx) as usize;
            if let Some(px) = buffer.get_mut(idx) {
                *px = color;
            }
        }
    }
}

fn draw_multiline_text(
    buffer: &mut [u32],
    width: u32,
    x: u32,
    y: u32,
    text: &str,
    color: u32,
    max_height: u32,
) {
    let mut yy = y;
    for line in text.lines() {
        if yy.saturating_add(CHAR_H) > max_height {
            break;
        }
        draw_text(buffer, width, x, yy, line, color, width.saturating_sub(x));
        yy = yy.saturating_add(CHAR_H + 2);
    }
}

fn draw_text(buffer: &mut [u32], width: u32, x: u32, y: u32, text: &str, color: u32, max_x: u32) {
    let mut xx = x;
    for ch in text.chars() {
        if xx.saturating_add(CHAR_W) >= max_x {
            break;
        }
        draw_glyph(buffer, width, xx, y, ch, color);
        xx = xx.saturating_add(CHAR_W);
    }
}

fn draw_glyph(buffer: &mut [u32], width: u32, x: u32, y: u32, ch: char, color: u32) {
    let segments = glyph_segments(ch);
    draw_segment_glyph(buffer, width, x, y, segments, color);
}

fn glyph_segments(ch: char) -> u16 {
    // 7-segment style bits: 0 top, 1 UL, 2 UR, 3 mid, 4 LL, 5 LR, 6 bottom, 7 center vert,
    // 8 dot, 9 comma, 10 apostrophe.
    const TOP: u16 = 1 << 0;
    const UL: u16 = 1 << 1;
    const UR: u16 = 1 << 2;
    const MID: u16 = 1 << 3;
    const LL: u16 = 1 << 4;
    const LR: u16 = 1 << 5;
    const BOT: u16 = 1 << 6;
    const CV: u16 = 1 << 7;
    const DOT: u16 = 1 << 8;
    const COMMA: u16 = 1 << 9;
    const APOSTROPHE: u16 = 1 << 10;

    match ch {
        '0' => TOP | UL | UR | LL | LR | BOT,
        '1' => UR | LR,
        '2' => TOP | UR | MID | LL | BOT,
        '3' => TOP | UR | MID | LR | BOT,
        '4' => UL | UR | MID | LR,
        '5' => TOP | UL | MID | LR | BOT,
        '6' => TOP | UL | MID | LL | LR | BOT,
        '7' => TOP | UR | LR,
        '8' => TOP | UL | UR | MID | LL | LR | BOT,
        '9' => TOP | UL | UR | MID | LR | BOT,
        'A' | 'a' => TOP | UL | UR | MID | LL | LR,
        'B' | 'b' => UL | MID | LL | LR | BOT,
        'C' | 'c' => TOP | UL | LL | BOT,
        'D' | 'd' => UR | MID | LL | LR | BOT,
        'E' | 'e' => TOP | UL | MID | LL | BOT,
        'F' | 'f' => TOP | UL | MID | LL,
        'G' | 'g' => TOP | UL | LL | LR | BOT | MID,
        'H' | 'h' => UL | UR | MID | LL | LR,
        'I' | 'i' => UR | LR,
        'J' | 'j' => UR | LR | BOT,
        'K' | 'k' => UL | MID | LL | UR | LR,
        'L' | 'l' => UL | LL | BOT,
        'M' | 'm' => UL | UR | LL | LR | CV,
        'N' | 'n' => UL | UR | LL | LR | CV,
        'O' | 'o' => TOP | UL | UR | LL | LR | BOT,
        'P' | 'p' => TOP | UL | UR | MID | LL,
        'Q' | 'q' => TOP | UL | UR | LL | LR | BOT | CV,
        'R' | 'r' => TOP | UL | UR | MID | LL | LR,
        'S' | 's' => TOP | UL | MID | LR | BOT,
        'T' | 't' => TOP | CV,
        'U' | 'u' => UL | UR | LL | LR | BOT,
        'V' | 'v' => UL | UR | LL | LR | BOT,
        'W' | 'w' => UL | UR | LL | LR | BOT | CV,
        'X' | 'x' => UL | UR | MID | LL | LR,
        'Y' | 'y' => UL | UR | MID | LR | BOT,
        'Z' | 'z' => TOP | UR | MID | LL | BOT,
        '+' => MID | CV,
        '-' => MID,
        '_' => BOT,
        '|' => CV,
        '/' => UR | LL,
        '\\' => UL | LR,
        ':' => DOT,
        ';' => DOT | COMMA,
        ',' => COMMA,
        '.' => DOT,
        '\'' => APOSTROPHE,
        '!' => CV | DOT,
        '?' => TOP | UR | MID | DOT,
        '(' => UL | LL,
        ')' => UR | LR,
        '[' => TOP | UL | LL | BOT,
        ']' => TOP | UR | LR | BOT,
        '=' => TOP | MID,
        '"' => APOSTROPHE,
        ' ' => 0,
        _ => TOP | UL | UR | MID | LL | LR | BOT,
    }
}

fn draw_segment_glyph(buffer: &mut [u32], width: u32, x: u32, y: u32, segments: u16, color: u32) {
    let mut plot = |px: u32, py: u32| {
        let idx = (py * width + px) as usize;
        if let Some(pixel) = buffer.get_mut(idx) {
            *pixel = color;
        }
    };

    let has = |bit: u8| (segments & (1 << bit)) != 0;

    if has(0) {
        for xx in x + 1..=x + 5 {
            plot(xx, y);
        }
    }
    if has(1) {
        for yy in y + 1..=y + 3 {
            plot(x, yy);
        }
    }
    if has(2) {
        for yy in y + 1..=y + 3 {
            plot(x + 6, yy);
        }
    }
    if has(3) {
        for xx in x + 1..=x + 5 {
            plot(xx, y + 3);
        }
    }
    if has(4) {
        for yy in y + 4..=y + 6 {
            plot(x, yy);
        }
    }
    if has(5) {
        for yy in y + 4..=y + 6 {
            plot(x + 6, yy);
        }
    }
    if has(6) {
        for xx in x + 1..=x + 5 {
            plot(xx, y + 6);
        }
    }
    if has(7) {
        for yy in y + 1..=y + 6 {
            plot(x + 3, yy);
        }
    }
    if has(8) {
        plot(x + 3, y + 7);
    }
    if has(9) {
        plot(x + 3, y + 7);
        plot(x + 2, y + 7);
    }
    if has(10) {
        plot(x + 4, y + 1);
        plot(x + 4, y);
    }
}
