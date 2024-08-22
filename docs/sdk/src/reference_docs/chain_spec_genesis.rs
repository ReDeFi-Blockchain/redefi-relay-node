//! # What is chain-spec.
//!
//! A chain specification file defines the set of properties that are required to run the node as
//! part of the chain. The chain specification consists of two main parts:
//! - initial state of the runtime,
//! - network / logical properties of the chain, the most important property being the list of
//!   bootnodes.
//!
//! This document describes how the initial state is handled in pallets and runtime, and how to
//! interact with the runtime in order to build the genesis state.
//!
//! For more information on chain specification and its properties, refer to
//! [`sc_chain_spec#from-initial-state-to-raw-genesis`].
//!
//! The initial genesis state can be provided in the following formats:
//!   - full
//!   - patch
//!   - raw
//!
//! Each of the formats is explained in [_chain-spec-format_][`sc_chain_spec#chain-spec-formats`].
//!
//!
//! # `GenesisConfig` for `pallet`
//!
//! Every frame pallet may have its initial state which is defined by the `GenesisConfig` internal
//! struct. It is a regular Rust struct, annotated with the [`pallet::genesis_config`] attribute.
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/src/pallets.rs", pallet_bar_GenesisConfig)]
//!
//! The struct shall be defined within the pallet `mod`, as in the following code:
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/src/pallets.rs", pallet_bar)]
//!
//! The initial state conveyed in  the `GenesisConfig` struct is transformed into state storage
//! items by means of the [`BuildGenesisConfig`] trait, which shall be implemented for the pallet's
//! `GenesisConfig` struct. The [`pallet::genesis_build`] attribute shall be attached to the `impl`
//! block:
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/src/pallets.rs", pallet_bar_build)]
//!
//! `GenesisConfig` may also contain more complicated types, including nested structs or enums, as
//! in the example for `pallet_foo`:
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/src/pallets.rs", pallet_foo_GenesisConfig)]
//!
//! Note that [`serde`] attributes can be used to control how the data
//! structures are stored into JSON. In the following example, the [`sp_core::bytes`] function is
//! used to serialize the `values` field.
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/src/pallets.rs", SomeFooData2)]
//!
//! Please note that fields of `GenesisConfig` may not be directly mapped to storage items. In the
//! following example, the initial struct fields are used to compute (sum) the value that will be
//! stored in the state as `ProcessedEnumValue`:
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/src/pallets.rs", pallet_foo_build)]
//!
//! # `GenesisConfig` for `runtimes`
//!
//! The runtime genesis config struct consists of configs for every pallet. For the [_demonstration
//! runtime_][`chain_spec_guide_runtime`] used in this guide, it consists of `SystemConfig`,
//! `BarConfig`, and `FooConfig`. This structure was automatically generated by a macro and it can
//! be sneak-peeked here: [`RuntimeGenesisConfig`]. For further reading on generated runtime
//! types, refer to [`frame_runtime_types`].
//!
//! The macro automatically adds an attribute that renames all the fields to [`camelCase`]. It is a
//! good practice to add it to nested structures too, to have the naming of the JSON keys consistent
//! across the chain-spec file.
//!
//! ## `Default` for `GenesisConfig`
//!
//! `GenesisConfig` of all pallets must implement the `Default` trait. These are aggregated into
//! the runtime's `RuntimeGenesisConfig`'s `Default`.
//!
//! The default value of `RuntimeGenesisConfig` can be queried by the [`GenesisBuilder::get_preset`]
//! function provided by the runtime with `id:None`.
//!
//! A default value for `RuntimeGenesisConfig` usually is not operational. This is because for some
//! pallets it is not possible to define good defaults (e.g. an initial set of authorities).
//!
//! A default value is a base upon which a patch for `GenesisConfig` is applied. A good description
//! of how it exactly works is provided in [`get_storage_for_patch`] (and also in
//! [`GenesisBuilder::get_preset`]). A patch can be provided as an external file (manually created)
//! or as a built-in runtime preset. More info on presets is in the material to follow.
//!
//! ## Implementing `GenesisBuilder` for runtime
//!
//! The runtime exposes a dedicated runtime API for interacting with its genesis config:
//! [`sp_genesis_builder::GenesisBuilder`]. The implementation shall be provided within
//! the [`sp_api::impl_runtime_apis`] macro, typically making use of some helpers provided:
//! [`build_state`], [`get_preset`].
//! A typical implementation of [`sp_genesis_builder::GenesisBuilder`] looks as follows:
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/src/runtime.rs", runtime_impl)]
//!
//! Please note that two functions are customized: `preset_names` and `get_preset`. The first one
//! just provides a `Vec` of the names of supported presets, while the latter delegates the call
//! to a function that maps the name to an actual preset:
//! [`chain_spec_guide_runtime::presets::get_builtin_preset`]
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/src/presets.rs", get_builtin_preset)]
//!
//! ## Genesis state presets for runtime
//!
//! The runtime may provide many flavors of initial genesis state. This may be useful for predefined
//! testing networks, local development, or CI integration tests. Predefined genesis state may
//! contain a list of pre-funded accounts, predefined authorities for consensus, sudo key, and many
//! others useful for testing.
//!
//! Internally, presets can be provided in a number of ways:
//! - JSON in string form:
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/src/presets.rs", preset_1)]
//! - JSON using runtime types to serialize values:
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/src/presets.rs", preset_2)]
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/src/presets.rs", preset_3)]
//! It is worth noting that a preset does not have to be the full `RuntimeGenesisConfig`, in that
//! sense that it does not have to contain all the keys of the struct. The preset is actually a JSON
//! patch that will be merged with the default value of `RuntimeGenesisConfig`. This approach should
//! simplify maintenance of built-in presets. The following example illustrates a runtime genesis
//! config patch:
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/src/presets.rs", preset_4)]
//!
//! ## Note on the importance of testing presets
//!
//! It is recommended to always test presets by adding tests that convert the preset into the
//! raw storage. Converting to raw storage involves the deserialization of the provided JSON blob,
//! which enforces the verification of the preset. The following code shows one of the approaches
//! that can be taken for testing:
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/src/presets.rs", check_presets)]
//!
//! ## Note on the importance of using the `deny_unknown_fields` attribute
//!
//! It is worth noting that it is easy to make a hard-to-spot mistake, as in the following example
//! ([`FooStruct`] does not contain `fieldC`):
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/src/presets.rs", preset_invalid)]
//! Even though `preset_invalid` contains a key that does not exist, the deserialization of the JSON
//! blob does not fail. The misspelling is silently ignored due to the lack of the
//! [`deny_unknown_fields`] attribute on the [`FooStruct`] struct, which is internally used in
//! `GenesisConfig`.
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/src/presets.rs", invalid_preset_works)]
//!
//! ## Runtime `GenesisConfig` raw format
//!
//! A raw format of genesis config contains just the state's keys and values as they are stored in
//! the storage. This format is used to directly initialize the genesis storage. This format is
//! useful for long-term running chains, where the `GenesisConfig` structure for pallets may be
//! evolving over time. The JSON representation created at some point in time may no longer be
//! deserializable in the future, making a chain specification useless. The raw format is
//! recommended for production chains.
//!
//! For a detailed description of how the raw format is built, please refer to
//! [_chain-spec-raw-genesis_][`sc_chain_spec#from-initial-state-to-raw-genesis`]. Plain and
//! corresponding raw examples of chain-spec are given in
//! [_chain-spec-examples_][`sc_chain_spec#json-chain-specification-example`].
//! The [`chain_spec_builder`] util supports building the raw storage.
//!
//! # Interacting with the tool
//!
//! The [`chain_spec_builder`] util allows interaction with the runtime in order to list or display
//! presets and build the chain specification file. It is possible to use the tool with the
//! [_demonstration runtime_][`chain_spec_guide_runtime`]. To build the required packages, just run
//! the following command:
//! ```ignore
//! cargo build -p staging-chain-spec-builder -p chain-spec-guide-runtime --release
//! ```
//! The `chain-spec-builder` util can also be installed with `cargo install`:
//! ```ignore
//! cargo install staging-chain-spec-builder
//! cargo build -p chain-spec-guide-runtime --release
//! ```
//! Here are some examples in the form of rust tests:
//! ## Listing available preset names:
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/tests/chain_spec_builder_tests.rs", list_presets)]
//! ## Displaying preset with given name
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/tests/chain_spec_builder_tests.rs", get_preset)]
//! ## Building chain-spec using given preset
#![doc = docify::embed!("./src/reference_docs/chain_spec_runtime/tests/chain_spec_builder_tests.rs", generate_chain_spec)]
//!
//! [`RuntimeGenesisConfig`]:
//!     chain_spec_guide_runtime::runtime::RuntimeGenesisConfig
//! [`FooStruct`]:
//!     chain_spec_guide_runtime::pallets::FooStruct
//! [`impl_runtime_apis`]: frame::runtime::prelude::impl_runtime_apis
//! [`build_state`]: frame_support::genesis_builder_helper::build_state
//! [`get_preset`]: frame_support::genesis_builder_helper::get_preset
//! [`pallet::genesis_build`]: frame_support::pallet_macros::genesis_build
//! [`pallet::genesis_config`]: frame_support::pallet_macros::genesis_config
//! [`BuildGenesisConfig`]: frame_support::traits::BuildGenesisConfig
//! [`chain_spec_builder`]: ../../../staging_chain_spec_builder/index.html
//! [`serde`]: https://serde.rs/field-attrs.html
//! [`get_storage_for_patch`]: sc_chain_spec::GenesisConfigBuilderRuntimeCaller::get_storage_for_patch
//! [`GenesisBuilder::get_preset`]: sp_genesis_builder::GenesisBuilder::get_preset
//! [`deny_unknown_fields`]: https://serde.rs/container-attrs.html#deny_unknown_fields
//! [`camelCase`]: https://serde.rs/container-attrs.html#rename_all