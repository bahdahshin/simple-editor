use dioxus::prelude::*;
use rfd::FileDialog;
use std::fs;
use std::path::PathBuf;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut text = use_signal(String::new);
    let mut status = use_signal(|| String::from("Ready"));
    let mut current_file = use_signal(|| Option::<PathBuf>::None);

    let open_file = move |_| {
        if let Some(path) = FileDialog::new().pick_file() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    text.set(content);
                    status.set(format!("Opened {}", path.display()));
                    current_file.set(Some(path));
                }
                Err(err) => status.set(format!("Open failed: {err}")),
            }
        }
    };

    let save_file = move |_| {
        if let Some(path) = current_file() {
            match fs::write(&path, text()) {
                Ok(_) => status.set(format!("Saved {}", path.display())),
                Err(err) => status.set(format!("Save failed: {err}")),
            }
        } else if let Some(path) = FileDialog::new().save_file() {
            match fs::write(&path, text()) {
                Ok(_) => {
                    status.set(format!("Saved {}", path.display()));
                    current_file.set(Some(path));
                }
                Err(err) => status.set(format!("Save failed: {err}")),
            }
        }
    };

    let new_file = move |_| {
        text.set(String::new());
        current_file.set(None);
        status.set(String::from("Created a new file"));
    };

    let title = current_file()
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map_or_else(
            || "Simple Rust Text Editor — Untitled".to_string(),
            |name| format!("Simple Rust Text Editor — {name}"),
        );

    rsx! {
        document::Title { "{title}" }
        style {
            r#"
            body {
                margin: 0;
                background: #f6f6f6;
                color: #1e1e1e;
                font-family: Consolas, 'Courier New', monospace;
            }
            .app {
                display: flex;
                flex-direction: column;
                height: 100vh;
                padding: 8px;
                box-sizing: border-box;
                gap: 8px;
            }
            .toolbar {
                display: flex;
                align-items: center;
                gap: 8px;
                background: #e6e6e6;
                padding: 8px;
                border-radius: 6px;
            }
            button {
                font-family: Consolas, 'Courier New', monospace;
                font-size: 14px;
                padding: 4px 10px;
            }
            textarea {
                flex: 1;
                width: 100%;
                resize: none;
                border: 1px solid #c6c6c6;
                border-radius: 6px;
                padding: 8px;
                box-sizing: border-box;
                font-family: Consolas, 'Courier New', monospace;
                font-size: 16px;
                line-height: 1.35;
                color: #1e1e1e;
                background: #ffffff;
            }
            .status {
                white-space: nowrap;
                overflow: hidden;
                text-overflow: ellipsis;
                opacity: 0.9;
            }
            "#
        }

        div { class: "app",
            div { class: "toolbar",
                button { onclick: new_file, "New" }
                button { onclick: open_file, "Open" }
                button { onclick: save_file, "Save" }
                span { class: "status", "{status()}" }
            }
            textarea {
                value: "{text()}",
                oninput: move |evt| text.set(evt.value()),
            }
        }
    }
}
