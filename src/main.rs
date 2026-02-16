use egui::{Key, Modifiers, TextureId};
use egui_winit::State as EguiWinitState;
use rfd::FileDialog;
use softbuffer::{Context, Surface};
use std::collections::HashMap;
use std::fs;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;
use tiny_skia::{Color, Pixmap};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

fn main() {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = EditorApp::default();
    event_loop.run_app(&mut app).expect("event loop failed");
}

#[derive(Default)]
struct EditorState {
    text: String,
    current_file: Option<PathBuf>,
    status: String,
    dirty: bool,
}

impl EditorState {
    fn title(&self) -> String {
        let name = self
            .current_file
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("Untitled");

        if self.dirty {
            format!("Simple CPU Editor - {name} *")
        } else {
            format!("Simple CPU Editor - {name}")
        }
    }

    fn mark_changed(&mut self) {
        self.dirty = true;
    }

    fn new_file(&mut self) {
        self.text.clear();
        self.current_file = None;
        self.status = "Created a new file".to_string();
        self.dirty = false;
    }

    fn open_file(&mut self) {
        if let Some(path) = FileDialog::new().pick_file() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    self.text = content;
                    self.current_file = Some(path.clone());
                    self.status = format!("Opened {}", path.display());
                    self.dirty = false;
                }
                Err(err) => {
                    self.status = format!("Open failed: {err}");
                }
            }
        }
    }

    fn save(&mut self) {
        if let Some(path) = self.current_file.clone() {
            self.save_to(path);
        } else {
            self.save_as();
        }
    }

    fn save_as(&mut self) {
        if let Some(path) = FileDialog::new().save_file() {
            self.save_to(path);
        }
    }

    fn save_to(&mut self, path: PathBuf) {
        match fs::write(&path, &self.text) {
            Ok(_) => {
                self.current_file = Some(path.clone());
                self.status = format!("Saved {}", path.display());
                self.dirty = false;
            }
            Err(err) => {
                self.status = format!("Save failed: {err}");
            }
        }
    }
}

#[derive(Clone)]
struct Texture {
    width: usize,
    height: usize,
    rgba: Vec<u8>,
}

#[derive(Default)]
struct EditorApp {
    window: Option<Rc<Window>>,
    context: Option<Context<Rc<Window>>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    egui_ctx: egui::Context,
    egui_state: Option<EguiWinitState>,
    textures: HashMap<TextureId, Texture>,
    editor: EditorState,
    start_time: Option<Instant>,
}

impl EditorApp {
    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn sync_window_title(&self) {
        if let Some(window) = &self.window {
            window.set_title(&self.editor.title());
        }
    }

    fn ui(&mut self, ctx: &egui::Context) {
        let mut changed_text = false;

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("New").clicked() {
                    self.editor.new_file();
                }
                if ui.button("Open").clicked() {
                    self.editor.open_file();
                }
                if ui.button("Save").clicked() {
                    self.editor.save();
                }
                if ui.button("Save As").clicked() {
                    self.editor.save_as();
                }
                ui.separator();
                if self.editor.status.is_empty() {
                    ui.label("Ctrl+N New | Ctrl+O Open | Ctrl+S Save");
                } else {
                    ui.label(&self.editor.status);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let response = ui.add(
                egui::TextEdit::multiline(&mut self.editor.text)
                    .desired_width(f32::INFINITY)
                    .desired_rows(32),
            );

            if response.changed() {
                changed_text = true;
            }
        });

        let save_shortcut = egui::KeyboardShortcut::new(Modifiers::COMMAND, Key::S);
        if ctx.input_mut(|input| input.consume_shortcut(&save_shortcut)) {
            self.editor.save();
        }

        let open_shortcut = egui::KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
        if ctx.input_mut(|input| input.consume_shortcut(&open_shortcut)) {
            self.editor.open_file();
        }

        let new_shortcut = egui::KeyboardShortcut::new(Modifiers::COMMAND, Key::N);
        if ctx.input_mut(|input| input.consume_shortcut(&new_shortcut)) {
            self.editor.new_file();
        }

