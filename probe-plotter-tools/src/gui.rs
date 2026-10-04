//! This example shows how to wrap the Rerun Viewer in your own GUI.

use std::{
    collections::HashMap, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc},
};

use probe_rs::flashing::{ProgressEvent, ProgressOperation};
use rerun::external::{
    eframe,
    egui::{self, ProgressBar},
    re_memory, re_viewer,
};

use crate::setting::Setting;

// By using `re_memory::AccountingAllocator` Rerun can keep track of exactly how much memory it is using,
// and prune the data store when it goes above a certain limit.
// By using `mimalloc` we get faster allocations.
#[global_allocator]
static GLOBAL: re_memory::AccountingAllocator<mimalloc::MiMalloc> =
    re_memory::AccountingAllocator::new(mimalloc::MiMalloc);

pub struct MyApp {
    rerun_app: re_viewer::App,
    settings: Arc<Mutex<Vec<Setting>>>,

    /// Send settigns here to apply them
    settings_channel: mpsc::Sender<Setting>,
    flash_progress: Arc<Mutex<FlashProgress>>,
    is_time_to_flash: Arc<AtomicBool>,
}

impl MyApp {
    pub fn new(
        rerun_app: re_viewer::App,
        settings: Arc<Mutex<Vec<Setting>>>,
        settings_channel: mpsc::Sender<Setting>,
        flash_progress: Arc<Mutex<FlashProgress>>,
        is_time_to_flash: Arc<AtomicBool>
    ) -> Self {
        Self {
            rerun_app,
            settings,
            settings_channel,
            flash_progress,is_time_to_flash
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

        for setting in &mut *self.settings.lock().unwrap() {
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

        if !self.is_time_to_flash.load(Ordering::SeqCst) {
            if ui.button("Flash").clicked() {
                self.is_time_to_flash.store(true, Ordering::SeqCst);
            }
        }

        let flash_progress = self.flash_progress.lock().unwrap();

        for op in [
            FlashOperation::Fill,
            FlashOperation::Erase,
            FlashOperation::Program,
            FlashOperation::Verify,
        ] {
            if let Some(x) = flash_progress.ops.get(&op)
                && x.started
            {
                ui.label(format!("{op:?}: "));
                ui.add(ProgressBar::new(
                    x.total.map(|t| x.progress as f32 / t as f32).unwrap_or(0.0),
                ));
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlashOperation {
    /// Reading back flash contents to restore erased regions that should be kept unchanged.
    Fill,

    /// Erasing flash sectors.
    Erase,

    /// Writing data to flash.
    Program,

    /// Checking flash contents.
    Verify,

    /// Writing data directly to RAM.
    Ram,
}

impl From<probe_rs::flashing::ProgressOperation> for FlashOperation {
    fn from(value: probe_rs::flashing::ProgressOperation) -> Self {
        match value {
            ProgressOperation::Fill => FlashOperation::Fill,
            ProgressOperation::Erase => FlashOperation::Erase,
            ProgressOperation::Program => FlashOperation::Program,
            ProgressOperation::Verify => FlashOperation::Verify,
            ProgressOperation::Ram => FlashOperation::Ram,
        }
    }
}

pub struct FlashOperationProgress {
    total: Option<u64>,
    progress: u64,
    started: bool,
}

impl FlashOperationProgress {
    fn new(total: Option<u64>) -> Self {
        Self {
            total,
            progress: 0,
            started: false,
        }
    }
}

pub struct FlashProgress {
    ops: HashMap<FlashOperation, FlashOperationProgress>,
    err: Option<String>,
}

impl FlashProgress {
    pub fn new() -> Self {
        Self {
            ops: HashMap::new(),
            err: None,
        }
    }

    pub fn reset(&mut self) {
        self.ops.clear();
        self.err = None;
    }

    pub fn update(&mut self, evnt: ProgressEvent) {
        match evnt {
            ProgressEvent::FlashLayoutReady { .. } => (),

            ProgressEvent::AddProgressBar { operation, total } => {
                let _ = self
                    .ops
                    .insert(operation.into(), FlashOperationProgress::new(total));
            }
            ProgressEvent::Started(operation) => {
                self.ops.get_mut(&operation.into()).unwrap().started = true
            }
            ProgressEvent::Progress {
                operation,
                size,
                time: _,
            } => {
                self.ops.get_mut(&operation.into()).unwrap().progress = size;
            }

            ProgressEvent::Failed(e) => {
                self.err = Some(format!("Flash failed: {e:?}"));
            }
            ProgressEvent::Finished(operation) => {
                let op = self.ops.get_mut(&operation.into()).unwrap();
                op.progress = op.total.unwrap_or(1);
            }
            ProgressEvent::DiagnosticMessage { .. } => (),
        }
    }
}
