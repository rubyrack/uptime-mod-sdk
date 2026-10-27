//! The glue between the component exports and a mod's [`Mod`] implementation.

use crate::bindings::StepOutput;
use crate::limits::{
    COMMAND_BYTES, OUTPUT_BYTES, PANEL_BYTES, STATE_BYTES, TOASTS_PER_STEP, TOAST_TEXT_BYTES,
};
use crate::{ModCommand, ModSnapshot, ModToast, PanelUpdate, Rng, Tone};

pub use crate::bindings::Quantum;

/// What the host hands `init`.
#[derive(Debug, Clone, Copy)]
pub struct Init<'a> {
    /// The mod's configuration bytes, as the player or operator supplied them.
    pub config: &'a [u8],
    /// The state the mod last returned from `step`, when a save carried it.
    pub saved_state: Option<&'a [u8]>,
}

/// A script mod. One instance lives for the session.
pub trait Mod: Sized + 'static {
    /// Builds the mod. An `Err` disables it at load with that message.
    ///
    /// # Errors
    /// Any message; the host logs it against the mod.
    fn init(init: Init<'_>) -> Result<Self, String>;

    /// Runs once per mod quantum. Commands issued through [`Step::issue`] are
    /// queued in the order issued and land two quanta later.
    ///
    /// # Errors
    /// Any message; the host discards the step's output. It is not a strike.
    fn step(&mut self, step: &mut Step<'_>) -> Result<(), String>;

    /// Rewrites state saved under an older schema.
    ///
    /// # Errors
    /// The default refuses: a mod that changes its state shape overrides this.
    fn migrate(from_schema: u32, state: &[u8]) -> Result<Vec<u8>, String> {
        let _ = state;
        Err(format!("no migration from state schema {from_schema}"))
    }
}

/// One quantum, as the mod sees it.
pub struct Step<'a> {
    /// The quantum index, simulated time and seed.
    pub quantum: Quantum,
    /// The latest snapshot the host refreshed. It is the same value across the
    /// quanta between refreshes.
    pub snapshot: &'a ModSnapshot,
    /// The event batch for this quantum, as raw named msgpack.
    pub events: &'a [u8],
    /// A stream seeded from `quantum.seed`.
    pub rng: Rng,
    commands: Vec<Vec<u8>>,
    panels: Vec<(String, Vec<u8>)>,
    toasts: Vec<Vec<u8>>,
    state: Option<Vec<u8>>,
    /// Bytes of commands, panels and toasts so far, panel ids included, as
    /// the host counts them against [`OUTPUT_BYTES`].
    output_bytes: usize,
}

