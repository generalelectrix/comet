//! A control for a string-labeled menu of choices.
//! This models simple things like color selection, where a choice directly corresponds
//! to a DMX value.

use anyhow::{anyhow, bail};
use itertools::Itertools;
use rosc::OscType;

use crate::osc::ScopedOscMessage;

use super::OscControl;

pub struct LabeledSelect {
    /// Currently-selected value.
    selected: usize,
    /// The menu of pairs of label and DMX value.
    options: Vec<(String, u8)>,
    /// Name of this control.
    name: String,
}

impl LabeledSelect {
    pub fn new<S: Into<String>>(name: S, options: Vec<(String, u8)>) -> Self {
        assert!(!options.is_empty());
        Self {
            selected: 0,
            options,
            name: name.into(),
        }
    }

    pub fn labels(&self) -> impl Iterator<Item = &str> {
        self.options.iter().map(|(l, _)| l.as_str())
    }
}

impl OscControl<str> for LabeledSelect {
    fn val(&self) -> &str {
        &self.options[self.selected].0
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
