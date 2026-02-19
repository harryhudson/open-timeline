// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! The edit timeline GUI
//!

use crate::app::ActionRequest;
use crate::components::{
    BooleanExpressionGui, EntityOrTimeline, HintText, NameGui, TagsGui, TimelineEntitiesGui,
    TimelineSubtimelinesGui,
};
use crate::config::SharedConfig;
use crate::consts::DEFAULT_WINDOW_SIZES;
use crate::shortcuts::global_shortcuts;
use crate::windows::{Deleted, DeletedStatus};
use crate::{
    common::*, impl_is_valid_method_for_iterable, impl_valid_asynchronous_macro_never_called,
    spawn_transaction_no_commit_send_result,
};
use eframe::egui::{
    self, CentralPanel, Context, Response, ScrollArea, Spinner, Ui, Vec2, ViewportId,
};
use open_timeline_core::{HasIdAndName, OpenTimelineId, TimelineEdit};
use open_timeline_crud::{CrudError, FetchById};
use open_timeline_gui_core::{
    BreakOutWindow, CheckForUpdates, CreateOrEdit, DisplayStatus, Draw, EmptyConsideredInvalid,
    GuiStatus, Reload, Shortcut, ShowRemoveButton, Valid, ValidSynchronous, ValidityAsynchronous,
    ValiditySynchronous, window_has_focus,
};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::{Receiver, error::TryRecvError};

/// Edit a timeline
#[derive(Debug)]
pub struct TimelineEditGui {
    /// The the timeline being edited as it is in the database (if one is being
    /// edited, rather than created)
    database_entry: Option<TimelineEdit>,

    // TODO: store the whole TimelineEdit
    /// The ID of the timeline being edited, else none if creating a new one.
    timeline_id: Option<OpenTimelineId>,

    /// The name input
    name: NameGui,

    /// The bool expr input
    bool_expr: BooleanExpressionGui,

    /// The entity inputs
    entities: TimelineEntitiesGui,

    /// The subtimeline inputs
    subtimelines: TimelineSubtimelinesGui,

    /// The tag inputs
    tags: TagsGui,

    /// Whether or not the a boolean expression is extant.  When editing a
    /// timeline, for example, it may or may not have an expression.
    has_expr: bool,

    /// Whether the timeline has been deleted or not.  If it has been, the
    /// `Deleted` variant holds the `Instant` it was deleted
    deleted_status: DeletedStatus,

    /// Whether the window is for creating or editing/updating a timeline
    create_or_edit: CreateOrEdit,

    /// The status of the current window
    status: Status,

    /// The CRUD operation requested by the user (e.g. update the database)
    crud_op_requested: Option<CrudOperationRequested>,

    /// Send an action request to the main loop
    tx_action_request: UnboundedSender<ActionRequest>,

    /// Receive CRUD operation updates
    rx_create_update: Option<Receiver<Result<TimelineEdit, CrudError>>>,

    /// Recevie updates on deletion
    rx_delete: Option<Receiver<Result<(), CrudError>>>,

    /// Receive reloaded data
    rx_reload: Option<Receiver<Result<TimelineEdit, CrudError>>>,

    /// Whether or not a reload has been requested
    requested_reload: bool,

    /// Used to indirectly inform the rest of the application that a CRUD
    /// operation has been executed
    tx_crud_operation_executed: UnboundedSender<()>,

    /// Whether this window should be closed or not
    wants_to_be_closed: bool,

    /// Database pool
    shared_config: SharedConfig,
}

// TODO: these are all the same as in entity_edit.rs
/// The current status of the window (status message for the user is derived
/// from this)
#[derive(Debug)]
enum Status {
    WaitingForReload,
    WaitingForInitialLoad,

    NewWindowForCreation,
    NewWindowForEditing,

    CreateError(CrudError),
    UpdateError(CrudError),
    DeleteError(CrudError),

    Created,
    Updated,

    // TODO: display the name of the thing deleted (have to save the valid name,
    // not use the possibly editing one)
    Deleted,

    Invalid(String),

    HasBeenDeletedElseWhere,
}