        if changed_text {
            self.editor.mark_changed();
        }
        self.sync_window_title();
    }

    fn render(&mut self) {
        let Some(window) = &self.window else {
            return;
        };
        let Some(surface) = self.surface.as_mut() else {
            return;
        };
        let Some(egui_state) = self.egui_state.as_mut() else {
            return;
        };

        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }

        let Some(width_nz) = NonZeroU32::new(size.width) else {
            return;
        };
        let Some(height_nz) = NonZeroU32::new(size.height) else {
            return;
        };

        if surface.resize(width_nz, height_nz).is_err() {
            return;
        }

        let raw_input = egui_state.take_egui_input(window);
        self.egui_ctx
            .set_pixels_per_point(window.scale_factor() as f32);

        let full_output = self.egui_ctx.run(raw_input, |ctx| self.ui(ctx));
        egui_state.handle_platform_output(window, full_output.platform_output);

        self.apply_textures(full_output.textures_delta);

        let clipped_primitives = self
            .egui_ctx
            .tessellate(full_output.shapes, self.egui_ctx.pixels_per_point());

        let mut pixmap = match Pixmap::new(size.width, size.height) {
            Some(pixmap) => pixmap,
            None => return,
        };

        pixmap.fill(Color::from_rgba8(20, 20, 20, 255));

        for primitive in &clipped_primitives {
            self.paint_primitive(&mut pixmap, primitive);
        }

        if let Ok(mut buffer) = surface.buffer_mut() {
            let src = pixmap.data();
            for (idx, pixel) in buffer.iter_mut().enumerate() {
                let base = idx * 4;
                let r = src[base] as u32;
                let g = src[base + 1] as u32;
                let b = src[base + 2] as u32;
                *pixel = (r << 16) | (g << 8) | b;
            }
            let _ = buffer.present();
        }

        if full_output
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .is_some_and(|viewport| !viewport.repaint_delay.is_zero())
        {
            self.request_redraw();
        }
    }

    fn apply_textures(&mut self, delta: egui::TexturesDelta) {
        for (id, image_delta) in delta.set {
            let image = image_delta.image;
            let (src_width, src_height, src_rgba) = match image {
                egui::ImageData::Color(color_image) => {
                    let mut rgba = Vec::with_capacity(color_image.pixels.len() * 4);
                    for color in &color_image.pixels {
                        rgba.push(color.r());
                        rgba.push(color.g());
                        rgba.push(color.b());
                        rgba.push(color.a());
                    }
                    (color_image.size[0], color_image.size[1], rgba)
                }
                egui::ImageData::Font(font_image) => {
                    let mut rgba = Vec::with_capacity(font_image.pixels.len() * 4);
                    for alpha in font_image.srgba_pixels(None) {
                        rgba.push(alpha.r());
                        rgba.push(alpha.g());
                        rgba.push(alpha.b());
                        rgba.push(alpha.a());
                    }
                    (font_image.size[0], font_image.size[1], rgba)
                }
            };

            if let Some(pos) = image_delta.pos {
                let tex = self.textures.entry(id).or_insert_with(|| Texture {
                    width: src_width,
                    height: src_height,
                    rgba: vec![0; src_width * src_height * 4],
                });

                for row in 0..src_height {
                    let dst_start = ((pos[1] + row) * tex.width + pos[0]) * 4;
                    let src_start = row * src_width * 4;
                    let len = src_width * 4;
                    tex.rgba[dst_start..dst_start + len]
                        .copy_from_slice(&src_rgba[src_start..src_start + len]);
                }
            } else {
                self.textures.insert(
                    id,
                    Texture {
                        width: src_width,
                        height: src_height,
                        rgba: src_rgba,
                    },
                );
            }
        }

        for id in delta.free {
            self.textures.remove(&id);
        }
    }

    fn paint_primitive(&self, pixmap: &mut Pixmap, primitive: &egui::ClippedPrimitive) {
        match &primitive.primitive {
            egui::epaint::Primitive::Mesh(mesh) => {
                self.paint_mesh(pixmap, mesh, primitive.clip_rect);
            }
            egui::epaint::Primitive::Callback(_) => {}
        }
    }

    fn paint_mesh(&self, pixmap: &mut Pixmap, mesh: &egui::Mesh, clip_rect: egui::Rect) {
        let Some(texture) = self.textures.get(&mesh.texture_id) else {
            return;
        };

        let clip_min_x = clip_rect.min.x.max(0.0).floor() as i32;
        let clip_min_y = clip_rect.min.y.max(0.0).floor() as i32;
        let clip_max_x = clip_rect.max.x.min(pixmap.width() as f32).ceil() as i32;
        let clip_max_y = clip_rect.max.y.min(pixmap.height() as f32).ceil() as i32;

        for tri in mesh.indices.chunks_exact(3) {
            let a = mesh.vertices[tri[0] as usize];
            let b = mesh.vertices[tri[1] as usize];
            let c = mesh.vertices[tri[2] as usize];

            raster_triangle(
                pixmap,
                texture,
                [a, b, c],
                clip_min_x,
                clip_min_y,
                clip_max_x,
                clip_max_y,
            );
        }
    }
}

