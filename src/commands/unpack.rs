use std::{
    fs::{self, OpenOptions},
    io,
    path::PathBuf,
    time,
};

use clap::Args;
use human_duration::human_duration;
use rpassword::prompt_password;

use crate::cli::util::{ActionError, ActionResult};
use fencryption::{crypto::Crypto, pack::Pack, tmp_dir::TmpDir};

#[derive(Args)]
/// Opens a pack: creates a directory where the decrypted
/// files appear. To close the pack see command "close".
pub struct Command {
    /// Path of the pack to open
    path: PathBuf,

    /// Whether to overwrite the output directory or not
    #[clap(short = 'O', long)]
    overwrite: bool,

    #[clap(from_global)]
    debug: bool,
}

pub fn action(args: &Command) -> ActionResult {
    let key = match prompt_password("[INPUT] Enter a key: ") {
        Ok(v) => v,
        Err(e) => {
            return Err(ActionError::new(
                "Failed to read key",
                Some(format!("  - {:?}", e)),
            ))
        }
    };

    if key.len() < 1 {
        return Err(ActionError::new(
            "The key cannot be less than 1 character long",
            None,
        ));
    };

    let timer = time::SystemTime::now();

    let crypto = match Crypto::new(&key.as_bytes()) {
        Ok(v) => v,
        Err(e) => {
            return Err(ActionError::new(
                "Failed to create cipher",
                if args.debug == true {
                    Some(format!("  - {:?}", e))
                } else {
                    None
                },
            ));
        }
    };

    let tmp_dir = match TmpDir::new() {
        Ok(v) => v,
        Err(e) => {
            return Err(ActionError::new(
                "Failed to create temporary directory",
                if args.debug == true {
                    Some(format!("  - {:?}", e))
                } else {
                    None
                },
            ));
        }
    };

    let tmp_pack_path = tmp_dir.gen_path();

    let mut source = match OpenOptions::new().read(true).write(true).open(&args.path) {
        Ok(v) => v,
        Err(e) => {
            return Err(ActionError::new(
                "Failed to read pack file",
                if args.debug == true {
                    Some(format!("  - {:?}", e))
                } else {
                    None
                },
            ));
        }
    };
    let mut dest = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&tmp_pack_path)
    {
        Ok(v) => v,
        Err(e) => {
            return Err(ActionError::new(
                "Failed to read/create temporary decrypted pack file",
                if args.debug == true {
                    Some(format!("  - {:?}", e))
                } else {
                    None
                },
            ));
        }
    };

    match crypto.decrypt_stream(&mut source, &mut dest) {
        Ok(_) => {
            println!("[OK] Encrypted pack");
        }
        Err(e) => {
            return Err(ActionError::new(
                "Failed to decrypt pack",
                if args.debug == true {
                    Some(format!("  - {:?}", e))
                } else {
                    None
                },
            ));
        }
    };

    let dir_path = PathBuf::from(match args.path.file_stem() {
        Some(v) => v,
        None => {
            return Err(ActionError::new("Failed to get output path", None));
        }
    });

    match Pack::new(&tmp_pack_path).unpack(&dir_path) {
        Ok(_) => {
            println!("[OK] Unpacked pack");
        }
        Err(e) => {
            return Err(ActionError::new(
                "Failed to unpack pack",
                if args.debug == true {
                    Some(format!("  - {:?}", e))
                } else {
                    None
                },
            ));
        }
    };

    println!(
        "[DONE] Decrypted pack in {}",
        human_duration(&timer.elapsed().unwrap_or_default())
    );

    println!("[INPUT] Press \"u\" to update the pack and any other key to discard changes");

    let stdout = console::Term::buffered_stdout();
    loop {
        if let Ok(c) = stdout.read_char() {
            match c {
                'u' => {
                    if let Err(e) = fs::remove_file(&args.path) {
                        return Err(ActionError::new(
                            "Failed to remove outdated pack",
                            if args.debug == true {
                                Some(format!("  - {:?}", e))
                            } else {
                                None
                            },
                        ));
                    };

                    let tmp_dir = match TmpDir::new() {
                        Ok(v) => v,
                        Err(e) => {
                            return Err(ActionError::new(
                                "Failed to create temporary directory",
                                if args.debug == true {
                                    Some(format!("  - {:?}", e))
                                } else {
                                    None
                                },
                            ));
                        }
                    };
                    let tmp_pack_path = tmp_dir.gen_path();

                    if let Err(e) = Pack::new(&tmp_pack_path).create(&dir_path) {
                        return Err(ActionError::new(
                            "Failed to update pack",
                            if args.debug == true {
                                Some(format!("  - {:?}", e))
                            } else {
                                None
                            },
                        ));
                    };

                    if let Err(e) = fs::remove_dir_all(&dir_path) {
                        return Err(ActionError::new(
                            "Failed to remove original directory",
                            if args.debug == true {
                                Some(format!("  - {:?}", e))
                            } else {
                                None
                            },
                        ));
                    };

                    let mut source = match OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(&tmp_pack_path)
                    {
                        Ok(v) => v,
                        Err(e) => {
                            return Err(ActionError::new(
                                "Failed to read pack file",
                                if args.debug == true {
                                    Some(format!("  - {:?}", e))
                                } else {
                                    None
                                },
                            ));
                        }
                    };
                    let mut dest = match OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .open(&args.path)
                    {
                        Ok(v) => v,
                        Err(e) => {
                            return Err(ActionError::new(
                                "Failed to update pack file",
                                if args.debug == true {
                                    Some(format!("  - {:?}", e))
                                } else {
                                    None
                                },
                            ));
                        }
                    };

                    if let Err(e) = crypto.encrypt_stream(&mut source, &mut dest) {
                        return Err(ActionError::new(
                            "Failed to encrypt updated pack",
                            if args.debug == true {
                                Some(format!("  - {:?}", e))
                            } else {
                                None
                            },
                        ));
                    };

                    println!("[OK] Updated pack");

                    break;
                }
                _ => break,
            }
        }
    }

    if let Err(e) = fs::remove_dir_all(&dir_path) {
        match e.kind() {
            io::ErrorKind::NotFound => (),
            e => {
                return Err(ActionError::new(
                    "Failed to remove original directory",
                    if args.debug == true {
                        Some(format!("  - {:?}", e))
                    } else {
                        None
                    },
                ));
            }
        };
    };

    Ok(None)
}
