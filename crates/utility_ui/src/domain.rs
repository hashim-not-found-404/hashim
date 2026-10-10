use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::Mutex;

pub trait HashimSignal<T: Clone + Default + DeserializeOwned + Serialize + 'static>:
    Default + DeserializeOwned + Serialize + 'static
{
    fn reset(&self) {
        self.set(T::default());
    }
    fn read(&self) -> T;
    fn set(&self, v: T);
}

#[derive(Debug, Default)]
pub struct NonDisplayableSignal<T>(Arc<Mutex<T>>);

impl<T: Serialize> Serialize for NonDisplayableSignal<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.lock().unwrap().serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for NonDisplayableSignal<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = T::deserialize(deserializer)?;
        Ok(Self(Arc::new(Mutex::new(value))))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub enum Dialog {
    #[default]
    Hide,
    Show,
    Error,
}
