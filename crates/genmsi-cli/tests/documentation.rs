//! The document of the CLI must describe the CLI.
//!
//! The subcommands are read from the binary's own help —the way a reader
//! discovers them— and the synopsis of `docs/cli.md` is checked against that
//! list. A subcommand that the document does not mention fails here, instead
//! of failing in the reader's head; and a subcommand the document mentions
//! that the binary does not have fails too, because it is a name the reader
//! would try and not find.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn document() -> String {
    std::fs::read_to_string(repo_root().join("docs/cli.md")).expect("docs/cli.md is readable")
}

/// The subcommands, as the binary's help lists them.
fn subcommands() -> Vec<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_genmsi"))
        .arg("--help")
        .output()
        .expect("the genmsi binary runs");
    let help = String::from_utf8(output.stdout).expect("the help is UTF-8");
    let mut names = Vec::new();
    let mut in_commands = false;
    for line in help.lines() {
        if line.starts_with("Commands:") {
            in_commands = true;
            continue;
        }
        if !in_commands {
            continue;
        }
        if line.trim().is_empty() || !line.starts_with("  ") {
            break;
        }
        // `help` is a subcommand of clap, not of the product.
        if let Some(name) = line.split_whitespace().next() {
            if name != "help" {
                names.push(name.to_string());
            }
        }
    }
    names
}

/// The synopsis of the document: the block that lists the subcommands.
fn synopsis() -> String {
    let doc = document();
    let start = doc
        .find("genmsi <command>")
        .expect("docs/cli.md has a synopsis");
    let rest = &doc[start..];
    let end = rest.find("```").expect("the synopsis is a fenced block");
    rest[..end].to_string()
}

/// The subcommands the synopsis names, in the order it names them.
fn documented() -> Vec<String> {
    synopsis()
        .lines()
        .filter_map(|line| line.trim().strip_prefix("genmsi "))
        .filter_map(|rest| rest.split_whitespace().next())
        // The first line of the synopsis is the usage, not a subcommand.
        .filter(|name| !name.starts_with('<'))
        .map(|name| name.to_string())
        .collect()
}

#[test]
fn the_synopsis_and_the_binary_name_the_same_subcommands() {
    let documented = documented();
    let binary = subcommands();
    assert!(
        !binary.is_empty(),
        "the binary's help lists no subcommand: the help moved"
    );
    assert_eq!(
        documented, binary,
        "the synopsis of docs/cli.md and the binary list different subcommands"
    );
}

#[test]
fn the_synopsis_names_a_subcommand_of_the_product() {
    for name in ["lint", "simulate", "trace", "gpss", "layout", "run-all", "mcp"] {
        assert!(
            documented().iter().any(|documented| documented == name),
            "docs/cli.md does not name the subcommand `{name}`"
        );
    }
}
