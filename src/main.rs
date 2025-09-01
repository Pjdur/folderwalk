use std::env;
use std::fs::{self, File, ReadDir};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

struct Config {
    start_dir: PathBuf,
    max_depth: Option<usize>,
    ascii: bool,
    show_content: bool,
    to_stdout: bool,
}

fn main() {
    let config = match parse_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            print_usage();
            std::process::exit(2);
        }
    };

    if let Err(e) = run(&config) {
        eprintln!("Failed: {e}");
        std::process::exit(1);
    }
}

fn parse_args() -> Result<Config, String> {
    let mut args = env::args().skip(1);

    let mut start_dir: Option<PathBuf> = None;
    let mut max_depth: Option<usize> = None;
    let mut ascii = false;
    let mut show_content = false;
    let mut to_stdout = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--max-depth" => {
                let v = args
                    .next()
                    .ok_or_else(|| "--max-depth requires a value".to_string())?;
                let d: usize = v
                    .parse()
                    .map_err(|_| "Invalid --max-depth value".to_string())?;
                max_depth = Some(d);
            }
            "--ascii" => {
                ascii = true;
            }
            "--content" | "-c" => {
                show_content = true;
            }
            "--stdout" | "-o" => {
                to_stdout = true;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            _ if arg.starts_with('-') => {
                return Err(format!("Unknown flag: {arg}"));
            }
            _ => {
                if start_dir.is_none() {
                    start_dir = Some(PathBuf::from(arg));
                } else {
                    return Err("Only one path argument is allowed".to_string());
                }
            }
        }
    }

    let start_dir = start_dir.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| ".".into()));
    Ok(Config {
        start_dir,
        max_depth,
        ascii,
        show_content,
        to_stdout,
    })
}

fn print_usage() {
    eprintln!(
        "Usage: folderwalk [path] [--max-depth N] [--ascii] [--content] [--stdout]
  - path:         directory to scan (default: current directory)
  - --max-depth N: limit recursion depth
  - --ascii:      use ASCII tree characters instead of Unicode
  - --content, -c: include file contents
  - --stdout, -o: output to stdout instead of files.txt
Output: files.txt is created in the target directory unless --stdout is used."
    );
}

fn run(config: &Config) -> io::Result<()> {
    let start_meta = fs::metadata(&config.start_dir)?;
    if !start_meta.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Path is not a directory: {}",
                config.start_dir.to_string_lossy()
            ),
        ));
    }

    let output_path = config.start_dir.join("files.txt");

    let mut writer: Box<dyn Write> = if config.to_stdout {
        Box::new(io::stdout())
    } else {
        let outfile = File::create(&output_path)?;
        Box::new(BufWriter::with_capacity(128 * 1024, outfile))
    };

    writeln!(
        writer,
        "{}",
        display_root_name(&config.start_dir).unwrap_or_else(|| ".".to_string())
    )?;

    walk_dir(
        &config.start_dir,
        if config.to_stdout { None } else { Some(&output_path) },
        &mut *writer,
        "",
        0,
        config.max_depth,
        config.ascii,
        config.show_content,
        true,
    )?;

    writer.flush()?;
    Ok(())
}

fn display_root_name(p: &Path) -> Option<String> {
    p.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .or_else(|| Some(p.to_string_lossy().to_string()))
}

