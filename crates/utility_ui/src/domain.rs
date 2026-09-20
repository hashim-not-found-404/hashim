use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait HashimSignal<T: Clone + Default + DeserializeOwned + Serialize + 'static>:
    Default + 'static + DeserializeOwned + Serialize
{
    fn reset(&self) {
        self.set(T::default());
    }
    fn read(&self) -> T;
    fn set(&self, v: T);
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub enum Dialog {
    #[default]
    Hide,
    Show,
    Error,
}
