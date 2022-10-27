use crate::comet::{ControlMessage as CometControlMessage, StateChange as CometStateChange};
use crate::lumasphere::{
    ControlMessage as LumasphereControlMessage, StateChange as LumasphereStateChange,
};

pub trait EmitStateChange {
    fn emit(&mut self, sc: StateChange);
}

#[derive(Clone, Copy)]
pub enum StateChange {
    Comet(CometStateChange),
    Lumasphere(LumasphereStateChange),
}

#[derive(Clone, Copy)]
pub enum ControlMessage {
    Comet(CometControlMessage),
    Lumasphere(LumasphereControlMessage),
}
