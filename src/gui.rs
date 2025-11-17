#![allow(dead_code)]
use crate::dnf::*;
use crate::utils::*;

use eframe::{App, CreationContext, Frame, HardwareAcceleration, NativeOptions, run_native};
use egui::{
    Align, Button, CentralPanel, Color32, Context, Direction, FontFamily, Label, Layout, RichText,
    ScrollArea, TextWrapMode, Ui, Vec2, ViewportBuilder,
    text::{LayoutJob, TextFormat, TextWrapping},
};

use rpm::Package;

use std::{
    error::Error,
    path::PathBuf,
    sync::{Arc, Mutex, mpsc::Receiver},
    thread::{self, JoinHandle},
};

#[derive(PartialEq, Eq, Debug)]
enum AppStep {
    Intro,
    Process,
    Finished,
}

struct Application {
    pkg_path: PathBuf,
    pkg: Package,
    step: AppStep,
    process_log: String,
    pkg_state: Option<PackageState>,
    pkg_state_shared: Arc<Mutex<Option<PackageState>>>,
    process_rx: Option<Receiver<String>>,
    pkg_state_loading_thread: Option<JoinHandle<()>>,
    process_thread: Option<JoinHandle<()>>,
}

impl Application {
    fn new(_cc: &CreationContext, pkg_path: PathBuf) -> Self {
        let pkg = Package::open(&pkg_path).expect("Failed to read rpm package =(");

        Self {
            pkg_path,
            pkg,
            step: AppStep::Intro,
            process_log: String::new(),
            pkg_state: None,
            pkg_state_shared: Arc::new(Mutex::new(None)),
            process_rx: None,
            pkg_state_loading_thread: None,
            process_thread: None,
        }
    }

    fn get_package_state(&mut self) -> JoinHandle<()> {
        let pkg_state_shared = self.pkg_state_shared.clone();
        let pkg = self.pkg.clone();
        let thread = thread::spawn(move || {
            let pkg_state = get_package_state(&pkg);
            let mut guard = pkg_state_shared.lock().unwrap();
            *guard = Some(pkg_state);
        });
        return thread;
    }

    fn start_process(&mut self) {
        self.step = AppStep::Process;
        let (process_thread, process_rx) = match self.pkg_state.as_ref().unwrap() {
            PackageState::NewPackage => {
                dnf_start_action(self.pkg_path.to_str().unwrap(), DNFAction::Install)
            }
            PackageState::OldVersion => dnf_start_action(
                self.pkg.metadata.get_name().unwrap_or_default(),
                DNFAction::Remove,
            ),
            PackageState::NewVersion(_) => {
                dnf_start_action(self.pkg_path.to_str().unwrap(), DNFAction::Upgrade)
            }
        };
        self.process_thread = Some(process_thread);
        self.process_rx = Some(process_rx);
    }

