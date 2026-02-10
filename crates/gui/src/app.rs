// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! OpenTimeline egui desktop app
//!

use crate::Config;
use crate::app_colours::{AppColours, ColourTheme};
use crate::config::{RuntimeConfig, SharedConfig};
use crate::games::{
    DecadesGameGui, LeftRightGameGui, OrderEntitiesGameGui, WereTheyAliveWhenGameGui,
    WhichDateGameGui,
};
use crate::primary_window::{
    AppInfoGui, BackupMergeRestoreGui, EntityCountsGui, SearchGui, SettingsGui, StatsGui,
    TagCountsGui, TimelineCountsGui,
};
use crate::shortcuts::global_shortcuts;
use crate::windows::{
    AppColoursGui, BreakOutWindows, EntityEditGui, EntityViewGui, TagBulkEditGui, TagViewGui,
    TimelineEditGui, TimelineViewGui,
};
use bool_tag_expr::Tag;
use eframe::App;
use eframe::egui::{
    self, Align, Button, CentralPanel, Context, Layout, OpenUrl, Pos2, SidePanel, Ui, Vec2,
};
use open_timeline_core::OpenTimelineId;
use open_timeline_crud::db_url_from_path;
use open_timeline_gui_core::{
    BreakOutWindow, Draw, Reload, using_wayland, widget_x_spacing, widget_y_spacing,
};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// Indicates which of the tabs in the main window is selected.
#[derive(Debug, PartialEq, Eq, Clone)]
enum MainTabSelected {
    Search,
    Entities,
    Tags,
    Timelines,
    Stats,
    BackupRestoreMerge,

    GameDecades,
    GameLeftRight,
    GameOrderEntities,
    GameAliveWhen,
    GameWhichDate,

    Settings,
    AppInfo,
}

impl MainTabSelected {
    fn to_label_text(&self) -> String {
        match self {
            Self::Search => String::from("Search"),
            Self::Entities => String::from("Entities"),
            Self::Tags => String::from("Tags"),
            Self::Timelines => String::from("Timelines"),
            Self::Stats => String::from("Stats"),
            Self::BackupRestoreMerge => String::from("Backup | Restore | Merge"),

            Self::GameDecades => String::from("Decades"),
            Self::GameLeftRight => String::from("Left/Right"),
            Self::GameOrderEntities => String::from("Order Entities"),
            Self::GameAliveWhen => String::from("Alive When"),
            Self::GameWhichDate => String::from("Which Date"),

            Self::Settings => String::from("Settings"),
            Self::AppInfo => String::from("Information"),
        }
    }
}

/// All possible action requests
///
/// e.g. "edit entity X", "view timeline Y", "bulk edit tag Z"
#[derive(Debug)]
pub enum ActionRequest {
    Entity(EntityOrTimelineActionRequest),
    Timeline(EntityOrTimelineActionRequest),
    Tag(TagActionRequest),

    // TODO: shouldn't send a channel, I think
    AppColours(UnboundedSender<AppColours>),
}

/// All possible action requests for entities and timelines
#[derive(Debug)]
pub enum EntityOrTimelineActionRequest {
    CreateNew,
    ViewExisting(OpenTimelineId),
    EditExisting(OpenTimelineId),
}

/// All possible action requests for tags
#[derive(Debug)]
pub enum TagActionRequest {
    ViewExisting(Tag),
    BulkEditExisting(Tag),
}

// TODO: impl a new()?
/// Holds both the `tx` and `rx` ends of an unbounded channel.
#[derive(Debug)]
pub struct UnboundedChannel<T> {
    pub tx: UnboundedSender<T>,
    pub rx: UnboundedReceiver<T>,
}

impl<T> From<(UnboundedSender<T>, UnboundedReceiver<T>)> for UnboundedChannel<T> {
    fn from(value: (UnboundedSender<T>, UnboundedReceiver<T>)) -> Self {
        UnboundedChannel {
            tx: value.0,
            rx: value.1,
        }
    }
}

/// All data needed for the OpenTimeline (egui) desktop app
pub struct OpenTimelineApp {
    /// The position of the main window (if it's open)
    position: Option<Pos2>,

    /// Which of the sidebar tabs in the main window is selected
    tab_selected: MainTabSelected,

    /// All pop-out windows
    windows: BreakOutWindows,

    /// The search panel of the main window
    search_gui: SearchGui,

    /// The entity count panel of the main window
    entity_counts_gui: EntityCountsGui,

    // TODO: update to show both timeline and entity tags
    /// The tags count panel of the main window
    entity_tag_counts_gui: TagCountsGui,

    /// The timeline count panel of the main window
    timeline_counts_gui: TimelineCountsGui,

