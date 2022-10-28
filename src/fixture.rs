use crate::comet::{ControlMessage as CometControlMessage, StateChange as CometStateChange};
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
}

#[derive(Clone, Copy, Debug)]
pub enum ControlMessage {
    Comet(CometControlMessage),
    Lumasphere(LumasphereControlMessage),
    Venus(VenusControlMessage),
}