    fn draw_intro(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            fn add_info_entry(
                ui: &mut Ui,
                key: &str,
                value: &str,
                max_rows: usize,
                hyperlink: bool,
            ) {
                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                    let key_label = Label::new(
                        RichText::new(format!("• {}\t→\t", key))
                            .color(Color32::LIGHT_GRAY)
                            .family(FontFamily::Monospace),
                    );
                    ui.add(key_label);

                    if hyperlink {
                        ui.hyperlink(value);
                    } else {
                        let mut value_layout_job = LayoutJob {
                            wrap: TextWrapping {
                                max_rows: max_rows,
                                ..Default::default()
                            },
                            ..Default::default()
                        };
                        value_layout_job.append(value, 0.0, TextFormat::default());
                        ui.add(Label::new(value_layout_job).wrap_mode(TextWrapMode::Wrap));
                    }
                });
            }
            ScrollArea::vertical()
                .max_height(ui.available_height() - 30.0)
                .show(ui, |ui| {
                    ui.take_available_width();
                    add_info_entry(
                        ui,
                        "Name        ",
                        self.pkg.metadata.get_name().unwrap_or("-"),
                        1,
                        false,
                    );
                    match self.pkg_state.as_ref().unwrap() {
                        PackageState::NewVersion(old_pkg) => {
                            add_info_entry(
                                ui,
                                "Old Version ",
                                &format!("{}-{}", old_pkg.version, old_pkg.release),
                                1,
                                false,
                            );
                            add_info_entry(
                                ui,
                                "New Version ",
                                &format!(
                                    "{}-{}",
                                    self.pkg.metadata.get_version().unwrap_or("-"),
                                    self.pkg.metadata.get_release().unwrap_or("-")
                                ),
                                1,
                                false,
                            );
                        }
                        _ => {
                            add_info_entry(
                                ui,
                                "Version     ",
                                &format!(
                                    "{}-{}",
                                    self.pkg.metadata.get_version().unwrap_or("-"),
                                    self.pkg.metadata.get_release().unwrap_or("-")
                                ),
                                1,
                                false,
                            );
                        }
                    }

                    add_info_entry(
                        ui,
                        "Architecture",
                        &self.pkg.metadata.get_arch().unwrap_or("-"),
                        1,
                        false,
                    );
                    add_info_entry(
                        ui,
                        "Size        ",
                        &size_to_string(self.pkg.metadata.get_installed_size().unwrap_or(0) as f64),
                        1,
                        false,
                    );
                    add_info_entry(
                        ui,
                        "Summary     ",
                        &self.pkg.metadata.get_summary().unwrap_or("-"),
                        3,
                        false,
                    );
                    add_info_entry(
                        ui,
                        "URL         ",
                        &self.pkg.metadata.get_url().unwrap_or("-"),
                        1,
                        true,
                    );
                    add_info_entry(
                        ui,
                        "License     ",
                        &self.pkg.metadata.get_license().unwrap_or("-"),
                        2,
                        false,
                    );
                    add_info_entry(
                        ui,
                        "Description ",
                        &self.pkg.metadata.get_description().unwrap_or("-"),
                        5,
                        false,
                    );
                });
        });

        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            ui.with_layout(Layout::right_to_left(Align::BOTTOM), |ui| {
                if ui
                    .button(
                        RichText::new(match self.pkg_state.as_ref().unwrap() {
                            PackageState::NewPackage => "Install",
                            PackageState::OldVersion => "Remove",
                            PackageState::NewVersion(_) => "Upgrade",
                        })
                        .size(18.0)
                        .family(FontFamily::Monospace),
                    )
                    .clicked()
                {
                    self.start_process();
                };
                if ui
                    .button(
                        RichText::new("Cancel")
                            .size(18.0)
                            .family(FontFamily::Monospace),
                    )
                    .clicked()
                {
                    std::process::exit(0);
                };
            });

            ui.separator();
        });
    }

    fn draw_process(&mut self, ui: &mut Ui) {
        ScrollArea::vertical()
            .stick_to_bottom(true)
            .max_height(ui.available_height() - 30.0)
            .show(ui, |ui| {
                ui.take_available_width();
                ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                    ui.add(Label::new(&self.process_log).wrap_mode(TextWrapMode::Wrap));
                });
            });

        if self.step == AppStep::Finished {
            ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                ui.with_layout(Layout::right_to_left(Align::BOTTOM), |ui| {
                    let close_button = Button::new(
                        RichText::new("Close")
                            .size(18.0)
                            .family(FontFamily::Monospace),
                    );
                    let back_button = Button::new(
                        RichText::new("Back")
                            .size(18.0)
                            .family(FontFamily::Monospace),
                    );

                    if ui.add(close_button).clicked() {
                        std::process::exit(0);
                    }
                    if ui.add(back_button).clicked() {
                        self.step = AppStep::Intro;
                        self.process_log = String::new();
                        self.pkg_state = None;
                    }
                });

                ui.separator();
            });
        }

        ui.ctx().request_repaint();
    }
}

impl App for Application {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        if let Some(process_thread) = &self.process_thread
            && let Some(process_rx) = &self.process_rx
        {
            if process_thread.is_finished() {
                if let Ok(msg) = process_rx.try_recv() {
                    self.process_log.push_str(&msg);
                    self.process_log.push('\n');
                }
                self.process_thread = None;
                self.process_rx = None;
                self.step = AppStep::Finished;
            } else if let Ok(msg) = process_rx.try_recv() {
                self.process_log.push_str(&msg);
                self.process_log.push('\n');
            }
        }

        CentralPanel::default().show(ctx, |ui| {
            if self.pkg_state.is_none() {
                if self.pkg_state_loading_thread.is_none() {
                    self.pkg_state_loading_thread = Some(self.get_package_state());
                } else if let Some(t) = &self.pkg_state_loading_thread
                    && t.is_finished()
                {
                    let mut guard = self.pkg_state_shared.lock().unwrap();
                    self.pkg_state = guard.clone();
                    *guard = None;
                    self.pkg_state_loading_thread = None;
                }
                ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                    ui.spinner()
                });
            } else {
                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    ui.add(Label::new(
                        RichText::new(format!(
                            "{} {}-{}-{}.{}.rpm",
                            match self.pkg_state.as_ref().unwrap() {
                                PackageState::NewPackage => "Install",
                                PackageState::OldVersion => "Remove",
                                PackageState::NewVersion(_) => "Upgrade",
                            },
                            self.pkg.metadata.get_name().unwrap_or("unknown"),
                            self.pkg.metadata.get_version().unwrap_or("0.0.0"),
                            self.pkg.metadata.get_release().unwrap_or("1"),
                            self.pkg.metadata.get_arch().unwrap_or("unknown"),
                        ))
                        .size(14.0)
                        .color(Color32::LIGHT_GRAY)
                        .family(FontFamily::Monospace),
                    ));
                    ui.separator();

                    match self.step {
                        AppStep::Intro => self.draw_intro(ui),
                        _ => self.draw_process(ui),
                    }
                });
            }
        });
    }
}

pub fn run(arg: PathBuf) -> Result<(), Box<dyn Error>> {
    let opts = NativeOptions {
        vsync: true,
        centered: true,
        hardware_acceleration: HardwareAcceleration::Preferred,

        viewport: ViewportBuilder::default()
            .with_app_id("ru.arabianq.rpmi")
            .with_resizable(false)
            .with_inner_size(Vec2::new(600.0, 300.0)),

        ..Default::default()
    };

    match run_native(
        "RPM Installer",
        opts,
        Box::new(|cc| Ok(Box::new(Application::new(cc, arg)))),
    ) {
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    }
}
