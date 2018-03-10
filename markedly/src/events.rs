use std::collections::{VecDeque};
use std::rc::{Rc};
use std::cell::{RefCell};

use template::{EventHook};

/// Data for interacting with an active UI component tree inserted through a template.
#[derive(Clone)]
pub struct EventSink {
    events: Rc<RefCell<VecDeque<String>>>,
}

impl EventSink {
    pub(crate) fn new() -> Self {
        EventSink {
            events: Default::default(),
        }
    }

    /// Retrieves the next event raised by a component, or returns None.
    pub fn next(&self) -> Option<String> {
        self.events.borrow_mut().pop_front()
    }

    /// Raises an event.
    pub fn raise(&self, event: &EventHook) {
        match *event {
            EventHook::Direct(ref value) =>
                self.events.borrow_mut().push_back(value.clone()),
            EventHook::Script(ref _script) =>
                unimplemented!(),
        }
    }
}