    /// The stats panel of the main window
    stats_gui: StatsGui,

    /// The backup|merge|restore panel of the main window
    backup_merge_restore_gui: BackupMergeRestoreGui,

    /// The settings panel of the main window
    settings_gui: SettingsGui,

    /// The app info panel of the main window
    app_info_gui: AppInfoGui,

    /// Unbounded channel for requesting actions on entites, timelines, and
    /// tags.  e.g. a request to edit an entity.
    channel_action_request: UnboundedChannel<ActionRequest>,

    /// Unbounded channel used for letting the main app know when a CUD
    /// operation (read operation not important) has happened successfully.
    /// This lets the main loop request app-wide reloads of data so that it
    /// reflects the change(s).
    channel_crud_operation_executed: UnboundedChannel<()>,

    /// Tracks whether a global reload is required (i.e. if a message has been
    /// received on `channel_crud_operation_executed`)
    reload_required: bool,

    /// The "decades" game panel of the main window
    game_decades: DecadesGameGui,

    /// The "left right" game panel of the main window
    game_left_right: LeftRightGameGui,

    /// The "order entities" game panel of the main window
    game_order_entities: OrderEntitiesGameGui,

    /// The "were they alive when" game panel of the main window
    game_were_they_alive_when: WereTheyAliveWhenGameGui,

    /// The "which_date" game panel of the main window
    game_which_date: WhichDateGameGui,

    /// Database pool
    shared_config: SharedConfig,
}

impl OpenTimelineApp {
    /// Create a new `OpenTimelineApp`
    pub fn new() -> Self {
        let channel_action_request: UnboundedChannel<ActionRequest> =
            tokio::sync::mpsc::unbounded_channel().into();
        let channel_crud_operation_executed: UnboundedChannel<()> =
            tokio::sync::mpsc::unbounded_channel().into();

        // Config
        let (tx, rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let result = async move {
                Config::ensure_setup().await?;
                Config::load()
            }
            .await;
            let _ = tx.send(result);
        });
        // TODO: remove unwrap()
        let config = match rx.blocking_recv().unwrap() {
            Ok(config) => config,
            Err(error) => panic!("Initial config error: {error}"),
        };

        // Path to database
        let db_path = Arc::new(RwLock::new(config.database_path()));

