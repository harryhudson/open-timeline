// SPDX-License-Identifier: GPL-3.0-or-later

//!
//!
//!

/// Whether to show the remove button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowRemoveButton {
    Yes,
    No,
}

/// Used to indicate whether the window was opened for creating or editing
/// something.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CreateOrEdit {
    Edit,
    Create,
}
