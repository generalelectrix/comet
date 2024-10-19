use crate::fixture::aquarius::{Aquarius, ControlMessage, StateChange};
use crate::fixture::prelude::*;use crate::osc::prelude::*;
use crate::osc::basic_controls::{button, Button};
use crate::osc::{GroupControlMap, HandleOscStateChange};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = Aquarius::NAME.0;