        // Database pool
        let (tx, rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let result: Result<Pool<Sqlite>, sqlx::Error> = async move {
                let db_path = db_path.read().await;
                let db_url = db_url_from_path(&db_path);
                let db_pool = SqlitePool::connect(&db_url).await?;
                Ok(db_pool)
            }
            .await;
            let _ = tx.send(result);
        });
        let db_pool = match rx.blocking_recv().unwrap() {
            Ok(db_pool) => db_pool,
            Err(error) => panic!("Initial SQLite pool error: {error}"),
        };
        let shared_config = Arc::new(RwLock::new(RuntimeConfig {
            db_pool: db_pool,
            config: config.clone(),
        }));

        Self {
            position: None,
            tab_selected: MainTabSelected::Search,
            windows: BreakOutWindows::default(),
            search_gui: SearchGui::new(
                Arc::clone(&shared_config),
                channel_action_request.tx.clone(),
            ),
            entity_counts_gui: EntityCountsGui::new(
                Arc::clone(&shared_config),
                channel_action_request.tx.clone(),
            ),
            entity_tag_counts_gui: TagCountsGui::new(
                Arc::clone(&shared_config),
                channel_action_request.tx.clone(),
            ),
            timeline_counts_gui: TimelineCountsGui::new(
                Arc::clone(&shared_config),
                channel_action_request.tx.clone(),
            ),
            stats_gui: StatsGui::new(Arc::clone(&shared_config)),
            backup_merge_restore_gui: BackupMergeRestoreGui::new(
                Arc::clone(&shared_config),
                channel_crud_operation_executed.tx.clone(),
            ),
            settings_gui: SettingsGui::new(
                config,
                Arc::clone(&shared_config),
                channel_action_request.tx.clone(),
                channel_crud_operation_executed.tx.clone(),
            ),
            app_info_gui: AppInfoGui::new(),
            channel_action_request,
            channel_crud_operation_executed,
            reload_required: false,
            game_decades: DecadesGameGui::new(Arc::clone(&shared_config)),
            game_left_right: LeftRightGameGui::new(Arc::clone(&shared_config)),
            game_order_entities: OrderEntitiesGameGui::new(Arc::clone(&shared_config)),
            game_were_they_alive_when: WereTheyAliveWhenGameGui::new(Arc::clone(&shared_config)),
            game_which_date: WhichDateGameGui::new(Arc::clone(&shared_config)),
            shared_config,
        }
    }

    fn draw_side_bar_option(
        &mut self,
        _ctx: &Context,
        ui: &mut Ui,
        tab_variant: MainTabSelected,
        separator_after: bool,
    ) {
        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            let tab = Button::selectable(
                self.tab_selected == tab_variant,
                tab_variant.to_label_text(),
            );
            let tab = ui.add(tab);
            if tab.clicked() {
                self.tab_selected = tab_variant;
            }
            if separator_after {
                ui.separator();
            }
        });
    }

    fn draw_side_panel(&mut self, ctx: &Context, ui: &mut Ui) {
        let space = widget_y_spacing(ui);
        ui.add_space(space * 2.0);
        open_timeline_gui_core::Label::heading(ui, "OpenTimeline");
        ui.separator();

        // Donate button
        ui.scope(|ui| {
            // Get the colours
            let (button_fill, button_text) =
                match self.shared_config.blocking_read().config.colour_theme {
                    ColourTheme::Custom(app_colours) => {
                        let fill = app_colours.donate_button_fill.into();
                        let text = app_colours.donate_button_text.into();
                        (fill, text)
                    }
                    _ => {
                        let fill = AppColours::default_donate_button_fill();
                        let text = AppColours::default_donate_button_text_colour();
                        (fill, text)
                    }
                };

            // Set colours
            let style = ui.style_mut();
            style.visuals.widgets.active.weak_bg_fill = button_fill;
            style.visuals.widgets.inactive.weak_bg_fill = button_fill;
            style.visuals.widgets.hovered.weak_bg_fill = button_fill;
            style.visuals.override_text_color = Some(button_text);

            // Draw button
            let size = Vec2::new(ui.available_width(), 0.0);
            let button = egui::Button::new("Donate");
            if ui.add_sized(size, button).clicked() {
                ctx.open_url(OpenUrl {
                    url: "https://www.open-timeline.org/donate".to_owned(),
                    new_tab: true,
                });
            }
        });
        ui.separator();

        self.draw_side_bar_option(ctx, ui, MainTabSelected::Search, true);
        self.draw_side_bar_option(ctx, ui, MainTabSelected::Entities, true);
        self.draw_side_bar_option(ctx, ui, MainTabSelected::Tags, true);
        self.draw_side_bar_option(ctx, ui, MainTabSelected::Timelines, true);
        self.draw_side_bar_option(ctx, ui, MainTabSelected::Stats, true);
        self.draw_side_bar_option(ctx, ui, MainTabSelected::BackupRestoreMerge, true);
        ui.horizontal(|ui| {
            let space = widget_x_spacing(ui) / 2.0;
            ui.add_space(space);
            ui.label("Games");
        });

        ui.indent("id_salt", |ui| {
            self.draw_side_bar_option(ctx, ui, MainTabSelected::GameDecades, false);
            self.draw_side_bar_option(ctx, ui, MainTabSelected::GameLeftRight, false);
            self.draw_side_bar_option(ctx, ui, MainTabSelected::GameOrderEntities, false);
            self.draw_side_bar_option(ctx, ui, MainTabSelected::GameAliveWhen, false);
            self.draw_side_bar_option(ctx, ui, MainTabSelected::GameWhichDate, false);
        });
        ui.separator();

        self.draw_side_bar_option(ctx, ui, MainTabSelected::Settings, false);
        self.draw_side_bar_option(ctx, ui, MainTabSelected::AppInfo, false);
    }

    fn draw_central_panel(&mut self, ctx: &Context, ui: &mut Ui) {
        open_timeline_gui_core::Label::heading(ui, &self.tab_selected.to_label_text());
        ui.separator();

        match self.tab_selected {
            MainTabSelected::Search => {
                self.windows.draw(ctx, ui);
                self.search_gui.draw(ctx, ui);
            }
            MainTabSelected::Entities => {
                self.windows.draw(ctx, ui);
                self.entity_counts_gui.draw(ctx, ui);
            }
            MainTabSelected::Tags => {
                self.windows.draw(ctx, ui);
                self.entity_tag_counts_gui.draw(ctx, ui);
            }
            MainTabSelected::Timelines => {
                self.windows.draw(ctx, ui);
                self.timeline_counts_gui.draw(ctx, ui);
            }
            MainTabSelected::Stats => {
                self.windows.draw(ctx, ui);
                self.stats_gui.draw(ctx, ui);
            }
            MainTabSelected::BackupRestoreMerge => {
                self.backup_merge_restore_gui.draw(ctx, ui);
            }

            MainTabSelected::GameDecades => self.game_decades.draw(ctx, ui),
            MainTabSelected::GameLeftRight => self.game_left_right.draw(ctx, ui),
            MainTabSelected::GameOrderEntities => self.game_order_entities.draw(ctx, ui),
            MainTabSelected::GameAliveWhen => self.game_were_they_alive_when.draw(ctx, ui),
            MainTabSelected::GameWhichDate => self.game_which_date.draw(ctx, ui),

            MainTabSelected::Settings => {
                self.windows.draw(ctx, ui);
                self.settings_gui.draw(ctx, ui);
            }
            MainTabSelected::AppInfo => {
                self.windows.draw(ctx, ui);
                self.app_info_gui.draw(ctx, ui);
            }
        }
    }

    // TODO: improve the error handling
    // TODO: rename (receives, and opens)
    /// Receive requests and any associated OpenTimelineIds (e.g. open a new window
    /// for the creation of a new entity, or open a new window for the viewing
    /// of the timeline associated with the given ID).
    fn create_any_new_windows(&mut self, ctx: &Context) {
        let db = Arc::clone(&self.shared_config);
        let tx_crud = self.channel_crud_operation_executed.tx.clone();
        let tx_req = self.channel_action_request.tx.clone();
        if let Ok(msg) = self.channel_action_request.rx.try_recv() {
            let window: Box<dyn BreakOutWindow> = match msg {
                // Entity windows
                ActionRequest::Entity(action) => match action {
                    EntityOrTimelineActionRequest::CreateNew => Box::new(
                        EntityEditGui::new_window_for_creating_entity(db, tx_req, tx_crud),
                    ),
                    EntityOrTimelineActionRequest::EditExisting(id) => Box::new(
                        EntityEditGui::new_window_for_editing_entity(db, tx_req, tx_crud, id),
                    ),
                    EntityOrTimelineActionRequest::ViewExisting(id) => {
                        Box::new(EntityViewGui::new(db, tx_req, id))
                    }
                },
                // Timeline windows
                ActionRequest::Timeline(action) => match action {
                    EntityOrTimelineActionRequest::CreateNew => Box::new(
                        TimelineEditGui::new_window_for_creating_timeline(db, tx_req, tx_crud),
                    ),
                    EntityOrTimelineActionRequest::EditExisting(id) => Box::new(
                        TimelineEditGui::new_window_for_editing_timeline(db, tx_req, tx_crud, id),
                    ),
                    EntityOrTimelineActionRequest::ViewExisting(id) => {
                        Box::new(TimelineViewGui::new(db, ctx, tx_req, id))
                    }
                },
                // Tag windows
                ActionRequest::Tag(action) => match action {
                    TagActionRequest::BulkEditExisting(tag) => {
                        Box::new(TagBulkEditGui::new(db, tx_req, tx_crud, tag))
                    }
                    TagActionRequest::ViewExisting(tag) => {
                        Box::new(TagViewGui::new(db, tx_req, tag))
                    }
                },
                // Colour windows
                ActionRequest::AppColours(tx_app_colours) => {
                    debug!("recv ActionRequest::AppColours");
                    // TODO: don't want to block
                    let config = self.shared_config.blocking_read().config.clone();
                    Box::new(AppColoursGui::new(config, tx_req, tx_app_colours))
                }
            };
            self.windows.insert(ctx, self.position, window);
        }
    }
}

