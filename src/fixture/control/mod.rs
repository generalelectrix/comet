//! Declarative fixture control models.
//! These types are intended to provide both a data model for fixture state,
//! as well as standardized ways to interact with that state.

use number::UnipolarFloat;

pub struct UnipolarFader {
    val: UnipolarFloat,
    osc_name: String,
}
