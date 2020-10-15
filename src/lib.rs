//! Calloop, a Callback-based Event Loop
//!
//! This crate provides an [`EventLoop`] type, which is a small abstraction
//! over a polling system. The main difference between this crate
//! and other traditional rust event loops is that it is based on callbacks:
//! you can register several event sources, each being associated with a callback
//! closure that will be invoked whenever the associated event source generates
//! events.
//!
//! The main target use of this event loop is thus for apps that expect to spend
//! most of their time waiting for events and wishes to do so in a cheap and convenient
//! way. It is not meant for large scale high performance IO.
//!
//! ## How to use it
//!
//! ```no_run
//! extern crate calloop;
//!
//! use calloop::{generic::Generic, EventLoop, Interest, Mode};
//!
//! use std::time::Duration;
//!
//! fn main() {
//!     // Create the event loop
//!     let mut event_loop = EventLoop::try_new()
//!                 .expect("Failed to initialize the event loop!");
//!     // Retrieve an handle. It is used to insert new sources into the event loop
//!     // It can be cloned, allowing you to insert sources from within source callbacks
//!     let handle = event_loop.handle();
//!
//!     // Inserting an event source takes this general form
//!     // it can also be done from within the callback of an other event source
//! #   let (_ping, source) = calloop::ping::make_ping().unwrap();
//!     handle.insert_source(
//!         // a type implementing the EventSource trait
//!         source,
//!         // a callback that is invoked whenever this source generates an event
//!         |event, metadata, shared_data| {
//!             // This callback is given 3 values:
//!             // - the event generated by the source
//!             // - &mut access to some metadata, specific to the event source
//!             // - &mut access to the global shared data that was passed to EventLoop::dispatch
//!         }
//!     );
//!
//!     // Actual run of your loop
//!     //
//!     // Dispatch received events to their callbacks, waiting at most 20 ms for
//!     // new events between each invocation of the provided callback.
//!     //
//!     // The `&mut shared_data` is a mutable reference that will be forwarded to all
//!     // your callbacks, allowing them to share some state
//! #   let mut shared_data = ();
//!     event_loop.run(Duration::from_millis(20), &mut shared_data, |shared_data| {
//!         /*
//!         * Insert here the processing you need to do do between each waiting session
//!         * like your drawing logic if you're doing a GUI app for example.
//!         */
//!     });
//! }
//! ```
//!
//! ## Event source types
//!
//! The event loop is backed by an OS provided polling selector (epoll on Linux).
//!
//! This crate also provide some adapters for common event sources such as:
//!
//! - [MPSC channels](channel)
//! - [Timers](timer)
//! - [unix signals](signals) on Linux
//!
//! As well as generic objects backed by file descriptors.
//!
//! It is also possible to insert "idle" callbacks. These callbacks represent computations that
//! need to be done at some point, but are not as urgent as processing the events. These callbacks
//! are stored and then executed during [`EventLoop::dispatch`](EventLoop#method.dispatch), once all
//! events from the sources have been processed.
//!
//! ## Async/Await compatibility
//!
//! `calloop` can be used with futures, both as an executor and for monitoring Async IO.
//!
//! Activating the `executor` cargo feature will add the [`futures`] module, which provides
//! a future executor that can be inserted into an [`EventLoop`] as yet another [`EventSource`].
//!
//! IO objects can be made Async-aware via the [`LoopHandle::adapt_io`](LoopHandle#method.adapt_io)
//! method. Waking up the futures using these objects is handled by the associated [`EventLoop`]
//! directly.
//!
//! ## Custom event sources
//!
//! You can create custom event sources can will be inserted in the event loop by
//! implementing the [`EventSource`] trait. This can be done either directly from the file
//! descriptors of your source of interest, or by wrapping an other event source and further
//! processing its events. An [`EventSource`] can register more than one file descriptor and
//! aggregate them.
//!
//! ## Platforms support
//!
//! Currently, only Linux and the *BSD are supported.

#![warn(missing_docs)]

mod sys;

pub use sys::{Interest, Mode, Poll, Readiness, Token};

pub use self::loop_logic::{EventLoop, InsertError, LoopHandle, LoopSignal, RegistrationToken};
pub use self::sources::*;

pub mod io;
mod list;
mod loop_logic;
mod sources;

fn no_nix_err(err: nix::Error) -> std::io::Error {
    match err {
        ::nix::Error::Sys(errno) => errno.into(),
        _ => unreachable!(),
    }
}