impl App for OpenTimelineApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(300));

        // TODO: don't need to do every frame, only if changed
        // Update the colour theme
        self.settings_gui.check_for_app_colours_update();
        AppColours::use_theme(ctx, self.settings_gui.theme());

        // Get window position if we can (can't if using Wayland)
        self.position = match using_wayland() {
            false => ctx.input(|i| i.viewport().outer_rect).map(|rect| rect.min),
            true => None,
        };

        // Check if there have been any CRUD operations and thus if a reload is in order
        if let Ok(()) = self.channel_crud_operation_executed.rx.try_recv() {
            self.reload_required = true;
            self.windows.request_reload();
            self.search_gui.request_reload();
            self.entity_counts_gui.request_reload();
            self.entity_tag_counts_gui.request_reload();
            self.timeline_counts_gui.request_reload();
            self.entity_tag_counts_gui.request_reload();
            self.stats_gui.request_reload();
        }

        // Check for global shortcuts
        global_shortcuts(ctx, &mut self.channel_action_request.tx);

        // Open any new windows that need to be opened
        self.create_any_new_windows(ctx);

        // Draw the side panel
        SidePanel::left("sidebar").show(ctx, |ui| {
            self.draw_side_panel(ctx, ui);
        });

        // Draw the main central panel
        CentralPanel::default().show(ctx, |ui| {
            self.draw_central_panel(ctx, ui);
        });

        // The reload is requested in a single frame
        self.reload_required = false;
    }
}
