//! A control for a string-labeled menu of choices.
//! This models simple things like color selection, where a choice directly corresponds
//! to a DMX value.

use anyhow::{anyhow, bail};
use itertools::Itertools;
use rosc::OscType;

use crate::osc::ScopedOscMessage;

use super::{OscControl, RenderToDmxWithAnimations};

/// Select from a menu of labeled options.
#[derive(Debug)]
pub struct LabeledSelect {
    /// Currently-selected value.
    selected: usize,
    /// The menu of pairs of label and DMX value.
    options: Vec<(&'static str, u8)>,
    /// Name of this control.
    name: String,
    /// Offset into DMX buffer to render into.
    dmx_buf_offset: usize,
}

impl LabeledSelect {
    pub fn new<S: Into<String>>(
        name: S,
        dmx_buf_offset: usize,
        options: Vec<(&'static str, u8)>,
    ) -> Self {
        assert!(!options.is_empty());
        Self {
            selected: 0,
            options,
            name: name.into(),
            dmx_buf_offset,
        }
    }

    pub fn labels(&self) -> impl Iterator<Item = &str> {
        self.options.iter().map(|(l, _)| *l)
    }
}

impl OscControl<&str> for LabeledSelect {
    fn control_direct(
        &mut self,
        val: &str,
        emitter: &dyn crate::osc::EmitScopedOscMessage,
    ) -> anyhow::Result<()> {
        let Some(i) = self
            .labels()
            .enumerate()
            .filter_map(|(i, label)| (label == val).then_some(i))
            .next()
        else {
            bail!(
                "the label {val} did not match any valid option for {}:\n{}",
                self.name,
                self.labels().join(", ")
            );
        };
        // If selected is same as current, do nothing.
        if i == self.selected {
            return Ok(());
        }
        self.selected = i;
        self.emit_state(emitter);
        Ok(())
    }

    fn control(
        &mut self,
        msg: &crate::osc::OscControlMessage,
        emitter: &dyn crate::osc::EmitScopedOscMessage,
    ) -> anyhow::Result<bool> {
        if msg.control() != self.name {
            return Ok(false);
        }
        let name = msg
            .addr_payload()
            .split('/')
            .nth(1)
            .ok_or_else(|| msg.err("command is missing variant specifier"))?;
        let Some(i) = self
            .labels()
            .enumerate()
            .filter_map(|(i, label)| (label == name).then_some(i))
            .next()
        else {
            bail!(
                "the label {name} did not match any valid option for {}:\n{}",
                self.name,
                self.labels().join(", ")
            );
        };
        // If selected is same as current, do nothing.
        if i == self.selected {
            return Ok(true);
        }
        self.selected = i;
        self.emit_state(emitter);
        Ok(true)
    }

    fn emit_state(&self, emitter: &dyn crate::osc::EmitScopedOscMessage) {
        for (i, label) in self.labels().enumerate() {
            // TODO: consider caching outgoing addresses
            // We could also do this for matching incoming addresses.
            emitter.emit_osc(ScopedOscMessage {
                control: &format!("/{}/{}", self.name, label),
                arg: OscType::Float(if i == self.selected { 1.0 } else { 0.0 }),
            });
        }
    }
}

impl RenderToDmxWithAnimations for LabeledSelect {
    fn render(&self, _animations: impl Iterator<Item = f64>, dmx_buf: &mut [u8]) {
        dmx_buf[self.dmx_buf_offset] = self.options[self.selected].1;
    }
}