// TODO: same as in entity_edit.rs
impl DisplayStatus for Status {
    fn status_display(&self, ui: &mut Ui) -> Response {
        let str = match self {
            Self::WaitingForReload => String::from("Waiting for timeline to reload"),
            Self::WaitingForInitialLoad => String::from("Waiting for timeline to load"),

            Self::NewWindowForCreation => String::from("Ready to create an timeline"),
            Self::NewWindowForEditing => String::from("Ready to edit an timeline"),

            Self::CreateError(error) => {
                format!("Error when trying to create timeline: {error}")
            }
            Self::UpdateError(error) => {
                format!("Error when trying to update timeline: {error}")
            }
            Self::DeleteError(error) => {
                format!("Error when trying to delete timeline: {error}")
            }

            Self::Created => String::from("Timeline successfully created"),
            Self::Updated => String::from("Timeline successfully updated"),
            Self::Deleted => String::from("Timeline successfully deleted"),

            Self::Invalid(error) => format!("Timeline is invalid: {error}"),

            Self::HasBeenDeletedElseWhere => String::from("Timeline was deleted elsewhere"),
        };
        ui.add(egui::Label::new(str).truncate())
    }
}

///
///
/// We do not have to worry about the validity of self.bool_expr when the
/// timeline doesn't have a bool expr because self.validity() handles it
impl ValidSynchronous for TimelineEditGui {
    fn is_valid_synchronous(&self) -> bool {
        self.name.is_valid_synchronous()
            && self.bool_expr.is_valid_synchronous()
            && self.subtimelines.is_valid_synchronous()
            && self.entities.is_valid_synchronous()
            && self.tags.is_valid_synchronous()
    }

    fn update_validity_synchronous(&mut self) {
        self.name.update_validity_synchronous();
        self.bool_expr.update_validity_synchronous();
        self.subtimelines.update_validity_synchronous();
        self.entities.update_validity_synchronous();
        self.tags.update_validity_synchronous();
    }

    fn validity_synchronous(&self) -> ValiditySynchronous {
        for validity in [
            self.name.validity_synchronous(),
            self.bool_expr.validity_synchronous(),
            self.subtimelines.validity_synchronous(),
            self.entities.validity_synchronous(),
            self.tags.validity_synchronous(),
        ] {
            match validity {
                ValiditySynchronous::Invalid(error) => return ValiditySynchronous::Invalid(error),
                ValiditySynchronous::Valid => continue,
            }
        }
        ValiditySynchronous::Valid
    }
}

impl_valid_asynchronous_macro_never_called!(TimelineEditGui);

impl Valid for TimelineEditGui {
    fn validity(&self) -> ValidityAsynchronous {
        if self.has_expr {
            impl_is_valid_method_for_iterable!([
                self.name.validity(),
                self.bool_expr.validity(),
                self.entities.validity(),
                self.subtimelines.validity(),
                self.tags.validity(),
            ])
        } else {
            impl_is_valid_method_for_iterable!([
                self.name.validity(),
                self.entities.validity(),
                self.subtimelines.validity(),
                self.tags.validity(),
            ])
        }
    }

    fn update_validity(&mut self) {
        // Do nothing.  Components update their validity themselves.
        panic!()
    }
}

impl TimelineEditGui {
    /// Get whether the window is for editing or creating an timeline
    pub fn create_or_edit(&self) -> CreateOrEdit {
        self.create_or_edit.clone()
    }

    /// Get the ID of the timeline being edited (or none if it's being created)
    pub fn timeline_id(&self) -> Option<OpenTimelineId> {
        self.timeline_id
    }

    /// Create a new `TimelineEditGui` for creating a timeline
    pub fn new_window_for_creating_timeline(
        shared_config: SharedConfig,
        tx_action_request: UnboundedSender<ActionRequest>,
        tx_crud_operation_executed: UnboundedSender<()>,
    ) -> Self {
        TimelineEditGui {
            database_entry: None,
            timeline_id: None,
            name: NameGui::new(Arc::clone(&shared_config), EntityOrTimeline::Timeline),
            bool_expr: BooleanExpressionGui::new(
                ShowRemoveButton::Yes,
                EmptyConsideredInvalid::Yes,
                HintText::None,
            ),
            entities: TimelineEntitiesGui::new(Arc::clone(&shared_config)),
            subtimelines: TimelineSubtimelinesGui::new(Arc::clone(&shared_config)),
            tags: TagsGui::new(),
            has_expr: false,
            deleted_status: DeletedStatus::NotDeleted,
            create_or_edit: CreateOrEdit::Create,
            status: Status::NewWindowForCreation,
            crud_op_requested: None,
            tx_action_request,
            rx_create_update: None,
            rx_delete: None,
            rx_reload: None,
            requested_reload: false,
            tx_crud_operation_executed,
            wants_to_be_closed: false,
            shared_config,
        }
    }

