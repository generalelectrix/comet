use rosc::{OscMessage, OscType};

/// Model a 1D array of labels.
/// Allows dynamic remapping of a subset of labels.
/// Each individual label should accept an address of the form /group/control/n
#[derive(Clone)]
pub struct LabelArray {
    pub group: &'static str,
    pub control: &'static str,
    pub empty_label: &'static str,
    pub n: usize,
}

impl LabelArray {
    /// Write labels to this array.
    /// If there are more labels provided than defined for this array,
    /// the extra lables are ignored.
    pub fn set<S>(&self, labels: impl Iterator<Item = String>, send: &mut S)
    where
        S: FnMut(OscMessage),
    {
        for (i, label) in labels
            .chain(std::iter::repeat(self.empty_label).map(String::from))
            .enumerate()
        {
            if i >= self.n {
                return;
            }
            send(OscMessage {
                addr: format!("/{}/{}/{}", self.group, self.control, i),
                args: vec![OscType::String(label)],
            })
        }
    }
}
