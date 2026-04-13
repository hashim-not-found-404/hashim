use crate::state_pattern::{MutableState, Wrapper};
use derive_more::Display;

mod state_pattern;

fn main() {
    // Create initial state
    let state1 = EntryState1::from(64);
    dbg!(state1.get_value());

    // Create initial state
    let state1: EntryState1 = 64.into();
    dbg!(state1.get_value());

    // Transform to State2
    let state2: EntryState2 = state1.transmute();
    dbg!(state2.get_value());

    let mut state3: Entry = state2.transmute();
    dbg!(state3.get_value_mut());

    let c = state3.is_state::<State1>();
    dbg!(c);

    let c = state3.is_state::<State2>();
    dbg!(c);

    let c = t1::from(Wrapper::<[i8; 2], S1>::from([5, 6]));
    dbg!(&c);
    // let c = transmute!([i8; 2], c);
    // dbg!(c);

    let c = EntryState1::from(55000);
    // dbg!(&c);
    let c = t1::from(Wrapper::<[i8; 2], S1>::from([8, 4]));
    // dbg!(&c);

    let x = EntryState1::from(55);
}

#[derive(Debug, Display)]
struct S1;
#[derive(Debug, Display)]
struct S2;
type t1 = state_pattern::Wrapper<Wrapper<[i8; 2], S1>, S1>;
type t2 = state_pattern::Wrapper<Wrapper<[u16; 1], S2>, S2>;

// #[derive(Debug, Display, Debug, Display)]

create_type!(Entry, i32, State1, State2);
// create_type!(hoshhosh, i32, State1, State2);

// create_type!(hashem, i32, s1);

// fn dd<T>(d: Entry<T>) -> Entry {
//     d.transmute()
// }
