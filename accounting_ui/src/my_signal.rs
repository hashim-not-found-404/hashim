use dioxus::{core::ReactiveContext, prelude::*};
use my_core::accounting_client::client_traits::HashimSignal;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

pub struct S<T> {
    value: Arc<Mutex<T>>,
    subscribers: Arc<Mutex<HashSet<ReactiveContext>>>,
}

impl<T: 'static + Default> Default for S<T> {
    fn default() -> Self {
        use_hook(|| S {
            value: Arc::new(Mutex::new(T::default())),
            subscribers: Default::default(),
        })
    }
}

impl<T> Clone for S<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            subscribers: self.subscribers.clone(),
        }
    }
}

impl<T: PartialEq + Clone> PartialEq for S<T> {
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

impl<T: 'static + Clone + Default> HashimSignal<T> for S<T> {
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
