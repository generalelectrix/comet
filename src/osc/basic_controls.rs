use super::get_bool;

#[derive(Clone)]
pub struct Button {
    pub control: &'static str,
}

pub const fn button(control: &'static str) -> Button {
    Button { control }
}

impl Button {
    pub fn map_state<F, T>(&self, map: &mut super::GroupControlMap<T>, process: F)
    where
        F: Fn(bool) -> T + 'static + Copy,
    {
        map.add_fetch_process(self.control, get_bool, move |v| Some(process(v)))
    }

    pub fn map_trigger<T>(
        &self,
        map: &mut super::GroupControlMap<T>,
        event_factory: impl Fn() -> T + 'static,
    ) {
        map.add_fetch_process(self.control, get_bool, move |v| {
            if v {
                Some(event_factory())
            } else {
                None
            }
        })
    }

    pub fn send<E>(&self, val: bool, emitter: &E)
    where
        E: crate::osc::EmitScopedOscMessage + ?Sized,
    {
        emitter.emit_float(self.control, if val { 1.0 } else { 0.0 });
    }
}
