//! <div align=center>
//!   <img src="https://data.dilla.io/dilla.png" alt="" width=320>
//!   <p><strong>Share your design system in a tiny universal package.</strong></p>
//! </div>
//!
//! Dilla is a fast but minimal WASM builder based on the syntax and behavior
//! of the [Jinja2](https://jinja.palletsprojects.com/) implemented on top of
//! [Minijinja](https://docs.rs/minijinja/latest/minijinja). The goal is to
//! be able to pack your design system into a <strong>universal</strong>
//! package, executable through a simple <strong>declarative API</strong>, for
//! both server side and headless rendering.
//!
//! To know more about Dilla visit our website [dilla.io](https://dilla.io).
//!
//! ---
//!
//! **Dilla CLI for renderer and describer.**
//!
//! Simple utility to use and test Dilla without WASM build.
use clap::{Parser, Subcommand};
use dilla_describer::describe as dilla_describe;
use dilla_renderer::{render as dilla_render, DESIGN_SYSTEM};
use html_minifier::HTMLMinifier;
use html_parser::Dom;
use std::fs;
use std::path::PathBuf;
use std::str;
use std::time::Instant;

#[doc(hidden)]
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Cli entrypoint to allow commands.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Enum representing the possible commands for the CLI.
///
/// The `Commands` enum provides two subcommands: `Render` and `Describe`.
/// The `Render` subcommand is used for rendering from a payload, and accepts various options
/// such as payload file, output format, write location, raw flag, and quiet flag.
/// The `Describe` subcommand is used for performing introspection, and accepts options for
/// the artefact identifier and ID.
///
/// # Note
///
/// The specific implementation details of the command-line parsing and execution are omitted here.
/// Please refer to the full code implementation for more details.
#[derive(Subcommand)]
enum Commands {
    /// Render from a Payload
    Render {
        /// Payload file path, currently support only json files
        #[arg(value_name = "PAYLOAD_FILE")]
        payload: PathBuf,
        /// Optional output format, default to 'full'
        #[arg(
            short,
            long,
            default_value_t = String::from("full"),
            value_parser = clap::builder::PossibleValuesParser::new(["_logs", "_test", "_test_full", "full", "json", "dom_json"])
        )]
        mode: String,
        /// Optional, output result to a file instead of print
        #[clap(short, long, value_name = "FILE")]
        write: Option<String>,
        /// Do not prettify the output, default 'false'
        #[clap(short, long, default_value_t = false)]
        raw: bool,
        /// Print less messages, default 'false'
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
    },
    /// Introspection query for a Design System
    Describe {
        /// Artefact to look for, ie: components, styles, libraries...
        #[clap(default_value_t = String::from(""))]
        artefact: String,
        /// Optional artefact id to look for
        #[clap(default_value_t = String::from(""))]
        id: String,
    },
    Info {},
}

/// Rust Command-line Interface (CLI) for performing rendering and introspection tasks.
///
/// This module provides functionality for executing commands related to rendering and introspection. It
/// parses command line arguments and dispatches the appropriate actions based on the specified command.
/// The supported commands are Render and Describe.
///
/// * Usage
///
/// The `Commands::Render` variant renders the payload file to the requested output format,
/// writing to a file if `write` is specified.
///
/// The `Commands::Describe` variant calls the `describer::describe` function to provide
/// introspection for the given artefact and ID.
/// Rust Command-line Interface (CLI) for performing rendering and introspection tasks.
fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Render {
            payload,
            mode,
            write,
            raw,
            quiet,
        } => {
            render(payload, mode, write, raw.to_owned(), quiet.to_owned());
        }
        Commands::Describe { artefact, id } => describe(artefact, id),
        Commands::Info {} => info(),
    }
}

/// Return the definition of a Design System.
///
/// # Arguments
///
/// * `artefact` - An artefact value, ie: components, styles, libraries...
/// * `id` - An artefact id to return.
fn describe(artefact: &str, id: &str) {
    let result = dilla_describe(artefact, id);
    println!("{}", result);
}

/// Renders the specified payload using the given mode and outputs the result.
///
/// # Arguments
///
/// * `payload` - A `PathBuf` containing the path to the payload file.
/// * `mode` - A string specifying the rendering mode.
/// * `write` - An optional `String` specifying the path to write the output. If provided, the output will be written to the specified file.
/// * `raw` - A boolean indicating whether to output the result without any formatting.
/// * `quiet` - A boolean indicating whether to suppress any additional output and only display the result.
fn render(payload: &PathBuf, mode: &str, write: &Option<String>, raw: bool, quiet: bool) {
    let is_json = payload.display().to_string().ends_with(".json");
    if !is_json {
        return eprintln!("[Error] Payload is not a json file!");
    }

    let payload = fs::read_to_string(payload).expect("Failed to read payload file!");

    let (do_dom, format_output) = match mode {
        "dom_json" => (true, "_test_full"),
        _ => (false, mode),
    };

    let now = Instant::now();
    let mut result = dilla_render(&payload, format_output).expect("Dilla rendering failed!");
    let render = now.elapsed().as_micros() as f32 / 1000.0;

    if !raw && !do_dom {
        result = format_minify(result);
    } else if do_dom {
        result = format_dom_json(result);
    }

    let message = format!(
        "Dilla CLI v{VERSION} | ds: {}, minify: {}, render: {:.2} ms",
        DESIGN_SYSTEM, !raw, render
    );

    if let Some(file_output) = write {
        fs::write(file_output, result).expect("Failed to write file");
        println!("File output generated: {}", file_output);
    } else if format_output == "_logs" {
        println!("{}", message);
    } else if quiet {
        println!("{}", result);
    } else {
        println!("<!-- {} -->\n{}\n<!-- {} -->", message, result, message);
    }
}

#[doc(hidden)]
fn info() {
    println!("Dilla CLI {DESIGN_SYSTEM} v{VERSION}");
}

#[doc(hidden)]
fn format_dom_json(s: String) -> String {
    Dom::parse(&s)
        .expect("Failed to parse DOM")
        .to_json_pretty()
        .expect("Failed to format DOM as JSON")
}

#[doc(hidden)]
fn format_minify(s: String) -> String {
    let mut html_minifier = HTMLMinifier::new();
    html_minifier.set_remove_comments(false);
    html_minifier.digest(s.clone()).unwrap();

    str::from_utf8(html_minifier.get_html())
        .unwrap()
        .to_string()
}
