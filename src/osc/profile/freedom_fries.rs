use crate::fixture::color::StateChange as ColorStateChange;
use crate::fixture::freedom_fries::{FreedomFries as FreedomFriesFixture, StateChange};
use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::ControlMessagePayload;
use crate::fixture::PatchAnimatedFixture;
use crate::osc::basic_controls::{button, Button};
use crate::osc::label_array::LabelArray;
use crate::osc::profile::color::map_color;
use crate::osc::profile::generic::map_strobe;
use crate::osc::{GroupControlMap, HandleOscStateChange, MapControls};
use crate::util::unipolar_to_range;

const GROUP: &str = "FreedomFries";

const RUN_PROGRAM: Button = button(GROUP, "RunProgram");
const PROGRAM_CYCLE_ALL: Button = button(GROUP, "ProgramCycleAll");

const PROGRAM_SELECT_LABEL: LabelArray = LabelArray {
    group: GROUP,
    control: "ProgramLabel",
    n: 1,
    empty_label: "",
};

impl MapControls for FreedomFriesFixture {
    fn group(&self) -> &'static str {
        GROUP
    }

    fn map_controls(&self, map: &mut GroupControlMap<ControlMessagePayload>) {
        use StateChange::*;

        map.add_unipolar("Dimmer", |v| ControlMessagePayload::fixture(Dimmer(v)));
        map_color(map, &wrap_color);
        map_strobe(map, "Strobe", &wrap_strobe);
        map.add_unipolar("Speed", |v| ControlMessagePayload::fixture(Speed(v)));
        RUN_PROGRAM.map_state(map, |v| ControlMessagePayload::fixture(RunProgram(v)));
        map.add_unipolar("Program", |v| {
            ControlMessagePayload::fixture(Program(unipolar_to_range(
                0,
                FreedomFriesFixture::PROGRAM_COUNT as u8 - 1,
                v,
            ) as usize))
        });
        PROGRAM_CYCLE_ALL.map_state(map, |v| ControlMessagePayload::fixture(ProgramCycleAll(v)));
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        vec![(GROUP.to_string(), Self::NAME)]
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessagePayload {
    ControlMessagePayload::fixture(StateChange::Strobe(sc))
}

fn wrap_color(sc: ColorStateChange) -> ControlMessagePayload {
    ControlMessagePayload::fixture(StateChange::Color(sc))
}

impl HandleOscStateChange<StateChange> for FreedomFriesFixture {
    fn emit_osc_state_change<S>(sc: StateChange, send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
        if let StateChange::Program(v) = sc {
            let label = v.to_string();
            PROGRAM_SELECT_LABEL.set([label].into_iter(), send);
        }
    }
}
