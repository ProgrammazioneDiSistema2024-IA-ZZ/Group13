use egui::{Context};
use std::future::Future;
use std::sync::mpsc::{channel, Receiver, Sender};
use crate::config::Config;

pub struct BackupApp {
    source_channel: (Sender<String>, Receiver<String>),
    destination_channel: (Sender<String>, Receiver<String>),
    source: String,
    destination: String,
}

impl Default for BackupApp {
    fn default() -> Self {
        let mut app = Self {
            source_channel: channel(),
            destination_channel: channel(),
            source: String::from("No folder selected"),
            destination: String::from("No folder selected"),
        };

        app.load_config();
        app
    }
}

impl BackupApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        BackupApp::default()
    }

    fn save_config(&self) {
        let config = Config::new(self.source.clone(), self.destination.clone());
        config.save();
    }

    fn load_config(&mut self) {
        let mut config = Config::default();
        config.load();
        self.source = config.source;
        self.destination = config.destination;
    }
}

impl eframe::App for BackupApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        
        if let Ok(source) = self.source_channel.1.try_recv() {
            self.source = source;
            self.save_config();
        }

        if let Ok(destination) = self.destination_channel.1.try_recv() {
            self.destination = destination;
            self.save_config();
        }


        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome to the Back-up App!");

            ui.add_space(20.0);

            ui.label(format!("Source: {}", self.source.clone()));
            if ui.button("📂 Select folder to backup").clicked() {
                let sender = self.source_channel.0.clone();
                let task = rfd::AsyncFileDialog::new().pick_folder();
                let ctx = ui.ctx().clone();
                execute(async move {
                    let folder = task.await;
                    if let Some(folder) = folder {
                        let path = folder.path();
                        println!("Selected folder: {:?}", path);
                        sender.send(path.to_string_lossy().to_string()).unwrap();

                        ctx.request_repaint();
                    }
                });
            }

            ui.add_space(10.0);
            ui.label(format!("Destination: {}", self.destination.clone()));
            if ui.button("💾 Select folder to save backup").clicked() {
                let sender = self.destination_channel.0.clone();
                let task = rfd::AsyncFileDialog::new().pick_folder();
                let ctx = ui.ctx().clone();
                execute(async move {
                    let folder = task.await;
                    if let Some(folder) = folder {
                        let path = folder.path();
                        println!("Selected folder: {:?}", path);
                        sender.send(path.to_string_lossy().to_string()).unwrap();
                        ctx.request_repaint();
                    }
                });
            }

            ui.add_space(20.0);

            if ui.button("🚪 Close App").clicked() {
                self.save_config();
                std::process::exit(0);
            }
        });
    }
}

fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || futures::executor::block_on(f));
}