    // TODO: impl From<ReducedTimeline> to?
    /// Create a new `TimelineEditGui` for editing a timeline
    pub fn new_window_for_editing_timeline(
        shared_config: SharedConfig,
        tx_action_request: UnboundedSender<ActionRequest>,
        tx_crud_operation_executed: UnboundedSender<()>,
        timeline_id: OpenTimelineId,
    ) -> Self {
        let mut timeline_edit_gui = TimelineEditGui {
            database_entry: None,
            timeline_id: Some(timeline_id),
            name: NameGui::new(Arc::clone(&shared_config), EntityOrTimeline::Timeline),
            bool_expr: BooleanExpressionGui::new(
                ShowRemoveButton::Yes,
                EmptyConsideredInvalid::Yes,
                HintText::None,
            ),
            entities: TimelineEntitiesGui::new(Arc::clone(&shared_config)),
            subtimelines: TimelineSubtimelinesGui::new(Arc::clone(&shared_config)),
            tags: TagsGui::new(),
            has_expr: false,
            deleted_status: DeletedStatus::NotDeleted,
            create_or_edit: CreateOrEdit::Edit,
            status: Status::NewWindowForEditing,
            crud_op_requested: None,
            tx_action_request,
            rx_create_update: None,
            rx_delete: None,
            rx_reload: None,
            requested_reload: false,
            tx_crud_operation_executed,
            wants_to_be_closed: false,
            shared_config,
        };
        timeline_edit_gui.request_reload();
        timeline_edit_gui
    }

    /// Set the current timeline from the timeline passed in.  This is used when
    /// reloading the data.
    fn set_from_timeline(&mut self, timeline: TimelineEdit) {
        self.database_entry = Some(timeline.clone());
        self.timeline_id = timeline.id();
        self.name = NameGui::from_name(
            Arc::clone(&self.shared_config),
            EntityOrTimeline::Timeline,
            timeline.name().clone(),
        );
        self.bool_expr = timeline.bool_expr().clone().into();
        self.entities = TimelineEntitiesGui::from_reduced_entities(
            Arc::clone(&self.shared_config),
            timeline.entities().clone(),
        );
        self.subtimelines = TimelineSubtimelinesGui::from_reduced_timelines(
            Arc::clone(&self.shared_config),
            timeline.subtimelines().clone(),
        );
        self.has_expr = timeline.bool_expr().is_some();
        self.tags = timeline.tags().clone().into();
        self.deleted_status = DeletedStatus::NotDeleted;
        self.create_or_edit = CreateOrEdit::Edit;
        self.crud_op_requested = None;
        self.rx_create_update = None;
        self.rx_delete = None;
        self.rx_reload = None;
    }

    // TODO: same as in entity_edit
    fn request_create_or_update(&mut self) {
        // Catch those component that haven't been touched and are therefore "valid" but not really
        self.update_validity_synchronous();
        if let ValidityAsynchronous::Valid = self.validity() {
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            self.rx_create_update = Some(rx);
            self.crud_op_requested = Some(CrudOperationRequested::CreateOrUpdate);
            let timeline = self.to_opentimeline_type();
            let create_or_edit = self.create_or_edit.clone();
            let shared_config = Arc::clone(&self.shared_config);
            tokio::spawn(
                async move { save_crud(shared_config, &create_or_edit, timeline, tx).await },
            );
        }
    }

    // TODO: same as in entity_edit
    fn request_delete(&mut self) {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_delete = Some(rx);
        self.crud_op_requested = Some(CrudOperationRequested::Delete);
        let timeline_id = self.timeline_id.unwrap();
        let shared_config = Arc::clone(&self.shared_config);
        tokio::spawn(async move {
            delete_from_id_crud::<TimelineEdit>(shared_config, timeline_id, tx).await
        });
    }

