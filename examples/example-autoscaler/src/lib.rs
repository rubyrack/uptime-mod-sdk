//! Example autoscaler (`com.uptime.example-autoscaler`).
//!
//! Policy, one pass per quantum:
//!
//! 1. Accept every prospect the fleet can serve now, at most [`MAX_ACCEPTS`]
//!    per quantum, in snapshot order.
//! 2. When free vCPU or free memory falls under [`Config::low_pct`] percent of
//!    the fleet total, buy one host of [`Config::sku`], provided cash stays
//!    above [`Config::cash_reserve`] and [`Config::cooldown_quanta`] quanta have
//!    passed since the last purchase.
//!
//! A command issued at quantum `q` lands at `q + 2`, and a snapshot reflects it
//! once its own quantum reaches that. Until then a prospect already accepted,
//! and a host already bought, are not ordered again; after it, a prospect
//! still listed is accepted again.
//!
//! The mod keeps four things between steps, all in its saved state: the orders
//! in flight, the last purchase quantum, the pause switch and the last ten
//! decisions. A mod rebuilt from that state resumes where it left off. If the
//! saved state does not decode, the mod logs it and starts paused.
//!
//! Two panels show the policy and the decisions. The pause toggle on the policy
//! panel stops the mod issuing commands; the mod reads it from the UI input
//! events. It skips any event or UI value of a kind it does not know.
//!
//! A config value of nil selects the field's default. An integer field takes
//! 0 through `i64::MAX`, and so does `cash_reserve` when given an integer.

use std::fmt;

use serde::de::{self, Deserializer, IgnoredAny, Visitor};
use serde::{Deserialize, Serialize};
use uptime_mod::snapshot::CapacityView;
use uptime_mod::{
    export_mod, Init, Mod, ModCommand, PanelUpdate, Step, Tone, UiValue, WidgetValue,
};

/// Most prospects accepted in one quantum.
pub const MAX_ACCEPTS: usize = 8;

/// Quanta between a command's issue and its landing.
const LANDING_QUANTA: u64 = 2;

/// Decisions kept in state and shown.
const MAX_DECISIONS: usize = 10;

/// The SKUs the `sku` config key names: its enum value and the catalog name.
const SKUS: [(&str, &str); 10] = [
    ("balanced_x86_1u", "Balanced x86 1U"),
    ("balanced_x86_2u", "Balanced x86 2U"),
    ("arm_1u", "ARM 1U"),
    ("arm_2u", "ARM 2U"),
    ("compute_1u", "Compute 1U"),
    ("compute_2u", "Compute 2U"),
    ("memory_1u", "Memory 1U"),
    ("memory_2u", "Memory 2U"),
    ("storage_1u", "Storage 1U"),
    ("storage_2u", "Storage 2U"),
];

/// The mod's configuration. Every field has a default, and an empty config
/// selects them all.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// The catalog name of the SKU `BuyHost` orders.
    pub sku: String,
    /// Buy when free vCPU or memory is under this percent of the total.
    pub low_pct: u64,
    /// Never buy if it would leave less cash than this.
    pub cash_reserve: f64,
    /// Quanta to wait between purchases.
    pub cooldown_quanta: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sku: SKUS[0].1.to_owned(),
            low_pct: 20,
            cash_reserve: 5_000.0,
            cooldown_quanta: 8,
        }
    }
}

/// A config as it arrives: a missing or nil field is `None`.
#[derive(Default, Deserialize)]
#[serde(default)]
struct RawConfig {
    sku: Option<String>,
    #[serde(deserialize_with = "count")]
    low_pct: Option<u64>,
    #[serde(deserialize_with = "reserve")]
    cash_reserve: Option<f64>,
    #[serde(deserialize_with = "count")]
    cooldown_quanta: Option<u64>,
}

impl TryFrom<RawConfig> for Config {
    type Error = String;

