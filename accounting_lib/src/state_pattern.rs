use derive_more::Display;
use std::marker::PhantomData;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug, Display)]
pub struct MutableState;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Wrapper<T, State = MutableState> {
    value: T,
    _state: PhantomData<State>,
}

impl<T> Wrapper<T> {
    /// Get a mutable reference to the inner value
    pub const fn get_value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// set value to the wrapper
    pub fn set_value(&mut self, value: T) -> &mut Self {
        self.value = value;
        self
    }

    /// set value from wrapper to the wrapper
    pub fn set_value_wrapper<State>(&mut self, wrapper: Wrapper<T, State>) -> &mut Self {
        self.value = wrapper.value;
        self
    }
}

impl<T, State> Wrapper<T, State> {
    /// Get a reference to the inner value
    pub const fn get_value(&self) -> &T {
        &self.value
    }

    /// Consume the wrapper and return the inner value
    pub fn strip(self) -> T {
        self.value
    }

    /// Converts from OldState to NewState from the input type.
    pub fn transmute<NewState>(self) -> Wrapper<T, NewState> {
        Wrapper {
            value: self.value,
            _state: PhantomData,
        }
    }

    /// Converts from OldState to NewState from the input type.
    pub const fn transmute_ref<NewState>(&self) -> &Wrapper<T, NewState> {
        check_alignment::<_, &Wrapper<T, NewState>>(&self);
        unsafe { std::mem::transmute(self) }
    }

    /// Converts from OldState to NewState from the input type.
    pub const fn transmute_ref_mut<NewState>(&mut self) -> &mut Wrapper<T, NewState> {
        check_alignment::<_, &mut Wrapper<T, NewState>>(&self);
        unsafe { std::mem::transmute(self) }
    }
}

impl<T, State> From<T> for Wrapper<T, State> {
    fn from(value: T) -> Self {
        Wrapper {
            value,
            _state: PhantomData,
        }
    }
}

impl<T, State: 'static> Wrapper<T, State> {
    pub fn is_state<S: 'static>(&self) -> bool {
        std::any::TypeId::of::<State>() == std::any::TypeId::of::<S>()
    }
}

pub const fn check_alignment<Old, New>(_: &Old) {
    assert!(
        std::mem::align_of::<Old>() == std::mem::align_of::<New>(),
        "Cannot transmute: alignment differs"
    );
}

/// Converts OldDataType to NewDataType from the input , with size check (compile-time check) and alignment check (run-time check)
#[macro_export]
macro_rules! transmute {
    ($NewDataType:ty,$wrapper:expr) => {{
        state_pattern::check_alignment::<_, $NewDataType>(&$wrapper);
        unsafe { std::mem::transmute::<_, $NewDataType>($wrapper) }
    }};
}

#[macro_export]
macro_rules! create_type {
    ($name : ident , $Type : ty , $($state : ident),* ) => {
        type $name<State = state_pattern::MutableState> = state_pattern::Wrapper<$Type, State>;

        paste::paste! {
            $(
                struct $state;
                type [<$name $state>] = $name<$state>;
            )*
        }
    };
}
