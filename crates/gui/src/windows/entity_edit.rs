// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! The edit entity GUI
//!

use crate::app::ActionRequest;
use crate::common::{CrudOperationRequested, ToOpenTimelineType, delete_from_id_crud, save_crud};
use crate::components::{DatesGui, EntityOrTimeline, NameGui, TagsGui};
use crate::config::SharedConfig;
use crate::consts::DEFAULT_WINDOW_SIZES;
use crate::shortcuts::global_shortcuts;
use crate::windows::{Deleted, DeletedStatus};
use crate::{
    impl_is_valid_method_for_iterable, impl_valid_asynchronous_macro_never_called,
    impl_valid_synchronous_macro_never_called, spawn_transaction_no_commit_send_result,
};
use eframe::egui::{
    self, CentralPanel, Context, Response, ScrollArea, Spinner, Ui, Vec2, ViewportId,
};
use log::info;
use open_timeline_core::{Entity, HasIdAndName, OpenTimelineId};
use open_timeline_crud::{CrudError, FetchById};
use open_timeline_gui_core::{
    BreakOutWindow, CreateOrEdit, DisplayStatus, Draw, GuiStatus, Reload, Shortcut, Valid,
    ValidityAsynchronous, window_has_focus,
};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver, UnboundedSender};

/// Edit an entity
#[derive(Debug)]
pub struct EntityEditGui {
    /// The the entity being edited as it is in the database (if one is being
    /// edited, rather than created)
    database_entry: Option<Entity>,

    /// The ID of the entity being edited (if one is being edited, rather than
    /// created)
    entity_id: Option<OpenTimelineId>,

    /// The GUI name element
    name: NameGui,

    /// The GUI dates element
    dates: DatesGui,

    /// The GUI tags element
    tags: TagsGui,

    /// Whether the entity has been deleted or not.  If it has been, the
    /// `Deleted` variant holds the `Instant` it was deleted
    deleted_status: DeletedStatus,

    /// Whether or not a reload has been requested
    requested_reload: bool,

    /// Whether the window is for creating or editing/updating an entity
    create_or_edit: CreateOrEdit,

    /// The status of the current window
    status: Status,

    /// The CRUD operation requested by the user (e.g. update the database)
    crud_op_requested: Option<CrudOperationRequested>,

    /// Recevie updates on create & update CRUD operations
    rx_create_update: Option<Receiver<Result<Entity, CrudError>>>,

    /// Recevie updates on deletion
    rx_delete: Option<Receiver<Result<(), CrudError>>>,

    /// Receive reloaded data
    rx_reload: Option<Receiver<Result<Entity, CrudError>>>,

    /// Used to indirectly inform the rest of the application that a CRUD
    /// operation has been executed
    tx_crud_operation_executed: UnboundedSender<()>,

    /// Send an action request to the main loop
    tx_action_request: UnboundedSender<ActionRequest>,

    /// Whether this window should be closed or not
    wants_to_be_closed: bool,

    /// Database pool
    shared_config: SharedConfig,

    // TODO: clean up these (perhaps make a new logging helper struct that makes
    // is easy to log when things change)
    /// Used to track changes to whether the entity can be saved
    can_be_saved: bool,
    /// Used to track changes to whether the entity differs from the database
    differs_from_database: Option<bool>,
}

// TODO: these are all the same as in timeline_edit.rs
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

// TODO: same as in timeline_edit.rs
impl DisplayStatus for Status {
    fn status_display(&self, ui: &mut Ui) -> Response {
        let str = match &self {
            Self::WaitingForReload => String::from("Waiting for entity to reload"),
            Self::WaitingForInitialLoad => String::from("Waiting for entity to load"),

            Self::NewWindowForCreation => String::from("Ready to create an entity"),
            Self::NewWindowForEditing => String::from("Ready to edit an entity"),

            Self::CreateError(error) => {
                format!("Error when trying to create entity: {error}")
            }
            Self::UpdateError(error) => {
                format!("Error when trying to update entity: {error}")
            }
            Self::DeleteError(error) => {
                format!("Error when trying to delete entity: {error}")
            }

            Self::Created => String::from("Entity successfully created"),
            Self::Updated => String::from("Entity successfully updated"),
            Self::Deleted => String::from("Entity successfully deleted"),

            Self::Invalid(error) => format!("Entity is invalid: {error}"),

            Self::HasBeenDeletedElseWhere => String::from("Entity was deleted elsewhere"),
        };
        ui.add(egui::Label::new(str).truncate())
    }
}