    // TODO: Nearly identical to that in entity.rs (make generic or macro)
    /// Handle create/update/delete response
    fn check_for_crud_status_updates(&mut self) {
        // Response to create/update request
        if let Some(rx) = self.rx_create_update.as_mut() {
            match rx.try_recv() {
                Ok(result) => {
                    debug!("Recv timeline edit create/update request response");
                    self.rx_create_update = None;
                    self.crud_op_requested = None;
                    match result {
                        Ok(timeline) => {
                            self.set_from_timeline(timeline);
                            self.status = match self.create_or_edit {
                                CreateOrEdit::Create => Status::Created,
                                CreateOrEdit::Edit => Status::Updated,
                            };
                            let _ = self.tx_crud_operation_executed.send(());
                        }
                        Err(error) => {
                            self.status = match self.create_or_edit {
                                CreateOrEdit::Create => Status::CreateError(error),
                                CreateOrEdit::Edit => Status::UpdateError(error),
                            };
                        }
                    }
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => (),
            }
        }

        // Response to delete request
        if let Some(rx) = self.rx_delete.as_mut() {
            match rx.try_recv() {
                Ok(result) => {
                    debug!("Recv timeline edit delete request response");
                    self.rx_delete = None;
                    self.crud_op_requested = None;
                    match result {
                        Ok(()) => {
                            self.status = Status::Deleted;
                            self.set_deleted_status(DeletedStatus::Deleted(Instant::now()));
                            let _ = self.tx_crud_operation_executed.send(());
                        }
                        Err(error) => {
                            self.status = Status::DeleteError(error);
                        }
                    }
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => (),
            }
        }
    }

    // TODO: trait
    /// Set the current timeline from the timeline passed in.
    fn reset(&mut self) {
        match &self.database_entry {
            Some(timeline) => self.set_from_timeline(timeline.clone()),
            None => panic!("ERROR: shouldn't ever get here"),
        }
    }

    // TODO: trait
    ///
    fn differs_from_database_entry(&self) -> Option<bool> {
        if self.validity() == ValidityAsynchronous::Valid {
            let current_entity = self.to_opentimeline_type();
            match self.database_entry.as_ref() {
                Some(timeline_in_db) => Some(current_entity != *timeline_in_db),
                None => panic!("Shouldn't get here"),
            }
        } else {
            None
        }
    }

    // TODO: same as in entity_edit.rs
    fn draw_toolbar(&mut self, ui: &mut Ui) {
        //
        ui.horizontal(|ui| match self.create_or_edit {
            CreateOrEdit::Create => {
                if self.validity() == ValidityAsynchronous::Valid {
                    if open_timeline_gui_core::Button::create(ui).clicked() {
                        self.request_create_or_update();
                    }
                } else {
                    ui.label("Input valid information for a new timeline");
                }
            }
            CreateOrEdit::Edit => {
                // Delete comes first so that it never moves (reduced likelihood
                // of accidentally clicking it)
                if open_timeline_gui_core::Button::delete(ui).clicked() {
                    self.request_delete();
                }
                // Can be invalid or valid, but cannot be equal to the entry in the database
                if self.differs_from_database_entry() != Some(false)
                    && open_timeline_gui_core::Button::reset(ui).clicked()
                {
                    self.reset();
                }
                // Must be valid & differ from the database
                if self.differs_from_database_entry() == Some(true)
                    && self.validity() == ValidityAsynchronous::Valid
                    && open_timeline_gui_core::Button::update(ui).clicked()
                {
                    self.request_create_or_update();
                }
            }
        });
    }

    fn draw_status(&mut self, ui: &mut Ui) {
        if self.rx_create_update.is_some() || self.rx_delete.is_some() {
            ui.add(Spinner::new());
        }
        GuiStatus::display(ui, &self.status);
    }
}

impl ToOpenTimelineType<TimelineEdit> for TimelineEditGui {
    fn to_opentimeline_type(&self) -> TimelineEdit {
        let id = self.timeline_id;
        let name = self.name.to_opentimeline_type();
        let bool_expr = if self.has_expr {
            Some(self.bool_expr.to_opentimeline_type())
        } else {
            None
        };
        let entities = self.entities.to_opentimeline_type();
        let subtimelines = self.subtimelines.to_opentimeline_type();
        let tags = self.tags.to_opentimeline_type();

        // TODO: is this to returna result or not?
        TimelineEdit::from(id, name, bool_expr, entities, subtimelines, tags).unwrap()
    }
}

impl Reload for TimelineEditGui {
    fn request_reload(&mut self) {
        if self.has_been_deleted() {
            return;
        }
        match self.timeline_id {
            Some(timeline_id) => {
                self.requested_reload = true;
                let (tx, rx) = tokio::sync::mpsc::channel(1);
                self.rx_reload = Some(rx);
                let shared_config = Arc::clone(&self.shared_config);
                spawn_transaction_no_commit_send_result!(
                    shared_config,
                    bounded,
                    tx,
                    |transaction| async move {
                        TimelineEdit::fetch_by_id(transaction, &timeline_id).await
                    }
                );
            }
            None => self.set_deleted_status(DeletedStatus::Deleted(Instant::now())),
        }
    }

    fn check_reload_response(&mut self) {
        if let Some(rx) = self.rx_reload.as_mut() {
            match rx.try_recv() {
                Ok(result) => {
                    debug!("Recv timeline edit reload response");
                    self.rx_reload = None;
                    self.requested_reload = false;
                    match result {
                        Ok(timeline) => self.set_from_timeline(timeline),
                        Err(CrudError::IdNotInDb) => {
                            self.set_deleted_status(DeletedStatus::Deleted(Instant::now()))
                        }
                        Err(_) => todo!(),
                    }
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => (),
            }
        }
    }
}

impl Deleted for TimelineEditGui {
    fn set_deleted_status(&mut self, deleted_status: DeletedStatus) {
        self.deleted_status = deleted_status;
    }

    fn deleted_status(&self) -> DeletedStatus {
        self.deleted_status
    }
}

impl CheckForUpdates for TimelineEditGui {
    fn check_for_updates(&mut self) {
        self.check_reload_response();
        self.check_for_crud_status_updates();
    }

    fn waiting_for_updates(&mut self) -> bool {
        let waiting =
            self.rx_reload.is_some() || self.rx_create_update.is_some() || self.rx_delete.is_some();
        if waiting {
            info!("TimelineEditGui is waiting for updates");
        }
        waiting
    }
}

impl BreakOutWindow for TimelineEditGui {
    fn draw(&mut self, ctx: &Context) {
        // Handle shortcuts
        if window_has_focus(ctx) {
            if Shortcut::save(ctx) {
                self.request_create_or_update();
            }
            if Shortcut::close_window(ctx) {
                self.wants_to_be_closed = true;
            }
        }

        // Check for global shortcuts
        global_shortcuts(ctx, &mut self.tx_action_request);

        // Update the status
        match self.validity() {
            ValidityAsynchronous::Invalid(error) => self.status = Status::Invalid(error),

            // TODO: this is wrong
            ValidityAsynchronous::Valid => self.status = Status::NewWindowForEditing,
            ValidityAsynchronous::Waiting => (),
        }

        CentralPanel::default().show(ctx, |ui| {
            if self.requested_reload {
                ui.spinner();
                return;
            }

            // TODO: draw the name of the entity? (or when deleted)
            // Window title
            open_timeline_gui_core::Label::heading(ui, "Timeline");
            ui.separator();

            // Has been deleted
            if self.has_been_deleted() {
                self.draw_deleted_message(ctx, ui);

                //
                if let DeletedStatus::Deleted(deleted_at) = self.deleted_status() {
                    let elapsed_secs = deleted_at.elapsed().as_secs() as i32;
                    let remaining_seconds = 5 - elapsed_secs;
                    if remaining_seconds < 1 {
                        self.wants_to_be_closed = true;
                    }
                }

                return;
            }

            // Status
            self.draw_status(ui);
            ui.separator();

            // Create/Update/Delete buttons
            self.draw_toolbar(ui);
            ui.separator();

            // Name
            self.name.draw(ctx, ui);
            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                // Timeline entity boolean expressions
                open_timeline_gui_core::Label::sub_heading(ui, "Entity Boolean Expression");
                if self.has_expr {
                    ui.horizontal(|ui| {
                        self.bool_expr.draw(ctx, ui);

                        // Remove the bool expr if the user has request so
                        if open_timeline_gui_core::Button::remove(ui).clicked() {
                            self.has_expr = false;
                            self.update_validity_synchronous();
                            dbg!(self.validity_synchronous());
                        }
                    });
                } else {
                    // Print that there is not a bool expr if applicable
                    open_timeline_gui_core::Label::none(ui);
                    if open_timeline_gui_core::Button::add(ui).clicked() {
                        self.has_expr = true;
                        self.update_validity_synchronous();
                        dbg!(self.validity_synchronous());
                    }
                }
                ui.separator();

                // Timeline subtimelines
                self.subtimelines.draw(ctx, ui);
                ui.separator();

                // Timeline entities
                self.entities.draw(ctx, ui);
                ui.separator();

                // Timeline tags
                self.tags.draw(ctx, ui);
            });
        });
    }

    fn default_size(&self) -> Vec2 {
        Vec2::new(
            DEFAULT_WINDOW_SIZES.timeline_edit.width,
            DEFAULT_WINDOW_SIZES.timeline_edit.height,
        )
    }

    fn viewport_id(&mut self) -> ViewportId {
        ViewportId(eframe::egui::Id::from({
            match self.create_or_edit() {
                CreateOrEdit::Create => format!("timeline_create_{}", OpenTimelineId::new()),
                CreateOrEdit::Edit => {
                    format!("timeline_edit_{}", self.timeline_id().unwrap())
                }
            }
        }))
    }

    fn title(&mut self) -> String {
        match self.create_or_edit() {
            CreateOrEdit::Create => {
                format!("Create Timeline • {}", self.name.name)
            }
            CreateOrEdit::Edit => {
                format!("Edit Timeline • {}", self.name.name)
            }
        }
    }

    fn wants_to_be_closed(&mut self) -> bool {
        self.wants_to_be_closed
    }
}
