use crate::domain::HashimSignal;
use dioxus::core::ReactiveContext;
use dioxus::prelude::*;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::DeserializeOwned;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Debug;
use std::mem::take;
use std::prelude::v1::Result;
use std::sync::Arc;
use std::sync::Mutex;

pub struct MySignal<T> {
    value: Arc<Mutex<T>>,
    subscribers: Arc<Mutex<HashSet<ReactiveContext>>>,
}

impl<T: Debug> Debug for MySignal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MySignal")
            .field("value", &self.value.lock().unwrap())
            .finish()
    }
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

impl<T: 'static + Clone + Default + DeserializeOwned + Serialize> HashimSignal<T> for MySignal<T> {
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
        let mut subscribers = take(&mut *self.subscribers.lock().unwrap());
        subscribers.retain(|reactive_context| reactive_context.mark_dirty());
        // Extend the subscribers list instead of overwriting it in case a subscriber is added while reactive contexts are marked dirty
        self.subscribers.lock().unwrap().extend(subscribers);
    }
}

impl<T: 'static + Clone + Default + DeserializeOwned + Serialize> Serialize for MySignal<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.read().serialize(serializer)
    }
}

impl<'de, T: 'static + Clone + Default + DeserializeOwned + Serialize> Deserialize<'de>
    for MySignal<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(|a| {
            let my_signal = Self::default();
            my_signal.set(a);
            my_signal
        })
    }
}