impl EntityEditGui {
    /// Get whether the window is for editing or creating an entity
    pub fn create_or_edit(&self) -> CreateOrEdit {
        self.create_or_edit.clone()
    }

    // TODO: perfect for type stating
    /// Get the ID of the entity being edited (or none if it's being created)
    pub fn entity_id(&self) -> Option<OpenTimelineId> {
        self.entity_id
    }

    /// Create a new `EntityEditGui` for creating an entity
    pub fn new_window_for_creating_entity(
        shared_config: SharedConfig,
        tx_action_request: UnboundedSender<ActionRequest>,
        tx_crud_operation_executed: UnboundedSender<()>,
    ) -> Self {
        EntityEditGui {
            database_entry: None,
            entity_id: None,
            name: NameGui::new(Arc::clone(&shared_config), EntityOrTimeline::Entity),
            dates: DatesGui::new(),
            tags: TagsGui::new(),
            deleted_status: DeletedStatus::NotDeleted,
            requested_reload: false,
            create_or_edit: CreateOrEdit::Create,
            status: Status::NewWindowForCreation,
            crud_op_requested: None,
            rx_create_update: None,
            rx_delete: None,
            rx_reload: None,
            tx_crud_operation_executed,
            tx_action_request,
            wants_to_be_closed: false,
            shared_config,

            can_be_saved: false,
            differs_from_database: Some(false),
        }
    }

    /// Create a new `EntityEditGui` for editing an entity
    pub fn new_window_for_editing_entity(
        shared_config: SharedConfig,
        tx_action_request: UnboundedSender<ActionRequest>,
        tx_crud_operation_executed: UnboundedSender<()>,
        entity_id: OpenTimelineId,
    ) -> Self {
        let mut entity_edit_gui = EntityEditGui {
            database_entry: None,
            entity_id: Some(entity_id),
            name: NameGui::new(Arc::clone(&shared_config), EntityOrTimeline::Entity),
            dates: DatesGui::new(),
            tags: TagsGui::new(),
            deleted_status: DeletedStatus::NotDeleted,
            requested_reload: false,
            create_or_edit: CreateOrEdit::Edit,
            status: Status::NewWindowForEditing,
            crud_op_requested: None,
            rx_create_update: None,
            rx_delete: None,
            rx_reload: None,
            tx_crud_operation_executed,
            tx_action_request,
            wants_to_be_closed: false,
            shared_config,

            can_be_saved: false,
            differs_from_database: Some(false),
        };
        entity_edit_gui.request_reload();
        entity_edit_gui
    }

    /// Set the current entity from the entity passed in.  This is used when
    /// reloading the data.
    fn set_from_entity(&mut self, entity: Entity) {
        self.database_entry = Some(entity.clone());
        self.entity_id = entity.id();
        self.name = NameGui::from_name(
            Arc::clone(&self.shared_config),
            EntityOrTimeline::Entity,
            entity.name().clone(),
        );
        self.dates = (entity.start(), entity.end()).into();
        self.tags = entity.tags().to_owned().into();
        self.deleted_status = DeletedStatus::NotDeleted;
        self.create_or_edit = CreateOrEdit::Edit;
        self.crud_op_requested = None;
        self.rx_create_update = None;
        self.rx_delete = None;
        self.rx_reload = None;
    }

    // TODO: trait?
    // TODO: just reload?
    /// Set the current entity from the entity passed in.
    fn reset(&mut self) {
        match &self.database_entry {
            Some(entity) => self.set_from_entity(entity.clone()),
            None => panic!("ERROR: shouldn't ever get here"),
        }
    }

    // TODO: use an enum instead of Option<bool>
    // TODO: trait?
    // TODO: identical to one in timeline_edit.rs
    /// Whether the entity differs from the one in the database
    fn differs_from_database_entry(&mut self) -> Option<bool> {
        // If the entity is being created, then nothing in the database to check
        if self.create_or_edit() == CreateOrEdit::Create {
            return None;
        }

        // Note the current value
        let stored_value = self.differs_from_database;

        // Work out whether the entity as it currently is differs from the
        // entity in the database
        let differs = if self.validity() == ValidityAsynchronous::Valid {
            let current_entity = self.to_opentimeline_type();
            match self.database_entry.as_ref() {
                Some(entity_in_db) => Some(current_entity != *entity_in_db),
                None => panic!("Shouldn't get here"),
            }
        } else {
            None
        };

        // If the 2 values aren't the same
        if stored_value != differs {
            // Log the change (reduces logging output)
            debug!(
                "Entity (current name input value = {}) differs from database: {stored_value:?} -> {differs:?}",
                self.name.name
            );

            // Update the held value
            self.differs_from_database = differs;
        }

        // Return the current value
        self.differs_from_database
    }

