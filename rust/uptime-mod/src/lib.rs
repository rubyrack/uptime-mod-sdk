//! Guest SDK for Uptime script mods.
//!
//! A mod is a WebAssembly component built for `wasm32-wasip2` that exports the
//! `uptime-mod` world (`wit/uptime-mod.wit` in this crate). Implement [`Mod`], then
//! call [`export_mod!`] once:
//!
//! ```ignore
//! struct Autoscaler;
//!
//! impl uptime_mod::Mod for Autoscaler {
//!     fn init(_: uptime_mod::Init<'_>) -> Result<Self, String> { Ok(Self) }
//!     fn step(&mut self, step: &mut uptime_mod::Step<'_>) -> Result<(), String> {
//!         step.log(2, "hello");
//!         Ok(())
//!     }
//! }
//!
//! uptime_mod::export_mod!(Autoscaler);
//! ```
//!
//! Each quantum the host passes the latest snapshot as named msgpack. It sends
//! the bytes only when the snapshot refreshed; otherwise it sends an empty list
//! and the SDK hands the mod the copy it already decoded. Commands go back as
//! named msgpack [`ModCommand`]s and land two quanta later, and so do the
//! panel values set with [`Step::panel`].
//!
//! Only what [`Step::save_state`] saves survives a load or a fault: loading a
//! saved game rebuilds the mod through [`Mod::init`] with that state, as a
//! fault does. A mod whose behaviour depends only on its config, its saved
//! state and what each step hands it plays the same after a load. The first
//! step after a load gets the whole snapshot again, with the fuel to decode
//! it, so a step whose own work alone is near the fuel budget may end
//! differently.

#[rustfmt::skip]
#[allow(clippy::all, clippy::pedantic)]
pub mod bindings {
    wit_bindgen::generate!({
        path: "wit/uptime-mod.wit",
        world: "uptime-mod",
        pub_export_macro: true,
        default_bindings_module: "uptime_mod::bindings",
        // wit-bindgen emits the package segment `mod` of `uptime:mod` unescaped,
        // which is a Rust keyword. The one import is declared by hand in `host`.
        with: { "uptime:mod/host@0.1.0": crate::host },
    });
}

pub mod events;
mod host;
/// The caps the host holds a step's output to. [`Step`] refuses a command,
/// panel or state past them at the call, before the host would discard it.
pub mod limits {
    include!("../wit/limits.rs");
}
mod generated {
    #[rustfmt::skip]
    pub mod commands;
    #[rustfmt::skip]
    pub mod panels;
    #[rustfmt::skip]
    pub mod snapshot;
}
mod rng;
mod runtime;
#[cfg(feature = "testing")]
pub mod testing;
mod units;

pub use generated::commands::{self, CableEndpoint, ModCommand};
pub use generated::panels::{
    self, EntityUpdate, ModToast, PanelUpdate, Tone, UiValue, WidgetValue,
};
pub use generated::snapshot::{self, ModSnapshot};
pub use rng::Rng;
pub use runtime::{Init, Mod, Quantum, Runtime, Step};
pub use units::{units, HostUnits};

/// Exports `$ty` as the component's `uptime-mod` world.
///
/// `$ty` must implement [`Mod`]. Use it once per mod crate.
#[macro_export]
#[rustfmt::skip]
macro_rules! export_mod {
    ($ty:ty) => {
        const _: () = {
            thread_local! {
                static RUNTIME: ::std::cell::RefCell<$crate::Runtime<$ty>> =
                    const { ::std::cell::RefCell::new($crate::Runtime::new()) };
            }

            struct Glue;

            impl $crate::bindings::Guest for Glue {
                fn init(
                    config: ::std::vec::Vec<u8>,
                    saved_state: ::std::option::Option<::std::vec::Vec<u8>>,
                ) -> ::std::result::Result<(), ::std::string::String> {
                    RUNTIME.with(|rt| rt.borrow_mut().init(&config, saved_state.as_deref()))
                }

                fn step(
                    q: $crate::bindings::Quantum,
                    snapshot: ::std::vec::Vec<u8>,
                    events: ::std::vec::Vec<u8>,
                ) -> ::std::result::Result<$crate::bindings::StepOutput, ::std::string::String>
                {
                    RUNTIME.with(|rt| rt.borrow_mut().step(q, &snapshot, &events))
                }

                fn migrate(
                    from_schema: u32,
                    state: ::std::vec::Vec<u8>,
                ) -> ::std::result::Result<::std::vec::Vec<u8>, ::std::string::String> {
                    <$ty as $crate::Mod>::migrate(from_schema, &state)
                }
            }

            $crate::bindings::export!(Glue with_types_in $crate::bindings);
        };
    };
}
