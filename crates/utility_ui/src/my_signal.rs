use dioxus::core::ReactiveContext;
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
        Self {
            value: Arc::new(Mutex::new(T::default())),
            subscribers: Default::default(),
        }
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

impl<T> MySignal<T> {
    /// Register the current reactive context (if any) as a subscriber.
    /// Safe to call outside a render: it just no-ops.
    pub fn subscribe(&self) {
        if let Some(reactive_context) = ReactiveContext::current() {
            reactive_context.subscribe(self.subscribers.clone());
        }
    }

    /// Read the value through a shared borrow, without cloning.
    /// Subscribes the current reactive context if there is one.
    pub fn with_ref<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.subscribe();
        let guard = self.value.lock().unwrap();
        f(&guard)
    }

    /// Mutate the value through a unique borrow, without cloning.
    /// Any subscribers are marked dirty afterwards, so the components
    /// that read this signal re-render. Does not subscribe the caller
    /// (mutations should go through the actor, not through a render).
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let result = {
            let mut guard = self.value.lock().unwrap();
            f(&mut guard)
        };

        let mut subscribers = take(&mut *self.subscribers.lock().unwrap());
        subscribers.retain(|reactive_context| reactive_context.mark_dirty());
        self.subscribers.lock().unwrap().extend(subscribers);

        result
    }
}

impl<T: 'static + Clone + Default + DeserializeOwned + Serialize> Serialize for MySignal<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.with_ref(T::clone).serialize(serializer)
    }
}

impl<'de, T: 'static + Clone + Default + DeserializeOwned + Serialize> Deserialize<'de>
    for MySignal<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(|value| {
            let my_signal = Self::default();
            my_signal.with_mut(|v| *v = value);
            my_signal
        })
    }
}