    /// Draw the status
    fn draw_status(&mut self, ui: &mut Ui) {
        if self.rx_create_update.is_some() || self.rx_delete.is_some() {
            ui.add(Spinner::new());
        }
        GuiStatus::display(ui, &self.status);
    }

    // TODO: same as in entity_edit.rs
    /// Draw the toolbar and its buttons
    fn draw_toolbar(&mut self, ui: &mut Ui) {
        //
        ui.horizontal(|ui| match self.create_or_edit {
            CreateOrEdit::Create => {
                if self.can_be_saved() {
                    if open_timeline_gui_core::Button::create(ui).clicked() {
                        self.request_create_or_update();
                    }
                } else {
                    ui.label("Input valid information for a new entity");
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
                // If the entity can be saved, show the button
                if self.can_be_saved() {
                    if open_timeline_gui_core::Button::update(ui).clicked() {
                        debug!("Entity save button clicked");
                        self.request_create_or_update();
                    }
                }
            }
        });
    }

    /// Must be valid & differ from the database
    fn can_be_saved(&mut self) -> bool {
        // Note the current value
        let stored_value = self.can_be_saved;

        // Work out whether the entity can be saved
        let can_be_saved = self.differs_from_database_entry() != Some(false)
            && self.validity() == ValidityAsynchronous::Valid;

        // If the 2 values aren't the same
        if stored_value != can_be_saved {
            // Log the change (reduces logging output)
            debug!(
                "Entity (current name input value = {}) can be saved: {stored_value} -> {can_be_saved}",
                self.name.name
            );

            // Update the held value
            self.can_be_saved = can_be_saved;
        }

        // Return the current value
        self.can_be_saved
    }

    // TODO: nearly same as timeline_edit
    fn request_create_or_update(&mut self) {
        if self.can_be_saved() {
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            self.rx_create_update = Some(rx);
            self.crud_op_requested = Some(CrudOperationRequested::CreateOrUpdate);
            let entity = self.to_opentimeline_type();
            let edit_or_create = self.create_or_edit.clone();
            let shared_config = Arc::clone(&self.shared_config);
            tokio::spawn(
                async move { save_crud(shared_config, &edit_or_create, entity, tx).await },
            );
        }
    }

    // TODO: can probs be a generic with timeline
    fn request_delete(&mut self) {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_delete = Some(rx);
        self.crud_op_requested = Some(CrudOperationRequested::Delete);
        let entity_id = self.entity_id.unwrap();
        let shared_config = Arc::clone(&self.shared_config);
        tokio::spawn(
            async move { delete_from_id_crud::<Entity>(shared_config, entity_id, tx).await },
        );
    }

    // Nearly identical to that in timeline_edit.rs (make generic or macro)
    fn receive_any_crud_status_updates(&mut self) {
        // Response to create/update request
        if let Some(rx) = self.rx_create_update.as_mut() {
            match rx.try_recv() {
                Ok(result) => {
                    debug!("recv crud update");
                    match result {
                        Ok(entity) => {
                            info!("Entity updated sucessfully");
                            self.set_from_entity(entity);
                            self.status = match self.create_or_edit {
                                CreateOrEdit::Create => Status::Created,
                                CreateOrEdit::Edit => Status::Updated,
                            };
                            let _ = self.tx_crud_operation_executed.send(());
                        }
                        Err(error) => {
                            self.rx_create_update = None;
                            self.crud_op_requested = None;
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
                Ok(result) => match result {
                    Ok(()) => {
                        self.rx_delete = None;
                        self.crud_op_requested = None;
                        self.status = Status::Deleted;
                        self.set_deleted_status(DeletedStatus::Deleted(Instant::now()));
                        let _ = self.tx_crud_operation_executed.send(());
                    }
                    Err(error) => {
                        self.rx_delete = None;
                        self.crud_op_requested = None;
                        self.status = Status::DeleteError(error);
                    }
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => (),
            }
        }
    }
}

impl ToOpenTimelineType<Entity> for EntityEditGui {
    fn to_opentimeline_type(&self) -> Entity {
        let id = self.entity_id;
        let name = self.name.to_opentimeline_type();
        let (start, end) = self.dates.to_opentimeline_type();
        let tags = self.tags.to_opentimeline_type();

        Entity::from(id, name, start, end, tags).unwrap()
    }
}

impl Reload for EntityEditGui {
    fn request_reload(&mut self) {
        if self.has_been_deleted() {
            return;
        }
        match self.entity_id {
            Some(entity_id) => {
                // self.status = Status::WaitingForReload;
                self.requested_reload = true;
                let (tx, rx) = tokio::sync::mpsc::channel(1);
                self.rx_reload = Some(rx);
                let shared_config = Arc::clone(&self.shared_config);
                spawn_transaction_no_commit_send_result!(
                    shared_config,
                    bounded,
                    tx,
                    |transaction| async move { Entity::fetch_by_id(transaction, &entity_id).await }
                );
            }
            None => self.set_deleted_status(DeletedStatus::Deleted(Instant::now())),
        }
    }

    fn check_reload_response(&mut self) {
        if let Some(rx) = self.rx_reload.as_mut() {
            match rx.try_recv() {
                Ok(result) => {
                    debug!("reload response receved");
                    self.rx_reload = None;
                    self.requested_reload = false;
                    match result {
                        Ok(entity) => self.set_from_entity(entity),
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

impl Deleted for EntityEditGui {
    fn set_deleted_status(&mut self, deleted_status: DeletedStatus) {
        self.deleted_status = deleted_status;
    }

    fn deleted_status(&self) -> DeletedStatus {
        self.deleted_status
    }
}

impl_valid_synchronous_macro_never_called!(EntityEditGui);
impl_valid_asynchronous_macro_never_called!(EntityEditGui);

impl Valid for EntityEditGui {
    fn validity(&self) -> ValidityAsynchronous {
        impl_is_valid_method_for_iterable!([
            self.name.validity(),
            self.dates.validity(),
            self.tags.validity(),
        ])
    }

    fn update_validity(&mut self) {
        // Do nothing.  Components update their validity themselves.
        panic!()
    }
}

impl BreakOutWindow for EntityEditGui {
    fn draw(&mut self, ctx: &Context) {
        // Handle shortcuts
        if window_has_focus(ctx) {
            if self.can_be_saved() && Shortcut::save(ctx) {
                self.request_create_or_update();
            }
            if Shortcut::close_window(ctx) {
                self.wants_to_be_closed = true;
            }
        }

        // Check for global shortcuts
        global_shortcuts(ctx, &mut self.tx_action_request);

        // Check responses
        self.check_reload_response();
        self.receive_any_crud_status_updates();

        // Update status (TODO: needed or done elsewhere?)
        match self.validity() {
            ValidityAsynchronous::Invalid(error) => self.status = Status::Invalid(error),
            ValidityAsynchronous::Valid => (),
            ValidityAsynchronous::Waiting => (),
        }

        CentralPanel::default().show(ctx, |ui| {
            if self.requested_reload {
                ui.spinner();
                return;
            }

            // TODO: draw the name of the entity? (or when deleted)
            // Window title
            open_timeline_gui_core::Label::heading(ui, "Entity");
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

            // Dates
            self.dates.draw(ctx, ui);
            ui.separator();

            // Tags
            ScrollArea::vertical().show(ui, |ui| {
                self.tags.draw(ctx, ui);
            });
        });
    }

    fn default_size(&self) -> Vec2 {
        Vec2::new(
            DEFAULT_WINDOW_SIZES.entity_edit.width,
            DEFAULT_WINDOW_SIZES.entity_edit.height,
        )
    }

    fn viewport_id(&mut self) -> ViewportId {
        ViewportId(eframe::egui::Id::from(match self.create_or_edit() {
            CreateOrEdit::Create => format!("entity_create_{}", OpenTimelineId::new()),
            CreateOrEdit::Edit => {
                format!("entity_edit_{}", self.entity_id().unwrap())
            }
        }))
    }

    // TODO: add "unsaved"?
    fn title(&mut self) -> String {
        match self.create_or_edit() {
            CreateOrEdit::Create => {
                format!("Create Entity • {}", self.name.name)
            }
            CreateOrEdit::Edit => {
                format!("Edit Entity • {}", self.name.name)
            }
        }
    }

    fn wants_to_be_closed(&mut self) -> bool {
        self.wants_to_be_closed
    }
}