impl Step<'_> {
    /// Queues `command`.
    ///
    /// # Errors
    /// The command could not be encoded, its encoding is over
    /// [`COMMAND_BYTES`], or it would take this step's output past
    /// [`OUTPUT_BYTES`]. Nothing is queued then.
    pub fn issue(&mut self, command: &ModCommand) -> Result<(), String> {
        let bytes = rmp_serde::to_vec_named(command).map_err(|e| e.to_string())?;
        if bytes.len() > COMMAND_BYTES {
            return Err(format!(
                "the command is {} bytes; the cap is {COMMAND_BYTES}",
                bytes.len()
            ));
        }
        self.output_bytes = self.output_after(bytes.len(), 0)?;
        self.commands.push(bytes);
        Ok(())
    }

    /// Sets the values of panel `id`, a panel the pack's panels file declares.
    /// They land with this step's commands, two quanta later. The last call
    /// for an `id` in a step wins.
    ///
    /// # Errors
    /// The update could not be encoded, its encoding is over [`PANEL_BYTES`],
    /// or it would take this step's output past [`OUTPUT_BYTES`]. The panel
    /// keeps its earlier value then.
    pub fn panel(&mut self, id: &str, update: &PanelUpdate) -> Result<(), String> {
        let bytes = rmp_serde::to_vec_named(update).map_err(|e| e.to_string())?;
        if bytes.len() > PANEL_BYTES {
            return Err(format!(
                "the panel update is {} bytes; the cap is {PANEL_BYTES}",
                bytes.len()
            ));
        }
        match self.panels.iter().position(|(p, _)| p == id) {
            Some(i) => {
                self.output_bytes = self.output_after(bytes.len(), self.panels[i].1.len())?;
                self.panels[i].1 = bytes;
            }
            None => {
                self.output_bytes = self.output_after(id.len() + bytes.len(), 0)?;
                self.panels.push((id.to_owned(), bytes));
            }
        }
        Ok(())
    }

    /// Raises a toast: a short notice every player sees for a few seconds. It
    /// lands with this step's commands, two quanta later. The mod picks the
    /// tone; the game picks the colour and how long it shows.
    ///
    /// # Errors
    /// This step already raised [`TOASTS_PER_STEP`] toasts, `text` is empty or
    /// only whitespace, `text` is over [`TOAST_TEXT_BYTES`], or the toast would
    /// take this step's output past [`OUTPUT_BYTES`]. Nothing is raised then.
    pub fn toast(&mut self, tone: Tone, text: &str) -> Result<(), String> {
        if self.toasts.len() >= TOASTS_PER_STEP {
            return Err(format!(
                "this step already raised {TOASTS_PER_STEP} toasts; the cap is {TOASTS_PER_STEP}"
            ));
        }
        if text.trim().is_empty() {
            return Err("a toast's text is empty".to_owned());
        }
        if text.len() > TOAST_TEXT_BYTES {
            return Err(format!(
                "a toast's text is {} bytes; the cap is {TOAST_TEXT_BYTES}",
                text.len()
            ));
        }
        let toast = ModToast {
            text: text.to_owned(),
            tone,
        };
        let bytes = rmp_serde::to_vec_named(&toast).map_err(|e| e.to_string())?;
        self.output_bytes = self.output_after(bytes.len(), 0)?;
        self.toasts.push(bytes);
        Ok(())
    }

    /// Returns `bytes` as the mod's saved state. The last call in a step wins.
    /// State over [`STATE_BYTES`] is not kept: the step keeps the state it
    /// had, and an error line goes to the mod's log.
    pub fn save_state(&mut self, bytes: Vec<u8>) {
        if bytes.len() > STATE_BYTES {
            let n = bytes.len();
            log(
                3,
                &format!("save_state: {n} bytes not kept; the cap is {STATE_BYTES}"),
            );
            return;
        }
        self.state = Some(bytes);
    }

    /// The output's size with `added` bytes in place of `freed`, or why that is
    /// past [`OUTPUT_BYTES`].
    fn output_after(&self, added: usize, freed: usize) -> Result<usize, String> {
        let total = self.output_bytes - freed + added;
        if total > OUTPUT_BYTES {
            return Err(format!(
                "this step's output would be {total} bytes; the cap is {OUTPUT_BYTES}"
            ));
        }
        Ok(total)
    }

    /// Writes a line to the mod's log. `level` is 0 (trace) to 4 (error).
    pub fn log(&self, level: u8, msg: &str) {
        log(level, msg);
    }

    /// How many commands this step has queued.
    pub fn queued(&self) -> usize {
        self.commands.len()
    }
}

/// Writes a line to the mod's log. `level` is 0 (trace) to 4 (error).
pub fn log(level: u8, msg: &str) {
    crate::host::log(level, msg);
}

/// Holds the mod instance and the last decoded snapshot behind the exports.
pub struct Runtime<M> {
    instance: Option<M>,
    snapshot: Option<ModSnapshot>,
}

impl<M> Runtime<M> {
    /// An empty runtime: no mod instance and no snapshot yet.
    pub const fn new() -> Self {
        Self {
            instance: None,
            snapshot: None,
        }
    }
}

impl<M> Default for Runtime<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Mod> Runtime<M> {
    /// The `init` export.
    ///
    /// # Errors
    /// The mod's own error.
    pub fn init(&mut self, config: &[u8], saved_state: Option<&[u8]>) -> Result<(), String> {
        std::panic::set_hook(Box::new(|info| log(4, &format!("panic: {info}"))));
        self.snapshot = None;
        self.instance = Some(M::init(Init {
            config,
            saved_state,
        })?);
        Ok(())
    }