    fn try_from(raw: RawConfig) -> Result<Self, String> {
        let d = Config::default();
        let sku = match raw.sku {
            None => d.sku,
            Some(slug) => SKUS
                .iter()
                .find(|(s, _)| *s == slug)
                .map(|(_, name)| (*name).to_owned())
                .ok_or_else(|| format!("sku {slug:?} is not one of the listed SKUs"))?,
        };
        Ok(Self {
            sku,
            low_pct: raw.low_pct.unwrap_or(d.low_pct),
            cash_reserve: raw.cash_reserve.unwrap_or(d.cash_reserve),
            cooldown_quanta: raw.cooldown_quanta.unwrap_or(d.cooldown_quanta),
        })
    }
}

/// An integer in `0..=i64::MAX`, or nil.
struct CountVisitor;

impl Visitor<'_> for CountVisitor {
    type Value = Option<u64>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a non-negative integer")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
        i64::try_from(v)
            .map(|_| Some(v))
            .map_err(|_| E::custom("integer above i64::MAX"))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
        u64::try_from(v)
            .map(Some)
            .map_err(|_| E::custom("negative integer"))
    }
}

fn count<'de, D: Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error> {
    d.deserialize_any(CountVisitor)
}

/// A float, an integer in `0..=i64::MAX`, or nil.
struct ReserveVisitor;

impl Visitor<'_> for ReserveVisitor {
    type Value = Option<f64>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a number")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
        Ok(Some(v))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
        CountVisitor.visit_u64(v).map(|n| n.map(|n| n as f64))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
        CountVisitor.visit_i64(v).map(|n| n.map(|n| n as f64))
    }
}

fn reserve<'de, D: Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
    d.deserialize_any(ReserveVisitor)
}

/// One decision the mod made, as the decisions table shows it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Decision {
    action: String,
    /// The command's position in its step's output.
    index: u32,
    /// The quantum whose step issued it.
    quantum: u64,
    /// `pending` until the receipt lands, then `ok` or the failure code.
    result: String,
    target: String,
}

/// The mod's saved state, schema 1. Fields are declared in key order so the
/// encoding matches the Lua port's, which writes its keys sorted.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Saved {
    #[serde(default)]
    decisions: Vec<Decision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_buy_quantum: Option<u64>,
    #[serde(default)]
    paused: bool,
    /// `(prospect id, quantum issued)` of each accept still in flight, by id.
    #[serde(default)]
    pending: Vec<(u64, u64)>,
}

/// The state schema 0 saved: the quantum of the last purchase.
#[derive(Deserialize)]
struct SavedV0 {
    last_buy_quantum: u64,
}

/// The parts of an event batch the mod reads.
#[derive(Deserialize)]
struct Batch {
    #[serde(default)]
    events: Vec<Lenient<Event>>,
}

/// A value of a kind the mod reads, or any other, which it skips. A later API
/// minor may add event and UI value kinds, and one of those must not fail the
/// whole batch.
#[derive(Deserialize)]
#[serde(untagged)]
enum Lenient<T> {
    Known(T),
    Other(IgnoredAny),
}

#[derive(Deserialize)]
enum Event {
    Receipt(Receipt),
    UiInput(Input),
}

#[derive(Deserialize)]
struct Receipt {
    quantum: u64,
    index: u32,
    ok: bool,
    #[serde(default)]
    code: String,
}

#[derive(Deserialize)]
struct Input {
    player: u32,
    panel: String,
    widget: String,
    value: Lenient<Value>,
}

#[derive(Deserialize)]
enum Value {
    Bool(bool),
}

/// The autoscaler.
pub struct Autoscaler {
    config: Config,
    state: Saved,
    /// Why the saved state was set aside, logged on the first step.
    state_note: Option<String>,
}

/// True while a command issued at `issued` has not reached a snapshot taken at
/// `snapshot_quantum`.
fn in_flight(issued: u64, snapshot_quantum: u64) -> bool {
    snapshot_quantum < issued.saturating_add(LANDING_QUANTA)
}

/// True when a resource has a known total and less than `low_pct` percent of
/// it is free.
fn is_low(free: u64, total: u64, low_pct: u64) -> bool {
    total > 0 && free.saturating_mul(100) < total.saturating_mul(low_pct)
}

fn capacity_is_low(c: &CapacityView, low_pct: u64) -> bool {
    is_low(c.free_vcpu_milli, c.total_vcpu_milli, low_pct)
        || is_low(c.free_mem_mb, c.total_mem_mb, low_pct)
}

