use dioxus::{core::ReactiveContext, prelude::*};
use my_core::prelude::{AllSignalTypes, Signal, *};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex, MutexGuard},
};

pub struct MySignal<T> {
    value: Arc<Mutex<T>>,
    subscribers: Arc<Mutex<HashSet<ReactiveContext>>>,
}

pub fn my_use_signal<T: 'static>(init: impl FnOnce() -> T) -> MySignal<T> {
    use_hook(|| {
        // A set of subscribers to notify about changes to this signals value
        let subscribers = Default::default();
        // Create the initial state
        let value = Arc::new(Mutex::new(init()));

        MySignal { value, subscribers }
    })
}

impl<T> MySignal<T> {
    pub fn write(&self) -> MutexGuard<'_, T> {
        self.value.lock().unwrap()
    }
}

impl<T> Clone for MySignal<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            subscribers: self.subscribers.clone(),
        }
    }
}

impl<T: PartialEq + Clone> PartialEq for MySignal<T> {
    fn eq(&self, other: &Self) -> bool {
        let value_eq = {
            let s = { self.value.try_lock().unwrap().clone() };
            let o = { other.value.try_lock().unwrap().clone() };
            s == o
        };

        if !value_eq {
            return false;
        }

        let subscribers_eq = {
            let s = { self.subscribers.try_lock().unwrap().clone() };
            let o = { other.subscribers.try_lock().unwrap().clone() };
            s == o
        };

        subscribers_eq
    }
}

impl<T: 'static + Clone + Default> Default for MySignal<T> {
    fn default() -> Self {
        my_use_signal(|| T::default())
    }
}

impl<T: 'static + Clone + Default> Signal<T> for MySignal<T> {
    fn read(&self) -> T {
        // Subscribe the context observing the signal (if any) to updates of its value.
        if let Some(reactive_context) = ReactiveContext::current() {
            reactive_context.subscribe(self.subscribers.clone());
        }

        self.value.lock().unwrap().clone()
    }

    fn set(&self, value: T) {
        // Update the state
        *self.value.lock().unwrap() = value;
        // Trigger a re-render of the components that observed the signal's previous value
        let mut subscribers = std::mem::take(&mut *self.subscribers.lock().unwrap());
        subscribers.retain(|reactive_context| reactive_context.mark_dirty());
        // Extend the subscribers list instead of overwriting it in case a subscriber is added while reactive contexts are marked dirty
        self.subscribers.lock().unwrap().extend(subscribers);
    }
}

struct MySignalForLists(MySignal<Vec<String>>);

impl MySignalForLists {
    fn read_all(&self) -> Vec<String> {
        self.0.read()
    }
}

impl Default for MySignalForLists {
    fn default() -> Self {
        Self(my_use_signal(|| Vec::new()))
    }
}

impl Signal<String> for MySignalForLists {
    fn read(&self) -> String {
        self.0.read().last().unwrap_or(&String::default()).clone()
    }
    fn set(&self, v: String) {
        self.0.write().push(v);
    }
}

#[derive(Default)]
pub struct MyAllSignalTypes;
impl AllSignalTypes for MyAllSignalTypes {
    type OptionRowId = MySignal<Option<db_types::UuidType>>;
    type String = MySignal<String>;
    type Bool = MySignal<bool>;
    type StringVec = MySignal<String>;
    type Currency = MySignal<db_types::Currency>;
    type Location = MySignal<db_types::Location>;
}
