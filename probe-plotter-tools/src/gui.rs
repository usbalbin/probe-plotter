//! This example shows how to wrap the Rerun Viewer in your own GUI.

use std::sync::mpsc;

use rerun::external::{eframe, egui, re_memory, re_viewer};

use crate::setting::Setting;

// By using `re_memory::AccountingAllocator` Rerun can keep track of exactly how much memory it is using,
// and prune the data store when it goes above a certain limit.
// By using `mimalloc` we get faster allocations.
#[global_allocator]
static GLOBAL: re_memory::AccountingAllocator<mimalloc::MiMalloc> =
    re_memory::AccountingAllocator::new(mimalloc::MiMalloc);

pub struct MyApp {
    rerun_app: re_viewer::App,
    settings: Vec<Setting>,

    /// Send settigns here to apply them
    settings_channel: mpsc::Sender<Setting>,
}

impl MyApp {
    pub fn new(
        rerun_app: re_viewer::App,
        settings: Vec<Setting>,
        settings_channel: mpsc::Sender<Setting>,
    ) -> Self {
        Self {
            rerun_app,
            settings,
            settings_channel,
        }
    }
}

impl eframe::App for MyApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Store viewer state on disk
        self.rerun_app.save(storage);
    }

    /// Called whenever we need repainting, which could be 60 Hz.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // First add our panel(s):
        egui::SidePanel::right("my_side_panel")
            .default_width(200.0)
            .show(ctx, |ui| {
                self.ui(ui);
            });

        // Now show the Rerun Viewer in the remaining space:
        self.rerun_app.update(ctx, frame);
    }
}

impl MyApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.vertical_centered(|ui| {
            ui.strong("Settings");
        });
        ui.separator();

        for setting in &mut self.settings {
            if ui
                .add(
                    egui::Slider::new(&mut setting.value, setting.range.clone())
                        .step_by(setting.step_size)
                        .text(&setting.name),
                )
                .changed()
            {
                self.settings_channel.send(setting.clone()).unwrap();
            }
        }
    }
}
