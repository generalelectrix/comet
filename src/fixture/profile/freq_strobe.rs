//! Profile for the Big Bar, the American DJ Freq Strobe 16.
use std::time::{Duration, Instant};

use crate::fixture::prelude::*;

#[derive(Debug, EmitState, Control)]
pub struct FreqStrobe {
    #[channel_control]
    #[animate]
    dimmer: ChannelLevelUnipolar<UnipolarChannel>,
    strobe: StrobeChannel,
    run: Bool<()>,
    #[skip_emit]
    #[skip_control]
    flasher: Flasher,
}

impl Default for FreqStrobe {
    fn default() -> Self {
        Self {
            dimmer: Unipolar::full_channel("Dimmer", 16).with_channel_level(),
            strobe: Strobe::channel("Strobe", 17, 9, 131, 0),
            run: Bool::new_off("Run", ()),
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
    fn update(&mut self, dt: std::time::Duration) {
        self.flasher.update(dt, self.run.val());
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
        self.flasher.render(dmx_buf);
        self.dimmer.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::Dimmer),
            dmx_buf,
        );
        self.strobe
            .render_with_group(group_controls, std::iter::empty(), dmx_buf);
    }
}

#[derive(Debug)]
struct Flasher {
    state: [Option<Flash>; 16],
    flash_len: Duration,
    last_flash_age: Duration,
    interval: Duration,
    next_cell: usize,
}

impl Default for Flasher {
    fn default() -> Self {
        Self {
            state: Default::default(),
            flash_len: Duration::from_millis(40),
            last_flash_age: Default::default(),
            interval: Duration::from_millis(500),
            next_cell: 0,
        }
    }
}

impl Flasher {
    fn render(&self, dmx_buf: &mut [u8]) {
        for (state, chan) in self.state.iter().zip(dmx_buf) {
            *chan = if state.is_some() { 255 } else { 0 }
        }
    }

    fn update(&mut self, dt: Duration, run: bool) {
        for flash in &mut self.state {
            if let Some(f) = flash {
                f.age += dt;
                if f.age >= self.flash_len {
                    *flash = None;
                }
            }
        }
        self.last_flash_age += dt;
        if run && self.last_flash_age >= self.interval {
            self.state[self.next_cell] = Some(Flash::default());
            self.last_flash_age = Duration::ZERO;
            self.next_cell = (self.next_cell + 1) % 16;
        }
    }
}

#[derive(Debug, Default)]
struct Flash {
    age: Duration,
}
