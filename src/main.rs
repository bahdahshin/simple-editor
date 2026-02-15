use eframe::{App, Frame, NativeOptions, Renderer, egui};
use rfd::FileDialog;
use std::fs;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    // Force DirectX-backed wgpu on Windows to avoid OpenGL requirements on hosts
    // where GL 2.0+ is unavailable (common in some VM/RDP configurations).
    if cfg!(target_os = "windows") {
        // SAFETY: This runs before any threads are spawned and before initializing
        // wgpu, which is the required context for process environment mutation.
        unsafe { std::env::set_var("WGPU_BACKEND", "dx12,dx11") };
    }

    run_app(Renderer::Wgpu)
}

fn run_app(renderer: Renderer) -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 650.0]),
        renderer,
        ..Default::default()
    };

    eframe::run_native(
        "Simple Rust Text Editor",
        options,
        Box::new(|_cc| Ok(Box::<EditorApp>::default())),
    )
}
#[derive(Default)]
struct EditorApp {
    text: String,
    current_file: Option<PathBuf>,
    status: String,
    dirty: bool,
}

impl EditorApp {
    fn title(&self) -> String {
        let name = self
            .current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");

        if self.dirty {
            format!("{name} *")
        } else {
            name.to_string()
        }
    }

    fn new_file(&mut self) {
        self.text.clear();
        self.current_file = None;
        self.dirty = false;
        self.status = "Created a new file".to_string();
    }

    fn open_file(&mut self) {
        if let Some(path) = FileDialog::new().pick_file() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    self.text = content;
                    self.current_file = Some(path.clone());
                    self.dirty = false;
                    self.status = format!("Opened {}", path.display());
                }
                Err(err) => {
                    self.status = format!("Failed to open file: {err}");
                }
            }
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
                self.dirty = false;
                self.status = format!("Saved {}", path.display());
            }
            Err(err) => {
                self.status = format!("Failed to save file: {err}");
            }
        }
    }
}

impl App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let mut window_title = format!("Simple Rust Text Editor — {}", self.title());
        if self.status.is_empty() {
            window_title.push_str("   ");
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(window_title));

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("New").clicked() {
                    self.new_file();
                }
                if ui.button("Open").clicked() {
                    self.open_file();
                }
                if ui.button("Save").clicked() {
                    self.save_file();
                }
                if ui.button("Save As").clicked() {
                    self.save_file_as();
                }

                ui.separator();

                let file_label = self
                    .current_file
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "No file selected".to_string());
                ui.label(format!("File: {file_label}"));
            });
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            let status = if self.status.is_empty() {
                "Ready"
            } else {
                &self.status
            };
            ui.label(status);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let response = ui.add(
                egui::TextEdit::multiline(&mut self.text)
                    .desired_rows(30)
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace)
                    .code_editor(),
            );

            if response.changed() {
                self.dirty = true;
            }
        });
    }
}
