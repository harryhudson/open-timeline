// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Desktop GUI settings
//!

use crate::app::{ActionRequest, UnboundedChannel};
use crate::app_colours::{AppColours, ColourTheme};
use crate::config::{Config, SharedConfig};
use eframe::egui::{self, Context, Grid, Response, RichText, ScrollArea, Spinner, Ui};
use log::{error, info};
use open_timeline_crud::{CrudError, db_url_from_path};
use open_timeline_gui_core::{CheckForUpdates, Draw};
use open_timeline_gui_core::{DisplayStatus, GuiStatus};
use sqlx::SqlitePool;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver, Sender, UnboundedSender};

/// The settings GUI panel in the main window
#[derive(Debug)]
pub struct SettingsGui {
    /// App config
    config: Config,

    /// Current status
    status: Status,

    /// Runtime live config
    shared_config: SharedConfig,

    // TODO: the save functionality should be in the breakout window as it is
    // with entities & timelines & bulk tags
    /// Whether or not to show the button for saving custom colours
    show_save_colours_button: bool,

    /// Used to indirectly inform the rest of the application to reload
    /// everything as a result of a new database selection
    tx_crud_operation_executed: UnboundedSender<()>,

    /// Send an action request to the main loop
    tx_action_request: UnboundedSender<ActionRequest>,

    /// Channel for `AppColours` transmission
    channel_app_colours: UnboundedChannel<AppColours>,

    /// Receive updates about database selection saving
    rx_database_config_update: Option<Receiver<Result<(), CrudError>>>,

    /// Receive updates about theme selection saving
    rx_theme_update: Option<Receiver<Result<(), CrudError>>>,

    /// Receive updates about theme selection saving
    rx_switch_database_update: Option<Receiver<Result<(), CrudError>>>,
}

/// The possible states of operation for the window
#[derive(Debug, Clone, PartialEq, Eq)]
enum Status {
    Ready,
    WaitingForResponse,
    SuccessfullyChangedDatabase,
    DatabaseHasDifferentSchema,
    SuccessfullyChangedTheme,
    CrudError(CrudError),
}

impl DisplayStatus for Status {
    fn status_display(&self, ui: &mut Ui) -> Response {
        match &self {
            Self::Ready => ui.add(egui::Label::new(String::from("Ready")).truncate()),
            Self::WaitingForResponse => ui.add(Spinner::new()),
            Self::SuccessfullyChangedDatabase => {
                ui.add(egui::Label::new(String::from("Successfully switched database")).truncate())
            }
            Self::DatabaseHasDifferentSchema => ui.add(
                egui::Label::new(String::from(
                    "Error: selected database has incompatible schema",
                ))
                .truncate(),
            ),
            Self::SuccessfullyChangedTheme => {
                ui.add(egui::Label::new(String::from("Successfully switched theme")).truncate())
            }
            Self::CrudError(error) => {
                ui.add(egui::Label::new(format!("Error: {error}")).truncate())
            }
        }
    }
}

impl SettingsGui {
    /// Create a new settings GUI panel manager
    pub fn new(
        config: Config,
        shared_config: SharedConfig,
        tx_action_request: UnboundedSender<ActionRequest>,
        tx_crud_operation_executed: UnboundedSender<()>,
    ) -> Self {
        debug!("New SettingsGui. config = {config:?}");
        Self {
            config,
            status: Status::Ready,
            shared_config,
            show_save_colours_button: false,
            tx_crud_operation_executed,
            tx_action_request,
            channel_app_colours: tokio::sync::mpsc::unbounded_channel().into(),
            rx_database_config_update: None,
            rx_theme_update: None,
            rx_switch_database_update: None,
        }
    }

    /// Get the app theme
    pub fn theme(&self) -> ColourTheme {
        self.config.colour_theme
    }

    /// Draw everything related to controlling the application's database
    /// settings
    fn draw_database_settings(&mut self, _ctx: &Context, ui: &mut Ui) {
        // Sub heading
        open_timeline_gui_core::Label::sub_heading(ui, "Database File");

        // Path of database file in use
        let database_path = self.config.database_path().to_string_lossy().to_string();
        let monospace_size = ui.style().text_styles[&egui::TextStyle::Monospace].size;
        let size = monospace_size * 0.9;
        let text = RichText::new(&database_path).monospace().size(size);
        ui.label(text);
        ui.add_space(5.0);

        // Buttons for database selection
        let width = ui.available_width() / 3.0;
        Grid::new("database_file_buttons")
            .min_col_width(width)
            .max_col_width(width)
            .num_columns(3)
            .show(ui, |ui| {
                self.select_existing_database(ui);
                self.select_new_database(ui);
                self.use_default_database(ui);
            });
        ui.add_space(10.0);
    }