// A command literal names every optional field, so a field added in a later
// API minor needs setting in these two helpers only.
fn accept(prospect_id: u64) -> ModCommand {
    ModCommand::AcceptProspect {
        prospect_id,
        isolate_dedicate: None,
        isolate_mode: None,
        isolate_segment_id: None,
    }
}

fn buy(sku_name: &str) -> ModCommand {
    ModCommand::BuyHost {
        az_id: None,
        sku_name: sku_name.to_owned(),
    }
}

fn stat(value: String, detail: &str, tone: Tone) -> WidgetValue {
    WidgetValue::Stat {
        detail: detail.to_owned(),
        tone,
        value,
    }
}

impl Autoscaler {
    /// Applies the receipts and UI inputs of one batch, in batch order.
    fn read_events(&mut self, events: &[u8], step: &Step<'_>) {
        if events.is_empty() {
            return;
        }
        let batch = match rmp_serde::from_slice::<Batch>(events) {
            Ok(b) => b,
            Err(e) => {
                step.log(3, &format!("events: {e}"));
                return;
            }
        };
        for event in batch.events {
            match event {
                Lenient::Known(Event::Receipt(r)) => self.settle(&r),
                // Only the host (player 0) can pause; a guest's press is ignored.
                Lenient::Known(Event::UiInput(Input {
                    player: 0,
                    panel,
                    widget,
                    value: Lenient::Known(Value::Bool(on)),
                })) if panel == "policy" && widget == "paused" => self.state.paused = on,
                Lenient::Known(Event::UiInput(_)) | Lenient::Other(_) => {}
            }
        }
    }

    /// Fills in the result of the decision a receipt answers.
    fn settle(&mut self, r: &Receipt) {
        let open = self
            .state
            .decisions
            .iter_mut()
            .find(|d| d.quantum == r.quantum && d.index == r.index && d.result == "pending");
        if let Some(d) = open {
            d.result = if r.ok {
                "ok".to_owned()
            } else if r.code.is_empty() {
                "refused".to_owned()
            } else {
                r.code.clone()
            };
        }
    }

    fn record(&mut self, action: &str, quantum: u64, index: usize, target: String) {
        let decisions = &mut self.state.decisions;
        decisions.push(Decision {
            action: action.to_owned(),
            index: u32::try_from(index).unwrap_or(u32::MAX),
            quantum,
            result: "pending".to_owned(),
            target,
        });
        if decisions.len() > MAX_DECISIONS {
            decisions.remove(0);
        }
    }

    /// The quantum from which the next purchase is allowed, when there was one.
    fn next_buy(&self) -> Option<u64> {
        self.state
            .last_buy_quantum
            .map(|last| last.saturating_add(self.config.cooldown_quanta))
    }

    fn policy_panel(&self, now: u64) -> PanelUpdate {
        let c = &self.config;
        let (next, mut tone, mut detail) = match self.next_buy() {
            None => ("now".to_owned(), Tone::Good, "next buy allowed at quantum"),
            Some(at) => (
                at.to_string(),
                if now >= at { Tone::Good } else { Tone::Info },
                "next buy allowed at quantum",
            ),
        };
        if self.state.paused {
            tone = Tone::Warn;
            detail = "paused: the autoscaler issues nothing";
        }
        let values = [
            ("sku", stat(c.sku.clone(), "host bought", Tone::Neutral)),
            (
                "low_pct",
                stat(
                    format!("{}%", c.low_pct),
                    "buy under this free share",
                    Tone::Neutral,
                ),
            ),
            (
                "cash_reserve",
                stat(
                    format!("${:.2}", c.cash_reserve),
                    "cash to keep",
                    Tone::Neutral,
                ),
            ),
            (
                "cooldown",
                stat(
                    format!("{} quanta", c.cooldown_quanta),
                    "between purchases",
                    Tone::Neutral,
                ),
            ),
            ("next_buy", stat(next, detail, tone)),
            (
                "paused",
                WidgetValue::Control(UiValue::Bool(self.state.paused)),
            ),
        ];
        PanelUpdate {
            entities: None,
            values: values.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(),
        }
    }

