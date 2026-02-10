// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! All macros
//!

// TODO: I used (unbounded, ...) on a bounded channel and got no errors or warnings etc (catch this)
/// Run a code block that requires a database connection in a context in which
/// one is made available.  The transaction is not committed and thus this
/// should only be used in contexts where that is desired (i.e. read-only).
#[macro_export]
macro_rules! spawn_transaction_no_commit_send_result {
    ($shared_config:ident, bounded, $tx:ident, $fetch_fn:expr) => {
        tokio::spawn(async move {
            // Get database path
            let db_pool = $shared_config.read().await.db_pool.clone();

            // TODO
            // If testing, vary the seconds
            // tokio::time::sleep(std::time::Duration::from_secs(0)).await;

            let result = async {
                let mut transaction = db_pool.begin().await?;
                $fetch_fn(&mut transaction).await
            }
            .await;
            let _ = $tx.send(result).await;
        });
    };

    ($shared_config:ident, unbounded, $tx:ident, $fetch_fn:expr) => {
        tokio::spawn(async move {
            // Get database path
            let db_pool = $shared_config.read().await.db_pool.clone();

            // If testing
            tokio::time::sleep(std::time::Duration::from_secs(0)).await;

            let result = async {
                let mut transaction = db_pool.begin().await?;
                $fetch_fn(&mut transaction).await
            }
            .await;
            let _ = $tx.send(result);
        });
    };
}

/// Helper macro that implements [`open_timeline_gui_core::ValidAsynchronous`] for some type
/// for which it should never be called (all methods panic)
#[macro_export]
macro_rules! impl_valid_asynchronous_macro_never_called {
    ($type:ty) => {
        impl open_timeline_gui_core::ValidAsynchronous for $type {
            type Error = open_timeline_crud::CrudError;

            fn is_valid_asynchronous(&self) -> Option<Result<(), Self::Error>> {
                // Do nothing.  Components update their validity themselves.
                panic!()
            }

            fn check_for_asynchronous_validity_response(&mut self) {
                // Do nothing.  Components update their validity themselves.
                panic!()
            }

            fn trigger_asynchronous_validity_update(&mut self) {
                // Do nothing.  Components update their validity themselves.
                panic!()
            }
        }
    };
}

/// Helper macro that implements [`open_timeline_gui_core::ValidSynchronous`] for some type
/// for which it should never be called (all methods panic)
#[macro_export]
macro_rules! impl_valid_synchronous_macro_never_called {
    ($type:ty) => {
        impl open_timeline_gui_core::ValidSynchronous for $type {
            fn is_valid_synchronous(&self) -> bool {
                // Do nothing.  Components update their validity themselves.
                panic!()
            }

            fn update_validity_synchronous(&mut self) {
                // Do nothing.  Components update their validity themselves.
                panic!()
            }

            fn validity_synchronous(&self) -> open_timeline_gui_core::ValiditySynchronous {
                // Do nothing.  Components update their validity themselves.
                panic!()
            }
        }
    };
}

/// Helper macro that returns the [`open_timeline_gui_core::ValidityAsynchronous`] of some
/// iterable
#[macro_export]
macro_rules! impl_is_valid_method_for_iterable {
    ($iterable:expr) => {{
        for validity in $iterable {
            match validity {
                open_timeline_gui_core::ValidityAsynchronous::Invalid(error) => {
                    return open_timeline_gui_core::ValidityAsynchronous::Invalid(error);
                }
                open_timeline_gui_core::ValidityAsynchronous::Waiting => {
                    return open_timeline_gui_core::ValidityAsynchronous::Waiting;
                }
                open_timeline_gui_core::ValidityAsynchronous::Valid => continue,
            }
        }
        open_timeline_gui_core::ValidityAsynchronous::Valid
    }};
}
