use eframe::egui;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[allow(dead_code)]
struct FolderSize {
    path: PathBuf,
    bytes: u64,
}

#[allow(dead_code)]
fn folder_size(path: &Path, rows: &mut Vec<FolderSize>) -> io::Result<u64> {
    let mut total = 0;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child_path = entry.path();
        let metadata = fs::metadata(&child_path)?;

        if metadata.is_dir() {
            // Recursively calculate this subfolder first.
            total += folder_size(&child_path, rows)?;
        } else if metadata.is_file() {
            total += metadata.len();
        }
    }

    // Record this folder after its contents have been counted.
    rows.push(FolderSize {
        path: path.to_path_buf(),
        bytes: total,
    });

    Ok(total)
}

fn main() -> eframe::Result {
    // let start = Instant::now();
    // let mut rows = Vec::new();
    // let root: &Path = Path::new("C:\\GIT");
    // println!("{:?}", folder_size(root, &mut rows));
    // println!("Execution time: {:?} s", start.elapsed());

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };
    eframe::run_native(
        "Native file dialogs and drag-and-drop files",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

#[derive(Default)]
struct MyApp {
    picked_path: Option<String>,
    is_searching: bool,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                if let Some(picked_path) = &self.picked_path {
                    ui.horizontal(|ui| {
                        ui.label("Picked file:");
                        ui.monospace(picked_path);
                    });
                }

                if ui
                    .add_enabled(!self.is_searching, egui::Button::new("Browse"))
                    .clicked()
                    && let Some(path) = rfd::FileDialog::new().pick_folder()
                {
                    self.picked_path = Some(path.display().to_string());
                }

                if ui
                    .add_enabled(
                        !self.is_searching && self.picked_path.is_some(),
                        egui::Button::new("Calculate Size"),
                    )
                    .clicked()
                {
                    self.is_searching = true;
                    let start = Instant::now();
                    println!("Execution time: {:?} s", start.elapsed());
                }
            })
        });
    }
}