    /// The `step` export. An empty `snapshot` keeps the one decoded before.
    ///
    /// # Errors
    /// The snapshot did not decode, none has arrived yet, or the mod failed.
    pub fn step(
        &mut self,
        quantum: Quantum,
        snapshot: &[u8],
        events: &[u8],
    ) -> Result<StepOutput, String> {
        if !snapshot.is_empty() {
            // The old copy is freed before the new one is decoded, so a refresh
            // never holds two.
            drop(self.snapshot.take());
            let decoded: ModSnapshot =
                rmp_serde::from_slice(snapshot).map_err(|e| format!("snapshot: {e}"))?;
            self.snapshot = Some(decoded);
        }
        let Some(snapshot) = self.snapshot.as_ref() else {
            return Err("no snapshot has arrived yet".to_owned());
        };
        let Some(instance) = self.instance.as_mut() else {
            return Err("step before init".to_owned());
        };
        let mut step = Step {
            quantum,
            snapshot,
            events,
            rng: Rng::new(quantum.seed),
            commands: Vec::new(),
            panels: Vec::new(),
            toasts: Vec::new(),
            state: None,
            output_bytes: 0,
        };
        instance.step(&mut step)?;
        Ok(StepOutput {
            commands: step.commands,
            panels: step.panels,
            toasts: step.toasts,
            state: step.state,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{Quantum, Step};
    use crate::limits::{
        COMMAND_BYTES, OUTPUT_BYTES, PANEL_BYTES, STATE_BYTES, TOASTS_PER_STEP, TOAST_TEXT_BYTES,
    };
    use crate::{ModCommand, ModSnapshot, PanelUpdate, Rng, Tone, WidgetValue};

    fn envelope() -> ModSnapshot {
        let value = serde_json::json!({
            "mod_api": {"generation": "v1alpha1", "minor": 0},
            "quantum": 0,
            "sim_time_ns": 0,
        });
        let bytes = rmp_serde::to_vec_named(&value).expect("encode");
        rmp_serde::from_slice(&bytes).expect("the envelope decodes")
    }

    fn step(snapshot: &ModSnapshot) -> Step<'_> {
        Step {
            quantum: Quantum {
                index: 0,
                sim_time_ns: 0,
                seed: 0,
            },
            snapshot,
            events: &[],
            rng: Rng::new(0),
            commands: Vec::new(),
            panels: Vec::new(),
            toasts: Vec::new(),
            state: None,
            output_bytes: 0,
        }
    }

    #[test]
    fn toasts_past_the_per_step_cap_are_refused_at_the_call() {
        let snapshot = envelope();
        let mut s = step(&snapshot);
        for _ in 0..TOASTS_PER_STEP {
            s.toast(Tone::Info, "fine")
                .expect("within the per-step cap");
        }
        let err = s.toast(Tone::Info, "one more").expect_err("past the cap");
        assert!(
            err.contains(&format!("the cap is {TOASTS_PER_STEP}")),
            "{err}"
        );
        assert_eq!(s.toasts.len(), TOASTS_PER_STEP);
    }

    #[test]
    fn a_toast_text_must_be_one_to_the_cap_bytes() {
        let snapshot = envelope();
        let mut s = step(&snapshot);
        let err = s.toast(Tone::Good, "").expect_err("empty");
        assert!(err.contains("empty"), "{err}");
        for blank in [" ", "\t\n", "\u{a0}", "\u{2003} \u{3000}"] {
            let err = s.toast(Tone::Good, blank).expect_err("whitespace only");
            assert!(err.contains("empty"), "{blank:?}: {err}");
        }
        let err = s
            .toast(Tone::Good, &"x".repeat(TOAST_TEXT_BYTES + 1))
            .expect_err("over the cap");
        assert!(
            err.contains(&format!("the cap is {TOAST_TEXT_BYTES}")),
            "{err}"
        );
        assert!(s.toasts.is_empty());
        s.toast(Tone::Good, &"x".repeat(TOAST_TEXT_BYTES))
            .expect("exactly the cap fits");
        assert_eq!(s.toasts.len(), 1);
    }

    #[test]
    fn toast_bytes_count_toward_the_output_cap() {
        let snapshot = envelope();
        let mut s = step(&snapshot);
        let per = 16_000;
        let fit = OUTPUT_BYTES / (per + 64);
        for _ in 0..fit {
            s.issue(&buy_rack(per)).expect("within the output cap");
        }
        let before = s.output_bytes;
        s.toast(Tone::Warn, "x").expect("a small toast fits");
        assert!(s.output_bytes > before, "the toast's bytes are counted");
        s.output_bytes = OUTPUT_BYTES - 1;
        let err = s.toast(Tone::Warn, "y").expect_err("past the output cap");
        assert!(err.contains(&format!("the cap is {OUTPUT_BYTES}")), "{err}");
    }

    /// A command whose encoding is `pad` bytes plus a few of framing.
    fn buy_rack(pad: usize) -> ModCommand {
        ModCommand::BuyRack {
            az_id: Some(0),
            sku_name: Some("x".repeat(pad)),
        }
    }

    fn text_panel(pad: usize) -> PanelUpdate {
        PanelUpdate {
            entities: None,
            values: BTreeMap::from([("t".to_owned(), WidgetValue::Text("x".repeat(pad)))]),
        }
    }

    #[test]
    fn a_command_over_the_cap_is_refused_at_issue() {
        let snapshot = envelope();
        let mut s = step(&snapshot);
        let err = s.issue(&buy_rack(COMMAND_BYTES)).expect_err("over the cap");
        assert!(
            err.contains(&format!("the cap is {COMMAND_BYTES}")),
            "{err}"
        );
        s.issue(&buy_rack(10)).expect("a small command fits");
        assert_eq!(s.queued(), 1);
    }

    #[test]
    fn commands_and_panels_past_the_output_cap_are_refused() {
        let snapshot = envelope();
        let mut s = step(&snapshot);
        let per = 16_000;
        let fit = OUTPUT_BYTES / (per + 64);
        for _ in 0..fit {
            s.issue(&buy_rack(per)).expect("within the output cap");
        }
        let err = s.issue(&buy_rack(per)).expect_err("past the output cap");
        assert!(err.contains(&format!("the cap is {OUTPUT_BYTES}")), "{err}");
        let err = s
            .panel("p", &text_panel(per))
            .expect_err("past the output cap");
        assert!(err.contains(&format!("the cap is {OUTPUT_BYTES}")), "{err}");
        assert_eq!(s.queued(), fit);
    }

    #[test]
    fn a_panel_over_the_cap_is_refused_and_a_rewrite_frees_its_bytes() {
        let snapshot = envelope();
        let mut s = step(&snapshot);
        let err = s
            .panel("p", &text_panel(PANEL_BYTES))
            .expect_err("over the cap");
        assert!(err.contains(&format!("the cap is {PANEL_BYTES}")), "{err}");
        assert!(s.panels.is_empty());
        // Rewriting one panel many times stays within the output cap.
        for _ in 0..(2 * OUTPUT_BYTES / 15_000) {
            s.panel("p", &text_panel(15_000))
                .expect("a rewrite replaces");
        }
        assert_eq!(s.panels.len(), 1);
    }

    #[test]
    fn a_state_over_the_cap_is_not_kept() {
        let snapshot = envelope();
        let mut s = step(&snapshot);
        s.save_state(vec![1; 8]);
        s.save_state(vec![0; STATE_BYTES + 1]);
        assert_eq!(
            s.state.as_deref(),
            Some(&[1u8; 8][..]),
            "the last state that fit"
        );
        s.save_state(vec![2; STATE_BYTES]);
        assert_eq!(s.state.as_ref().map(Vec::len), Some(STATE_BYTES));
    }
}