fn walk_dir(
    dir: &Path,
    output_path: Option<&Path>,
    writer: &mut dyn Write,
    prefix: &str,
    depth: usize,
    max_depth: Option<usize>,
    ascii: bool,
    show_content: bool,
    is_root: bool,
) -> io::Result<()> {
    if let Some(maxd) = max_depth {
        if depth >= maxd {
            return Ok(());
        }
    }

    let mut entries = read_dir_entries(dir)?;
    if let Some(out_path) = output_path {
        entries.retain(|e| e.path != out_path);
    }

    entries.sort_by(|a, b| {
        let ad = a.file_type.is_dir();
        let bd = b.file_type.is_dir();
        match ad.cmp(&bd).reverse() {
            std::cmp::Ordering::Equal => {
                let an = a.file_name.to_string_lossy().to_lowercase();
                let bn = b.file_name.to_string_lossy().to_lowercase();
                an.cmp(&bn)
            }
            other => other,
        }
    });

    let (tee, elbow, pipe, space) = if ascii {
        ("|-- ", "`-- ", "|   ", "    ")
    } else {
        ("├── ", "└── ", "│   ", "    ")
    };

    for (idx, entry) in entries.iter().enumerate() {
        let is_last = idx == entries.len().saturating_sub(1);
        let branch = if is_last { elbow } else { tee };

        let mut name = entry.file_name.to_string_lossy().to_string();
        if entry.file_type.is_dir() {
            name.push('/');
        }

        let display_name = if entry.file_type.is_symlink() {
            match fs::read_link(&entry.path) {
                Ok(target) => format!("{name} -> {}", target.to_string_lossy()),
                Err(_) => format!("{name} -> <unreadable>"),
            }
        } else {
            name
        };

        if !is_root {
            writeln!(writer, "{prefix}{branch}{display_name}")?;
        } else {
            writeln!(writer, "{branch}{display_name}")?;
        }

        if entry.file_type.is_file() && show_content {
            match fs::read_to_string(&entry.path) {
                Ok(content) => {
                    writeln!(writer, "{prefix}    --- FILE CONTENT START ---")?;
                    for line in content.lines() {
                        writeln!(writer, "{prefix}    {}", line)?;
                    }
                    writeln!(writer, "{prefix}    --- FILE CONTENT END ---")?;
                }
                Err(err) => {
                    writeln!(writer, "{prefix}    [Could not read file: {}]", err)?;
                }
            }
        }

        if entry.file_type.is_dir() && !entry.is_symlink_dir {
            let new_prefix = if is_last {
                if is_root {
                    space.to_string()
                } else {
                    format!("{prefix}{space}")
                }
            } else {
                if is_root {
                    pipe.to_string()
                } else {
                    format!("{prefix}{pipe}")
                }
            };
            walk_dir(
                &entry.path,
                output_path,
                writer,
                &new_prefix,
                depth + 1,
                max_depth,
                ascii,
                show_content,
                false,
            )?;
        }
    }

    Ok(())
}

struct DirEntryInfo {
    path: PathBuf,
    file_name: std::ffi::OsString,
    file_type: fs::FileType,
    is_symlink_dir: bool,
}

fn read_dir_entries(dir: &Path) -> io::Result<Vec<DirEntryInfo>> {
    let rd: ReadDir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(err) => {
            eprintln!(
                "Warning: cannot read directory {}: {}",
                dir.to_string_lossy(),
                err
            );
            return Ok(Vec::new());
        }
    };

    let mut out = Vec::with_capacity(64);
    for res in rd {
        match res {
            Ok(de) => {
                let sy_meta = match fs::symlink_metadata(de.path()) {
                    Ok(m) => m,
                    Err(err) => {
                        eprintln!(
                            "Warning: cannot stat {}: {}",
                            de.path().to_string_lossy(),
                            err
                        );
                        continue;
                    }
                };
                let ft = sy_meta.file_type();
                let is_symlink = ft.is_symlink();

                let file_type = if is_symlink {
                    ft
                } else {
                    match fs::metadata(de.path()) {
                        Ok(m) => m.file_type(),
                        Err(_) => ft,
                    }
                };

                let is_symlink_dir =
                    is_symlink && fs::metadata(de.path()).map(|m| m.is_dir()).unwrap_or(false);

                use std::collections::HashSet;

                let excluded_dirs: HashSet<&str> =
                    ["node_modules", ".git", "target"].iter().cloned().collect();

                let file_name_os = de.file_name();
                let file_name_str = file_name_os.to_string_lossy();

                if excluded_dirs.contains(file_name_str.as_ref()) {
                    continue;
                }

                out.push(DirEntryInfo {
                    path: de.path(),
                    file_name: de.file_name(),
                    file_type,
                    is_symlink_dir,
                });
            }
            Err(err) => {
                eprintln!(
                    "Warning: error while reading in {}: {}",
                    dir.to_string_lossy(),
                    err
                );
            }
        }
    }
    Ok(out)
}
