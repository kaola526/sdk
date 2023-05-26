// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the Aleo library.

// The Aleo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Aleo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Aleo library. If not, see <https://www.gnu.org/licenses/>.

//!
//![![Crates.io](https://img.shields.io/crates/v/aleo-wasm.svg?color=neon)](https://crates.io/crates/aleo-wasm)
//! [![Authors](https://img.shields.io/badge/authors-Aleo-orange.svg)](https://aleo.org)
//! [![License](https://img.shields.io/badge/License-GPLv3-blue.svg)](./LICENSE.md)
//!
//! [![github]](https://github.com/AleoHQ/aleo)&ensp;[![crates-io]](https://crates.io/crates/aleo-wasm)&ensp;[![docs-rs]](https://docs.rs/aleo-wasm/latest/aleo-wasm/)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//!
//! # Aleo Wasm
//!
//! Aleo JavaScript and WebAssembly bindings for building zero-knowledge web applications.
//!
//! `Rust` compiles easily to `WebAssembly` but creating the bindings necessary to use the compiled WebAssembly binaries
//! from other languages such as JavaScript is a challenging task. `wasm-bindgen` is a tool that simplifies this process by
//! auto-generating  JavaScript bindings to Rust code which has been compiled into WebAssembly.
//!
//! This crate uses `wasm-bindgen` to create JavaScript bindings to Aleo code so that it can be used in directly within
//! `web browsers` and `NodeJS`. The code within this crate can be used to create zero knowledge proof directly within
//! the browser.
//!
//! Functionality exposed by this crate includes:
//! * Aleo account management objects
//! * Aleo primitives such as `Records`, `Programs`, and `Transactions` and their associated helper methods
//! * Objects and methods for authoring, deploying, and interacting with Aleo programs
//!
//! More information on these concepts can be found at the [Aleo Developer Hub](https://developer.aleo.org/concepts).
//!
//! ## Usage
//! The [wasm-pack](https://crates.io/crates/wasm-pack) tool is used to compile the Rust code in this crate into JavaScript
//! modules which can be imported into other JavaScript projects.
//!
//! #### Install Wasm-Pack
//! To install [wasm-pack](https://crates.io/crates/wasm-pack) use the following command:
//! ```bash
//! curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
//! ```
//!
//! ### Build Instructions
//! To build a JavaScript module using [wasm-pack](https://crates.io/crates/wasm-pack):
//! ```bash
//! wasm-pack build --target <target> --out-dir <out-dir> -- --features <crate-features>
//! ```
//!
//! When a module is built, it is output in a directory with the default name: `pkg` (which can be changed as necessary
//! using the `--out-dir` flag). This folder can then be imported directly as a JavaScript module to your application!
//!
//! There are 3 possible JavaScript modules that [wasm-pack](https://crates.io/crates/wasm-pack) can be used to generate based
//! on this crate.
//! 1. **NodeJS module:** Used to build NodeJS applications.
//! 2. **Single-Threaded browser module:** Used to build browser-based web applications.
//! 3. **Multi-Threaded browser module:** Used to build browser-based web applications which use web-worker based
//! multi-threading to achieve significant performance increases.
//!
//! These 3 modules and how to build them are explained in more detail below.
//!
//! ### 1. NodeJS Module
//!
//! This module has the features of the NodeJS environment built in. It is single-threaded and unfortunately cannot yet be
//! used to generate Aleo program executions or deployments due to current Aleo protocol limitations. It is planned to make
//! this module usable for program execution and deployment in the future, however.
//!
//! #### Build Instructions
//! ```bash
//! wasm-pack build --target nodejs -- --features "serial"
//! ```
//!
//! ### 2. Single-Threaded browser module
//!
//! This module is very similar to the NodeJS module, however it is built to make use browser-based JavaScript environments
//! and can be used for program execution and deployment. Because this module single-threaded, if used for program execution
//! or deployment it will be quite slow and will cause the browser to hang if these operations run in the main thread. This
//! blocking can be avoided by running it on a web-worker.
//!
//! #### Build Instructions
//! ```bash
//! wasm-pack build --release --target web
//! ```
//!
//! If you are intending to use this for program execution or deployment, it is recommended to build
//! with maximum or close to maximum memory allocation (4gb for wasm).
//!
//! ```bash
//! export RUSTFLAGS='-C link-arg=--max-memory=4294967296' wasm-pack build --release --target web
//! ````
//!
//! ### 3. Multi-Threaded browser module
//!
//! This module is also built for browser-based JavaScript environments, however it is built to make use of Rust-native
//! threading via web-workers (using the approach outlined in the `rayon-wasm-bindgen` crate). It is the most complex to use,
//! but it will run significantly faster when performing Aleo program executions and deployments and should be the choice for
//! performance-critical applications.
//!
//! To build with threading enabled, it is necessary to use `nightly` Rust and to set certain `RUSTFLAGS` to enable the
//! necessary threading features. The build command is shown below.
//! ```bash
//! # Set rustflags to enable atomics,
//! # bulk-memory, and mutable-globals.
//! # Also, set the maximum memory to
//! # 4294967296 bytes (4GB).
//! export RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals -C link-arg=--max-memory=4294967296'
//!
//! # Use rustup to run the following commands
//! # with the nightly version of Rust.
//! rustup run nightly \
//!
//! # Use wasm-pack to build the project.
//! wasm-pack build --release --target web --out-dir pkg-parallel \
//!
//! # Specify the 'parallel' feature for
//! # multi-threading and the 'browser'
//! # feature to enable program execution
//! -- --features "parallel, browser"
//! ```
//!
//! ## Testing
//!
//! Run tests in NodeJS
//! ```bash
//! wasm-pack test --node
//! ```
//!
//! Run tests in a browser
//! ```bash
//! wasm-pack test --headless --[firefox/chrome/safari]
//! ```
//!
//! ## Building Web Apps
//!
//! Further documentation and tutorials as to how to use the modules built from this crate to build web apps  will be built
//! in the future and linked here. However - in the meantime, the [aleo.tools](https://aleo.tools) react website is a good
//! example of how to use these modules to build a web app. Its source code can be found in the
//! [Aleo SDK](https://github.com/AleoHQ/sdk) repo in the `website` folder.
//!

pub mod account;
pub use account::*;

pub mod programs;
pub use programs::*;

pub mod record;
pub use record::*;

pub(crate) mod types;

use wasm_bindgen::prelude::*;

#[cfg(feature = "parallel")]
pub use wasm_bindgen_rayon::init_thread_pool;

// Facilities for cross-platform logging in both web browsers and nodeJS
#[wasm_bindgen]
extern "C" {
    // Log a &str the console in the browser or console.log in nodejs
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
