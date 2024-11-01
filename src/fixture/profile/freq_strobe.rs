//! Profile for the Big Bar, the American DJ Freq Strobe 16.
use std::time::{Duration, Instant};

use crate::fixture::prelude::*;

#[derive(Debug, EmitState, Control)]
pub struct FreqStrobe {
    #[channel_control]
    #[animate]
    dimmer: ChannelLevelUnipolar<UnipolarChannel>,
    run: Bool<()>,
    rate: Unipolar<()>,
    #[skip_emit]
    #[skip_control]
    flasher: Flasher,
}

impl Default for FreqStrobe {
    fn default() -> Self {
        Self {
            dimmer: Unipolar::full_channel("Dimmer", 16).with_channel_level(),
            // strobe: Strobe::channel("Strobe", 17, 9, 131, 0),
            run: Bool::new_off("Run", ()),
            rate: Unipolar::new("Rate", ()),
            flasher: Flasher::default(),
        }
    }
}

impl PatchAnimatedFixture for FreqStrobe {
    const NAME: FixtureType = FixtureType("FreqStrobe");
    fn channel_count(&self) -> usize {
        18
    }
}

impl ControllableFixture for FreqStrobe {
    fn update(&mut self, master_controls: &MasterControls, dt: std::time::Duration) {
        let master_strobe = master_controls.strobe();
        let run = master_strobe.state.on && self.run.val();
        let rate = if master_controls.strobe().use_master_rate {
            master_controls.strobe().state.rate
        } else {
            self.rate.val()
        };
        self.flasher.update(dt, run, rate);
    }
}

impl AnimatedFixture for FreqStrobe {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        self.flasher.render(group_controls, dmx_buf);
        self.dimmer.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::Dimmer),
            dmx_buf,
        );
    }
}

#[derive(Debug)]
struct Flasher {
    state: [Option<Flash>; 16],
    flash_len: Duration,
    last_flash_age: Duration,
    next_cell: usize,
}

impl Default for Flasher {
    fn default() -> Self {
        Self {
            state: Default::default(),
            flash_len: Duration::from_millis(40),
            last_flash_age: Default::default(),
            next_cell: 0,
        }
    }
}

fn render_state_iter<'a>(iter: impl Iterator<Item = &'a Option<Flash>>, dmx_buf: &mut [u8]) {
    for (state, chan) in iter.zip(dmx_buf.iter_mut()) {
        *chan = if state.is_some() { 255 } else { 0 }
    }
}

impl Flasher {
    fn render(&self, group_controls: &FixtureGroupControls, dmx_buf: &mut [u8]) {
        if group_controls.mirror {
            render_state_iter(self.state.iter().rev(), dmx_buf);
        } else {
            render_state_iter(self.state.iter(), dmx_buf);
        }
    }

    fn update(&mut self, dt: Duration, run: bool, rate: UnipolarFloat) {
        for flash in &mut self.state {
            if let Some(f) = flash {
                f.age += dt;
                if f.age >= self.flash_len {
                    *flash = None;
                }
            }
        }
        self.last_flash_age += dt;
        if run && self.last_flash_age >= interval_from_rate(rate) {
            self.state[self.next_cell] = Some(Flash::default());
            self.last_flash_age = Duration::ZERO;
            self.next_cell = (self.next_cell + 1) % 16;
        }
    }
}

const MAX_INTERVAL_MILLIS: u64 = 1000;

fn interval_from_rate(rate: UnipolarFloat) -> Duration {
    // lowest rate: 1 flash/sec => 1 sec interval
    // highest rate: 50 flash/sec => 20 ms interval
    // use exact frame intervals
    // FIXME: this should depend on the show framerate explicitly.
    if rate == UnipolarFloat::ZERO {
        return Duration::from_millis(MAX_INTERVAL_MILLIS);
    }
    let raw_interval = (MAX_INTERVAL_MILLIS as f64 / (rate.val() * 10.)) as u64 - 80;
    let coerced_interval = ((raw_interval / 20) * 20).clamp(20, MAX_INTERVAL_MILLIS);
    Duration::from_millis(coerced_interval)
}

#[derive(Debug, Default)]
struct Flash {
    age: Duration,
}