impl ApplicationHandler for EditorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = WindowAttributes::default()
            .with_title(self.editor.title())
            .with_inner_size(LogicalSize::new(960.0, 720.0));

        let window = Rc::new(
            event_loop
                .create_window(attrs)
                .expect("failed to create window"),
        );

        let context = Context::new(window.clone()).expect("failed to create softbuffer context");
        let surface =
            Surface::new(&context, window.clone()).expect("failed to create softbuffer surface");

        let mut egui_state = EguiWinitState::new(
            self.egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            window.theme(),
            None,
        );
        egui_state.egui_input_mut().time = Some(0.0);

        self.start_time = Some(Instant::now());
        self.window = Some(window);
        self.context = Some(context);
        self.surface = Some(surface);
        self.egui_state = Some(egui_state);
        self.sync_window_title();
        self.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let (Some(window), Some(egui_state)) = (&self.window, self.egui_state.as_mut()) {
            let event_response = egui_state.on_window_event(window, &event);
            if event_response.repaint {
                self.request_redraw();
            }
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => self.request_redraw(),
            WindowEvent::RedrawRequested => {
                if let (Some(start), Some(egui_state)) = (self.start_time, self.egui_state.as_mut())
                {
                    egui_state.egui_input_mut().time = Some(start.elapsed().as_secs_f64());
                }
                self.render();
            }
            _ => {}
        }
    }
}

fn raster_triangle(
    pixmap: &mut Pixmap,
    texture: &Texture,
    vertices: [egui::epaint::Vertex; 3],
    clip_min_x: i32,
    clip_min_y: i32,
    clip_max_x: i32,
    clip_max_y: i32,
) {
    let [v0, v1, v2] = vertices;

    let min_x = v0
        .pos
        .x
        .min(v1.pos.x)
        .min(v2.pos.x)
        .floor()
        .max(clip_min_x as f32) as i32;
    let min_y = v0
        .pos
        .y
        .min(v1.pos.y)
        .min(v2.pos.y)
        .floor()
        .max(clip_min_y as f32) as i32;
    let max_x = v0
        .pos
        .x
        .max(v1.pos.x)
        .max(v2.pos.x)
        .ceil()
        .min(clip_max_x as f32) as i32;
    let max_y = v0
        .pos
        .y
        .max(v1.pos.y)
        .max(v2.pos.y)
        .ceil()
        .min(clip_max_y as f32) as i32;

    if min_x >= max_x || min_y >= max_y {
        return;
    }

    let area = edge(v0.pos.x, v0.pos.y, v1.pos.x, v1.pos.y, v2.pos.x, v2.pos.y);
    if area == 0.0 {
        return;
    }

    let width = pixmap.width() as usize;
    let data = pixmap.data_mut();

    for y in min_y..max_y {
        for x in min_x..max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let w0 = edge(v1.pos.x, v1.pos.y, v2.pos.x, v2.pos.y, px, py) / area;
            let w1 = edge(v2.pos.x, v2.pos.y, v0.pos.x, v0.pos.y, px, py) / area;
            let w2 = 1.0 - w0 - w1;

            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }

            let u = w0 * v0.uv.x + w1 * v1.uv.x + w2 * v2.uv.x;
            let v = w0 * v0.uv.y + w1 * v1.uv.y + w2 * v2.uv.y;

            let tex_x = (u * texture.width as f32)
                .floor()
                .clamp(0.0, (texture.width.saturating_sub(1)) as f32)
                as usize;
            let tex_y = (v * texture.height as f32)
                .floor()
                .clamp(0.0, (texture.height.saturating_sub(1)) as f32)
                as usize;

            let tex_idx = (tex_y * texture.width + tex_x) * 4;
            let tr = texture.rgba[tex_idx] as f32 / 255.0;
            let tg = texture.rgba[tex_idx + 1] as f32 / 255.0;
            let tb = texture.rgba[tex_idx + 2] as f32 / 255.0;
            let ta = texture.rgba[tex_idx + 3] as f32 / 255.0;

            let vr =
                (w0 * v0.color.r() as f32 + w1 * v1.color.r() as f32 + w2 * v2.color.r() as f32)
                    / 255.0;
            let vg =
                (w0 * v0.color.g() as f32 + w1 * v1.color.g() as f32 + w2 * v2.color.g() as f32)
                    / 255.0;
            let vb =
                (w0 * v0.color.b() as f32 + w1 * v1.color.b() as f32 + w2 * v2.color.b() as f32)
                    / 255.0;
            let va =
                (w0 * v0.color.a() as f32 + w1 * v1.color.a() as f32 + w2 * v2.color.a() as f32)
                    / 255.0;

            let src_r = tr * vr;
            let src_g = tg * vg;
            let src_b = tb * vb;
            let src_a = (ta * va).clamp(0.0, 1.0);

            let idx = (y as usize * width + x as usize) * 4;
            let dst_r = data[idx] as f32 / 255.0;
            let dst_g = data[idx + 1] as f32 / 255.0;
            let dst_b = data[idx + 2] as f32 / 255.0;

            let out_r = src_r * src_a + dst_r * (1.0 - src_a);
            let out_g = src_g * src_a + dst_g * (1.0 - src_a);
            let out_b = src_b * src_a + dst_b * (1.0 - src_a);

            data[idx] = (out_r * 255.0).clamp(0.0, 255.0) as u8;
            data[idx + 1] = (out_g * 255.0).clamp(0.0, 255.0) as u8;
            data[idx + 2] = (out_b * 255.0).clamp(0.0, 255.0) as u8;
            data[idx + 3] = 255;
        }
    }
}

fn edge(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}