    fn decisions_panel(&self) -> PanelUpdate {
        let rows = self
            .state
            .decisions
            .iter()
            .rev()
            .map(|d| {
                vec![
                    d.quantum.to_string(),
                    d.action.clone(),
                    d.target.clone(),
                    d.result.clone(),
                ]
            })
            .collect();
        PanelUpdate {
            entities: None,
            values: [("recent".to_owned(), WidgetValue::Rows(rows))].into(),
        }
    }
}

impl Mod for Autoscaler {
    fn init(init: Init<'_>) -> Result<Self, String> {
        let config = if init.config.is_empty() {
            Config::default()
        } else {
            rmp_serde::from_slice::<RawConfig>(init.config)
                .map_err(|e| e.to_string())
                .and_then(Config::try_from)
                .map_err(|e| format!("config: {e}"))?
        };
        // A state that does not decode starts the mod paused.
        let (state, state_note) = match init.saved_state.map(rmp_serde::from_slice::<Saved>) {
            None => (Saved::default(), None),
            Some(Ok(state)) => (state, None),
            Some(Err(e)) => (
                Saved {
                    paused: true,
                    ..Saved::default()
                },
                Some(format!("saved state did not decode ({e}); starting paused")),
            ),
        };
        Ok(Self {
            config,
            state,
            state_note,
        })
    }

    fn step(&mut self, step: &mut Step<'_>) -> Result<(), String> {
        let now = step.quantum.index;
        let seen = step.snapshot.quantum;

        if let Some(note) = self.state_note.take() {
            step.log(4, &note);
        }

        self.read_events(step.events, step);

        // The snapshot refreshes every few quanta and the quanta between see
        // the same prospects and the same shortage. An order is held back
        // until a snapshot can show it landed.
        self.state
            .pending
            .retain(|&(_, issued)| in_flight(issued, seen));

        if !self.state.paused {
            let fresh: Vec<u64> = step
                .snapshot
                .prospects
                .iter()
                .flatten()
                .filter(|p| p.fit_can_serve_now && !p.fit_over_capacity)
                .map(|p| p.id)
                .filter(|id| {
                    self.state
                        .pending
                        .binary_search_by_key(id, |&(held, _)| held)
                        .is_err()
                })
                .take(MAX_ACCEPTS)
                .collect();
            for prospect_id in fresh {
                let index = step.queued();
                step.issue(&accept(prospect_id))?;
                self.state.pending.push((prospect_id, now));
                self.record("accept", now, index, format!("prospect {prospect_id}"));
            }
            self.state.pending.sort_unstable_by_key(|&(id, _)| id);

            let cooled = self
                .state
                .last_buy_quantum
                .is_none_or(|last| now.saturating_sub(last) >= self.config.cooldown_quanta);
            let affordable = step
                .snapshot
                .topbar
                .as_ref()
                .is_some_and(|t| t.cash_dollars >= self.config.cash_reserve);
            let low = step
                .snapshot
                .capacity
                .as_ref()
                .is_some_and(|c| capacity_is_low(c, self.config.low_pct));
            let landed = self
                .state
                .last_buy_quantum
                .is_none_or(|last| !in_flight(last, seen));
            if cooled && landed && affordable && low {
                let index = step.queued();
                step.issue(&buy(&self.config.sku))?;
                self.state.last_buy_quantum = Some(now);
                self.record("buy", now, index, self.config.sku.clone());
                step.log(2, "capacity low: buying a host");
            }
        }

        let saved = rmp_serde::to_vec_named(&self.state).map_err(|e| e.to_string())?;
        step.save_state(saved);
        step.panel("policy", &self.policy_panel(now))?;
        step.panel("decisions", &self.decisions_panel())?;
        Ok(())
    }

    fn migrate(from_schema: u32, state: &[u8]) -> Result<Vec<u8>, String> {
        match from_schema {
            0 => {
                let old: SavedV0 = rmp_serde::from_slice(state).map_err(|e| e.to_string())?;
                let new = Saved {
                    last_buy_quantum: Some(old.last_buy_quantum),
                    ..Saved::default()
                };
                rmp_serde::to_vec_named(&new).map_err(|e| e.to_string())
            }
            n => Err(format!("no migration from state schema {n}")),
        }
    }
}

export_mod!(Autoscaler);
