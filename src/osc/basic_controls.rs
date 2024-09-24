use crate::fixture::ControlMessagePayload;

use super::{get_bool, send_float};

#[derive(Clone)]
pub struct Button {
    pub group: &'static str,
    pub control: &'static str,
}

pub const fn button(group: &'static str, control: &'static str) -> Button {
    Button { group, control }
}

impl Button {
    pub fn map_state<F>(&self, map: &mut super::FixtureControlMap, process: F)
    where
        F: Fn(bool) -> ControlMessagePayload + 'static + Copy,
    {
        map.add_fetch_process(self.group, self.control, get_bool, move |v| {
            Some(process(v))
        })
    }

    pub fn map_trigger(
        &self,
        map: &mut super::FixtureControlMap,
        event_factory: impl Fn() -> ControlMessagePayload + 'static,
    ) {
        map.add_fetch_process(self.group, self.control, get_bool, move |v| {
            if v {
                Some(event_factory())
            } else {
                None
            }
        })
    }

    pub fn send<S>(&self, val: bool, send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
        send_float(self.group, self.control, if val { 1.0 } else { 0.0 }, send);
    }
}
