use crate::comet::{ControlMessage as CometControlMessage, StateChange as CometStateChange};
use crate::h2o::{ControlMessage as H2OControlMessage, StateChange as H2OStateChange};
use crate::lumasphere::{
    ControlMessage as LumasphereControlMessage, StateChange as LumasphereStateChange,
};
use crate::venus::{ControlMessage as VenusControlMessage, StateChange as VenusStateChange};

pub trait EmitStateChange {
    fn emit(&mut self, sc: StateChange);
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    Comet(CometStateChange),
    Lumasphere(LumasphereStateChange),
    Venus(VenusStateChange),
    H2O(H2OStateChange),
}

#[derive(Clone, Copy, Debug)]
pub enum ControlMessage {
    Comet(CometControlMessage),
    Lumasphere(LumasphereControlMessage),
    Venus(VenusControlMessage),
    H2O(H2OControlMessage),
}