    /// Draw everything related to controlling the application's colours
    fn draw_app_colour_settings(&mut self, _ctx: &Context, ui: &mut Ui) {
        open_timeline_gui_core::Label::sub_heading(ui, "Colour Theme");
        let mut theme_changed = false;

        // Simple (system/light/dark)
        ui.horizontal(|ui| {
            theme_changed |= ui
                .radio_value(&mut self.config.colour_theme, ColourTheme::System, "System")
                .changed();
            theme_changed |= ui
                .radio_value(&mut self.config.colour_theme, ColourTheme::Light, "Light")
                .changed();
            theme_changed |= ui
                .radio_value(&mut self.config.colour_theme, ColourTheme::Dark, "Dark")
                .changed();
        });

        // Built in OpenTimeline themes
        ui.horizontal(|ui| {
            theme_changed |= ui
                .radio_value(
                    &mut self.config.colour_theme,
                    ColourTheme::Siphonophore,
                    "Siphonophore",
                )
                .changed();
        });

        // Custom theme
        ui.horizontal(|ui| {
            let app_colours = match self.config.colour_theme {
                // Use the current custom theme
                ColourTheme::Custom(app_colours) => app_colours,

                // Use the custom theme saved but not in use
                _ => self.config.custom_theme,
            };
            theme_changed |= ui
                .radio_value(
                    &mut self.config.colour_theme,
                    ColourTheme::Custom(app_colours),
                    "Custom",
                )
                .changed();
        });

        // Show save button
        let mut user_requested_save_custom_colours = false;
        if self.show_save_colours_button {
            if open_timeline_gui_core::Button::tall_full_width(ui, "Save Custom Theme").clicked() {
                user_requested_save_custom_colours = true;
            }
        }

        // Update the app theme if applicable
        if theme_changed || user_requested_save_custom_colours {
            // Setup the channel for receiving updates
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            self.rx_theme_update = Some(rx);

            // Make sure the custom theme that's saved is the up-to-date one
            if let ColourTheme::Custom(custom_colours) = self.config.colour_theme {
                self.config.custom_theme = custom_colours
            }

            // Update shared state
            self.switch_shared_colour_theme();

            // Request save config to disk
            self.request_save(tx);
        }

        // Draw request custom editor window
        if let ColourTheme::Custom(_) = self.config.colour_theme {
            if open_timeline_gui_core::Button::tall_full_width(ui, "Edit Custom Colours").clicked()
            {
                debug!("Requesting new window for custom app colour selection");
                let _ = self.tx_action_request.send(ActionRequest::AppColours(
                    self.channel_app_colours.tx.clone(),
                ));
            }
        };
    }

    fn select_existing_database(&mut self, ui: &mut Ui) {
        if open_timeline_gui_core::Button::tall_full_width(ui, "Use Existing").clicked() {
            if let Some(db_path) = rfd::FileDialog::new().pick_file() {
                println!("Selected file: {}", db_path.display());
                self.config.set_database_path(&db_path);
                let (tx, rx) = tokio::sync::mpsc::channel(1);
                self.rx_database_config_update = Some(rx);
                self.request_save(tx);
            }
        }
    }

    fn select_new_database(&mut self, ui: &mut Ui) {
        if open_timeline_gui_core::Button::tall_full_width(ui, "Create & Use New").clicked() {
            if let Some(db_path) = rfd::FileDialog::new().save_file() {
                self.config.set_database_path(&db_path);
                let (tx, rx) = tokio::sync::mpsc::channel(1);
                self.rx_database_config_update = Some(rx);
                self.request_save(tx);
            }
        }
    }

    fn use_default_database(&mut self, ui: &mut Ui) {
        if open_timeline_gui_core::Button::tall_full_width(ui, "Use Default").clicked() {
            self.config.set_to_default();
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            self.rx_database_config_update = Some(rx);
            self.request_save(tx);
        }
    }

    /// Attempt to save the config to disk
    fn request_save(&mut self, tx: Sender<Result<(), CrudError>>) {
        self.status = Status::WaitingForResponse;
        let config = self.config.clone();
        tokio::spawn(async move {
            let result = config.save().await;
            let _ = tx.send(result).await;
        });
    }

