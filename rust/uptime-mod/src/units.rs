//! Per-host workload counts, flattened out of the snapshot's hosts.

use crate::snapshot::{HostView, ModSnapshot};

/// What one host runs, from the unit-bearing fields of its [`HostView`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HostUnits<'a> {
    /// The host's id in the snapshot.
    pub host_id: u32,
    /// Units placed on the host.
    pub units: u32,
    /// Units the host holds that cannot run (the host is down or draining).
    pub stranded: u32,
    /// Service classes the host serves, by engine tag.
    pub service_classes: &'a [String],
}

impl<'a> From<&'a HostView> for HostUnits<'a> {
    fn from(h: &'a HostView) -> Self {
        Self {
            host_id: h.id,
            units: h.unit_count,
            stranded: h.stranded_unit_count,
            service_classes: &h.service_classes,
        }
    }
}

/// One row per host in the snapshot, in snapshot order. Empty when the mod's
/// snapshot withholds the `hosts` section.
pub fn units(snapshot: &ModSnapshot) -> impl Iterator<Item = HostUnits<'_>> {
    snapshot.hosts.iter().flatten().map(HostUnits::from)
}
