use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use mio::{Evented, Poll, PollOpt, Ready, Token};

use list::ErasedList;

pub mod channel;
pub mod generic;
#[cfg(target_os = "linux")]
pub mod signals;
pub mod timer;

/// Trait representing a source that can be inserted into an EventLoop
///
/// This is the interface between the source and the loop, you need to
/// implement it to use your custom event sources.
pub trait EventSource: Evented {
    /// The type of events generated by your sources
    type Event;

    /// The interest value that will be given to `mio` when registering your source
    fn interest(&self) -> Ready;

    /// The pollopt value that will be given to `mio` when registering your source
    fn pollopts(&self) -> PollOpt;

    /// Wrap an user callback into a dispatcher, that will convert an `mio` readiness
    /// into an event
    fn make_dispatcher<Data: 'static, F: FnMut(Self::Event, &mut Data) + 'static>(
        &self,
        callback: F,
    ) -> Rc<RefCell<EventDispatcher<Data>>>;
}

/// An event dispatcher
///
/// It is the junction between user callbacks and and an event source,
/// receiving `mio` readinesses, converting them into appropriate events
/// and calling their inner user callback.
pub trait EventDispatcher<Data> {
    /// The source has a readiness event
    fn ready(&mut self, ready: Ready, data: &mut Data);
}

/// An event source that has been inserted into the event loop
///
/// This handle allows you to remove it, and possibly more interactions
/// depending on the source kind that will be provided by the `Deref`
/// implementation of this struct to the evented object.
///
/// Dropping this handle does not deregister this source from the event loop,
/// but will drop the wrapped `EventSource`, maybe rendering it inert depending on
/// its implementation.
pub struct Source<E: EventSource> {
    pub(crate) source: E,
    pub(crate) poll: Rc<Poll>,
    pub(crate) list: Rc<RefCell<ErasedList>>,
    pub(crate) token: Token,
}

impl<E: EventSource> Source<E> {
    /// Refresh the registration of this event source to the loop
    ///
    /// This can be necessary if the evented object provides methods to change
    /// its behavior. Its documentation should inform you of the need for re-registration.
    pub fn reregister(&self) -> io::Result<()> {
        self.poll.reregister(
            &self.source,
            self.token,
            self.source.interest(),
            self.source.pollopts(),
        )
    }

    /// Remove this source from the event loop
    ///
    /// You are given the evented object back.
    pub fn remove(self) -> E {
        let _ = self.poll.deregister(&self.source);
        let _dispatcher = self.list.borrow_mut().del_source(self.token);
        self.source
    }
}

impl<E: EventSource> ::std::ops::Deref for Source<E> {
    type Target = E;
    fn deref(&self) -> &E {
        &self.source
    }
}

impl<E: EventSource> ::std::ops::DerefMut for Source<E> {
    fn deref_mut(&mut self) -> &mut E {
        &mut self.source
    }
}

/// An idle callback that was inserted in this loop
///
/// This handle allows you to cancel the callback. Dropping
/// it will *not* cancel it.
pub struct Idle {
    pub(crate) callback: Rc<RefCell<ErasedIdle>>,
}

impl Idle {
    /// Cancel the idle callback if it was not already run
    pub fn cancel(self) {
        self.callback.borrow_mut().cancel();
    }
}

pub(crate) trait ErasedIdle {
    fn cancel(&mut self);
}

impl<Data> ErasedIdle for Option<Box<FnMut(&mut Data)>> {
    fn cancel(&mut self) {
        self.take();
    }
}
