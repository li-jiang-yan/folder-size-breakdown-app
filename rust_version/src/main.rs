use eframe::egui;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::thread;
use std::time::Instant;

struct FolderSize {
    path: PathBuf,
    bytes: u64,
}

fn folder_size(path: &Path, tx: &std::sync::mpsc::Sender<WorkerMessage>) -> io::Result<u64> {
    let mut total = 0;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child_path = entry.path();
        let file_type = entry.file_type()?;
        let metadata = fs::metadata(&child_path)?;

        if file_type.is_symlink() {
            continue;
        }

        if metadata.is_dir() {
            // Recursively calculate this subfolder first.
            total += folder_size(&child_path, tx)?;
        } else if metadata.is_file() {
            total += metadata.len();
        }
    }

    // Record this folder after its contents have been counted.
    let _ = tx.send(WorkerMessage::Folder(FolderSize {
        path: path.to_path_buf(),
        bytes: total,
    }));

    Ok(total)
}

fn main() -> eframe::Result {
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

enum WorkerMessage {
    Folder(FolderSize),
    Finished(Result<u64, std::io::Error>),
}

#[derive(Default)]
struct MyApp {
    picked_path: Option<PathBuf>,
    receiver: Option<Receiver<WorkerMessage>>,
    rows: Vec<FolderSize>,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let messages: Vec<WorkerMessage> = self
            .receiver
            .as_ref()
            .map(|receiver| receiver.try_iter().collect())
            .unwrap_or_default();

        let mut received_folder = false;
        let mut finished = false;

        for message in messages {
            match message {
                WorkerMessage::Folder(row) => {
                    self.rows.push(row);
                    received_folder = true;
                }

                WorkerMessage::Finished(Ok(_total)) => {
                    finished = true;
                }

                WorkerMessage::Finished(Err(_error)) => {
                    finished = true;
                }
            }
        }

        if received_folder {
            self.rows.sort_by_key(|row| std::cmp::Reverse(row.bytes));
        }

        if finished {
            self.receiver = None;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            if self.receiver.is_some() {
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(50));
            }

            let is_running = self.receiver.is_some();

            ui.horizontal(|ui: &mut egui::Ui| {
                if let Some(picked_path) = &self.picked_path {
                    ui.horizontal(|ui| {
                        ui.label("Picked folder:");
                        ui.monospace(picked_path.display().to_string());
                    });
                }

                if ui
                    .add_enabled(!is_running, egui::Button::new("Browse"))
                    .clicked()
                    && let Some(path) = rfd::FileDialog::new().pick_folder()
                {
                    self.picked_path = Some(path);
                }

                if ui
                    .add_enabled(
                        !is_running && self.picked_path.is_some(),
                        egui::Button::new("Calculate Size"),
                    )
                    .clicked()
                    && let Some(picked_path) = &self.picked_path
                {
                    self.rows.clear();
                    let picked_path = picked_path.clone();
                    let ctx = ui.ctx().clone();
                    let (tx, rx) = channel();
                    self.receiver = Some(rx);

                    thread::spawn(move || {
                        let root = &picked_path;
                        let start = Instant::now();
                        let res = folder_size(root, &tx);
                        println!("Execution time: {:?}", start.elapsed());

                        let _ = tx.send(WorkerMessage::Finished(res));
                        ctx.request_repaint();
                    });
                }
            });

            egui::Grid::new("folder_sizes")
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Folder");
                    ui.strong("Size (GB)");
                    ui.end_row();

                    for row in &self.rows {
                        ui.label(row.path.display().to_string());
                        ui.label(format!("{:.3}", row.bytes as f64 / 1_000_000_000.0));
                        ui.end_row();
                    }
                });
        });
    }
}
