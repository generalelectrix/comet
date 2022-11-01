//! Control abstractions that re-usable across fixture types.

use number::UnipolarFloat;

#[derive(Default, Clone, Debug)]
pub struct StrobeState {
    on: bool,
    rate: UnipolarFloat,
}
