use crate::prelude::*;
use dioxus::{core::ReactiveContext, prelude::*};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex, MutexGuard},
};

pub struct MySignal<T> {
    value: Arc<Mutex<T>>,
    subscribers: Arc<Mutex<HashSet<ReactiveContext>>>,
}

impl<T: 'static + Default> Default for MySignal<T> {
    fn default() -> Self {
        use_hook(|| MySignal {
            value: Arc::new(Mutex::new(T::default())),
            subscribers: Default::default(),
        })
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

impl<T: 'static + Clone + Default> HashimSignal<T> for MySignal<T> {
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

#[derive(Default, Clone)]
pub struct MyAllSignalTypes;
impl AllSignalTypes for MyAllSignalTypes {
    type OptionRowId = MySignal<Option<db_types::UuidType>>;
    type Dialog = MySignal<front_end_model_view::Dialog>;
    type String = MySignal<String>;
    type Bool = MySignal<bool>;
    type StringVec = MySignal<String>;
    type Currency = MySignal<db_types::Currency>;
    type Location = MySignal<db_types::Location>;
}