    /// Attempt to switch the application's database pool to the new database
    fn request_switch_database_pools(&mut self) {
        let shared_config = self.shared_config.clone();
        let db_path = self.config.database_path();
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_switch_database_update = Some(rx);
        tokio::spawn(async move {
            let result = async move {
                let mut shared_config = shared_config.write().await;
                let db_url = db_url_from_path(&db_path);
                (*shared_config).db_pool = SqlitePool::connect(&db_url).await?;
                Ok(())
            }
            .await;
            let _ = tx.send(result).await;
        });
    }

    /// Switch the application's colour theme
    fn switch_shared_colour_theme(&mut self) {
        let shared_config = self.shared_config.clone();
        let config = self.config.clone();
        tokio::spawn(async move {
            let mut shared_config = shared_config.write().await;
            (*shared_config).config = config;
            debug!("Updated shared config = {shared_config:?}");
        });
    }

    /// Check for result of saving new database selection to disk
    fn check_for_database_selection_update(&mut self) {
        if let Some(rx) = self.rx_database_config_update.as_mut() {
            match rx.try_recv() {
                Ok(result) => {
                    self.rx_database_config_update = None;
                    match result {
                        Ok(()) => self.request_switch_database_pools(),
                        Err(CrudError::DbMigrate(error)) => {
                            self.status = Status::DatabaseHasDifferentSchema;
                            error!(
                                "Error - database is likely for a different application: {error}"
                            )
                        }
                        Err(error) => {
                            self.status = Status::CrudError(error.clone());
                            error!("Error: {error}");
                        }
                    }
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => (),
            }
        }
    }

    /// Check for result of saving new app theme choice to disk
    fn check_for_theme_selection_update(&mut self) {
        if let Some(rx) = self.rx_theme_update.as_mut() {
            match rx.try_recv() {
                Ok(result) => {
                    self.rx_theme_update = None;
                    self.show_save_colours_button = false;
                    match result {
                        Ok(()) => self.status = Status::SuccessfullyChangedTheme,
                        Err(error) => {
                            self.status = Status::CrudError(error.clone());
                            error!("Error: {error}");
                        }
                    }
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => (),
            }
        }
    }

    // TODO: how does this interact with the config saved to file status messages?
    /// Check if the result (if any) of the database pool switch over
    fn check_for_database_pool_switch_update(&mut self) {
        if let Some(rx) = self.rx_switch_database_update.as_mut() {
            match rx.try_recv() {
                Ok(result) => {
                    self.rx_switch_database_update = None;
                    match result {
                        Ok(()) => {
                            self.status = Status::SuccessfullyChangedDatabase;
                            info!("Database pool switched");
                            info!("Requesting search refresh");
                            let _ = self.tx_crud_operation_executed.send(());
                        }
                        Err(error) => {
                            self.status = Status::CrudError(error.clone());
                            error!("Error: {error}");
                        }
                    }
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => (),
            }
        }
    }

    // TODO: how does this interact with the config saved to file status messages?
    ///
    pub fn check_for_app_colours_update(&mut self) {
        // TODO: Option<channel>?
        // if let Some(rx) = self.channel_app_colours.as_mut() {
        match self.channel_app_colours.rx.try_recv() {
            Ok(app_colours) => {
                debug!("Received app colours");
                self.config.colour_theme = ColourTheme::Custom(app_colours);
                self.show_save_colours_button = true;
                self.switch_shared_colour_theme();
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => (),
        }
    }
    // }
}

impl Draw for SettingsGui {
    fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        // Draw status
        GuiStatus::display(ui, &self.status);
        ui.separator();

        ui.add_enabled_ui(self.status != Status::WaitingForResponse, |ui| {
            self.draw_database_settings(ctx, ui);
            self.draw_app_colour_settings(ctx, ui);
        });
    }
}

impl CheckForUpdates for SettingsGui {
    fn check_for_updates(&mut self) {
        self.check_for_database_selection_update();
        self.check_for_theme_selection_update();
        self.check_for_database_pool_switch_update();
        self.check_for_app_colours_update();
    }

    fn waiting_for_updates(&mut self) -> bool {
        let waiting = self.rx_database_config_update.is_some()
            || self.rx_switch_database_update.is_some()
            || self.rx_theme_update.is_some();
        if waiting {
            info!("SettingsGui is waiting for updates");
        }
        waiting
    }
}
