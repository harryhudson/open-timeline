// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Validity
//!

use std::{
    any::type_name,
    fmt::{Debug, Display},
};
use tokio::sync::mpsc::Receiver;

/// Whether an empty input is to be regarded as invalid
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyConsideredInvalid {
    Yes,
    No,
}

/// Used to indicate the asynchronous validity of something (i.e. the aspects of
/// validity that need to be done asynchronously, such as checking against the
/// database)
#[derive(Debug, PartialEq, Eq)]
pub enum ValidityAsynchronous {
    Valid,
    Invalid(String),
    Waiting,
}

/// Used to indicate the synchronous validity of something (i.e. the aspects of
/// validity that can done synchronously, such as checking it's structure)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValiditySynchronous {
    Valid,
    Invalid(String),
}

impl ValiditySynchronous {
    pub fn invalid_msg(&self) -> String {
        match &self {
            ValiditySynchronous::Invalid(invalid_msg) => invalid_msg.clone(),
            ValiditySynchronous::Valid => panic!(),
        }
    }
}

/// Holds everything needed for calculating & holding validity information
#[derive(Debug)]
pub struct ValitityStatus<T, E> {
    /// Whether the thing being validated is valid or not, without regard for
    /// any async validation required.
    ///
    /// For example, a `Name` might be synchronously valid because it is not
    /// empty, but may be asynchronously invalid when trying to create a new
    /// entity because it is already in use as the name of an entity in the
    /// database.
    synchronous: ValiditySynchronous,

    /// A channel which if extant receives asynchronous validation updates.
    pub rx_asynchronous: Option<Receiver<Result<T, E>>>,

    /// Whether the thing being validated has been asynchronoulsy validated,
    /// and if so whether or not is it valid or not.
    ///
    /// For example, a timeline's subtimeline may be a valid `Name`
    /// (synchronous validation), but we must also ensure that it exists in the
    /// database as the name of a timeline.
    pub asynchronous: Option<Result<(), E>>,
}

impl<T, E> ValitityStatus<T, E> {
    /// Create a new `ValitityStatus`
    pub fn from(synchronous: ValiditySynchronous, asynchronous: Option<Result<(), E>>) -> Self {
        Self {
            synchronous,
            rx_asynchronous: None,
            asynchronous,
        }
    }

    /// Set the value of the synchronous validity
    pub fn set_synchronous(&mut self, validity: ValiditySynchronous) {
        // If the new value isn't the same as the current value, log the change
        if self.synchronous != validity {
            debug!(
                "{} sync validity changed: {:?} -> {:?}",
                type_name::<T>(),
                self.synchronous,
                validity
            );
            self.synchronous = validity
        }
    }

    /// Get the value of the synchronous validity
    pub fn synchronous(&self) -> ValiditySynchronous {
        self.synchronous.clone()
    }
}

/// Implementing types can be validated in their structure/format/etc.  Whether
/// they are valid with reference to others is not dealt with here.  For
/// example, the name of a subtimeline may be valid, but it may fail other
/// checks due to it not actaully being the name of a timeline.
///
/// These checks are all synchronous. The other checks may not be.
pub trait ValidSynchronous {
    /// Is the target data synchronously valid (no database checks)
    fn is_valid_synchronous(&self) -> bool;

    /// Re-run the synchronous validation checks
    fn update_validity_synchronous(&mut self);

    /// Get the synchronous validity ([`ValiditySynchronous`])
    fn validity_synchronous(&self) -> ValiditySynchronous;
}

/// Implementing types can be validated in their relations to other things and
/// the database.  For example, if the name of a subtimeline is asynchronously
/// valid it means that it appears as a timeline in the database.
///
/// These checks are all asynchronous, but that fact is hidden due to the checks
/// being carried out in newly spawned threads.
pub trait ValidAsynchronous {
    type Error: Display;

    /// Is the target data asynchronously valid (database checks)
    fn is_valid_asynchronous(&self) -> Option<Result<(), Self::Error>>;

    /// Set off the asynchronous validation checks
    fn trigger_asynchronous_validity_update(&mut self);

    /// Receive any aysnchronous validity updates
    fn check_for_asynchronous_validity_response(&mut self);
}

/// Implementing types can be validated in all regards (synchronous and
/// asynchronous validity).  External code should only use these functions.
pub trait Valid: ValidSynchronous + ValidAsynchronous {
    /// Whether the data is both synchronoulsy & asynchronously valid or not.
    fn validity(&self) -> ValidityAsynchronous {
        if !self.is_valid_synchronous() {
            match self.validity_synchronous() {
                ValiditySynchronous::Invalid(error) => return ValidityAsynchronous::Invalid(error),
                ValiditySynchronous::Valid => panic!(),
            }
        }
        match self.is_valid_asynchronous() {
            Some(result) => match result {
                Ok(()) => ValidityAsynchronous::Valid,

                // TODO: this could be wrong (might just be a broken database
                // connection, in which case it's TempValidity::Waiting or
                // something else)
                Err(error) => ValidityAsynchronous::Invalid(error.to_string()),
            },
            None => ValidityAsynchronous::Waiting,
        }
    }

    /// Re-run validity checking code (short circuits if the data is
    /// synchronously invalid).
    fn update_validity(&mut self) {
        self.update_validity_synchronous();
        if !self.is_valid_synchronous() {
            return;
        }
        self.trigger_asynchronous_validity_update();
    }

    // // e.g. is the end date after the start date
    // fn is_semantically_valid(&mut self) -> bool;

    // // e.g. do our tag components parse successfully
    // fn is_format_valid(&mut self) -> bool;

    // // e.g. does an entity have at least a name, a start year, and 1 tag
    // fn is_structurally_valid(&mut self) -> bool;
}
