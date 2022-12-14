use std::path::PathBuf;

use clap::{arg, Args};
use human_duration::human_duration;
use rpassword::{self, prompt_password};

use crate::{
    actions::{self, ActionError, ActionResult},
    log,
};

#[derive(Args, Clone)]
/// Encrypt specified file/directory using the passed key
pub struct Command {
    /// Paths of the file(s)/directory(ies) to encrypt
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Set output path (only supported when one input path
    /// provided)
    #[arg(short, long)]
    output_path: Option<PathBuf>,

    /// Whether to overwrite the output file/directory
    #[clap(short = 'O', long)]
    overwrite: bool,

    /// Delete the original directory after encrypting
    #[clap(short = 'd', long)]
    delete_original: bool,

    #[clap(from_global)]
    pub debug: bool,
}

pub fn execute(args: &Command) -> ActionResult {
    if args.paths.len() == 0 {
        return Err(ActionError::new("You must provide at least one path"));
    }

    if args.paths.iter().any(|p| !p.exists()) {
        return Err(ActionError::new("One or more provided paths don't exist"));
    }

    if args.output_path.is_some() {
        if args.paths.len() != 1 {
            return Err(ActionError::new(
                "Only one input path can be provided when setting an output path",
            ));
        }
        if args.output_path.as_ref().unwrap().exists() {
            return Err(ActionError::new(
                "The specified custom output file/directory already exists, please remove it",
            ));
        }
    }

    let key = prompt_password(log::format_info("Enter key: "))
        .map_err(|e| ActionError::new_with_error("Failed to read key", e))?;
    let confirm_key = prompt_password(log::format_info("Confirm key: "))
        .map_err(|e| ActionError::new_with_error("Failed to read confirm key", e))?;

    if key.ne(&confirm_key) {
        return Err(ActionError::new("The two keys don't match"));
    }

    if key.len() < 1 {
        return Err(ActionError::new(
            "The key cannot be less than 1 character long",
        ));
    }

    log::println_info("Encrypting...");

    let (elapsed, success, skipped, failed) = actions::encrypt(
        args.paths.to_owned(),
        args.output_path.to_owned(),
        key,
        args.overwrite,
        args.delete_original,
    )?;

    if !success.is_empty() {
        log::println_success(format!(
            "Encrypted {} files in {}",
            success.len(),
            human_duration(&elapsed)
        ));
        if args.debug {
            success.iter().for_each(|msg| {
                log::println_success(log::with_start_line(msg.to_str().unwrap(), "    "))
            });
        }
    }
    if !failed.is_empty() {
        log::println_error(format!("Failed to encrypt {} files", failed.len()));
        if args.debug {
            failed.iter().for_each(|msg| {
                log::println_error(log::with_start_line(msg.to_str().unwrap(), "    "))
            });
        }
    }
    if !skipped.is_empty() {
        log::println_info(format!(
            "{} entr{} were skipped (unknown type)",
            skipped.len(),
            if skipped.len() == 1 { "y" } else { "ies" }
        ));
        if args.debug {
            skipped.iter().for_each(|msg| {
                log::println_info(log::with_start_line(msg.to_str().unwrap(), "    "))
            });
        }
    }

    Ok(())
}
