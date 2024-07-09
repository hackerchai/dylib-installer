use clap::{Arg, Command, ArgAction, error::ErrorKind};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as SysCommand;
use anyhow::{Context, Result};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::io::Write;

fn main() -> Result<()> {
    let app = Command::new("dylib Installer")
        .version("0.1.0")
        .disable_version_flag(true)
        .author("LLGO Team")
        .about("Handles dylib directories and generates .pc files")
        .arg(Arg::new("dylib_path")
            .short('d')
            .long("dylib")
            .value_name("DIR")
            .help("Sets the directory where the dylib files are stored")
            .action(ArgAction::Set)
            .required(true))
        .arg(Arg::new("library_name")
            .short('n')
            .long("name")
            .value_name("NAME")
            .help("Sets the name of the library")
            .action(ArgAction::Set))
        .arg(Arg::new("header_path")
            .short('i')
            .long("headerpath")
            .value_name("HEADERPATH")
            .help("Sets the path to store the header files")
            .action(ArgAction::Set)
        )
        .arg(Arg::new("version")
            .short('v')
            .long("version")
            .value_name("VERSION")
            .help("Sets the version of the library")
            .action(ArgAction::Set))
        .arg(Arg::new("description")
            .short('c')
            .long("description")
            .value_name("DESC")
            .help("Sets the description of the library")
            .action(ArgAction::Set))
        .arg(Arg::new("pc_path")
            .short('p')
            .long("pcpath")
            .value_name("PCPATH")
            .help("Sets the path to store the .pc file")
            .action(ArgAction::Set))
        .arg(Arg::new("lib_target_path")
            .short('t')
            .long("libpath")
            .value_name("LIBPATH")
            .help("Sets the target path for the library files")
            .action(ArgAction::Set))
        .arg(Arg::new("header_target_path")
            .short('r')
            .long("header_target_path")
            .value_name("HEADER_TARGET_PATH")
            .help("Sets the target path for the header files")
            .action(ArgAction::Set)
        );
    
    let matches = app.clone().try_get_matches();
    match matches {
        Ok(matches) => {
            let dylib_path = matches.get_one::<String>("dylib_path").unwrap();
            let version = matches.get_one::<String>("version")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "0.1.0".to_string());
            let description = matches.get_one::<String>("description")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "No description provided".to_string());
            let pc_path = matches.get_one::<String>("pc_path")
                .map(PathBuf::from)
                .unwrap_or_else(|| get_pc_path().context("Failed to get pc_path").unwrap());
            let lib_target_path = matches.get_one::<String>("lib_target_path")
                .map(PathBuf::from)
                .unwrap_or_else(|| get_system_lib_path().context("Failed to get lib_target_path").unwrap());
            let lib_source_path = fs::canonicalize(dylib_path)
                .context("Failed to convert dylib_path to absolute path")?;

            let library_name = matches.get_one::<String>("library_name").map(|s| s.to_string())
                .or_else(|| {
                    // if no library name is provided, try to find a dylib file in the directory
                    fs::read_dir(&lib_source_path).ok()?.find_map(|entry| {
                        let entry = entry.ok()?;
                        let path = entry.path();
                        if path.extension()? == "dylib" {
                            path.file_stem()?.to_str().map(|s| {
                                // remove the `lib` prefix and the extension
                                s.trim_start_matches("lib").split('.').next().unwrap().to_string()
                            })
                        } else {
                            None
                        }
                    })
                }).unwrap_or_else(|| {
                println!("No library name provided and no dylib file found in the directory. Exiting.");
                std::process::exit(1);
            });
            
            let header_target_path = matches.get_one::<String>("header_target_path")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    let base = lib_target_path.parent().unwrap_or(&lib_target_path);
                    base.join("include").join(library_name.clone())
                });

            let header_source_path = matches.get_one::<String>("header_path")
                .map(PathBuf::from);
            let header_source_path = header_source_path.as_ref().map(|p| fs::canonicalize(p).context("Failed to convert header_path to absolute path").unwrap());

            // print the library information
            let mut stdout = StandardStream::stdout(ColorChoice::Always);

            print_colored(&mut stdout, "Library Name: ", &library_name, Color::Green)?;
            print_colored(&mut stdout, "Version: ", &version, Color::Green)?;
            print_colored(&mut stdout, "Description: ", &description, Color::Green)?;
            print_colored(&mut stdout, "Library Source Path: ", &format!("{:?}", lib_source_path), Color::Green)?;
            print_colored(&mut stdout, "Library Target Path: ", &format!("{:?}", lib_target_path), Color::Green)?;
            if let Some(header_source_path) = &header_source_path {
                print_colored(&mut stdout, "Header Source Path: ", &format!("{:?}", header_source_path), Color::Green)?;
            }
            print_colored(&mut stdout, "Header Target Path: ", &format!("{:?}", header_target_path), Color::Green)?;
            print_colored(&mut stdout, "Pkg-config Path: ", &format!("{:?}", pc_path), Color::Green)?;
            let pc_full_path = pc_path.join(format!("{}.pc", library_name));
            print_colored(&mut stdout, "PC File Path: ", &format!("{:?}", pc_full_path), Color::Green)?;
            
            generate_pc_file(&lib_target_path, &pc_path, &library_name, &version, &description)
                .context("Failed to generate pc file")?;
            copy_lib_files(&lib_source_path, &lib_target_path)
                .context("Failed to copy library files")?;
            if let Some(header_source_path) = header_source_path {
                if header_source_path.exists() {
                    copy_header_files(&header_source_path, &header_target_path)
                        .context("Failed to copy header files")?;
                } else {
                    print_colored(&mut stdout, "Warning: ", "Header path provided does not exist, skipping header file copy.", Color::Yellow)?;
                }
            } else {
                print_colored(&mut stdout, "Warning: ", "No header files provided, skipping header file copy.", Color::Yellow)?;
            }

            print_colored(&mut stdout, "Success!!!", " Library installation completed successfully", Color::Green)?;
        }
        Err(ref e) if e.kind() == ErrorKind::MissingRequiredArgument => {
            println!("Error: Missing required arguments: {}", e);
            std::process::exit(1);
        }
        Err(e) => {
            println!("Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

// print colored text
fn print_colored(stdout: &mut StandardStream, label: &str, value: &str, color: Color) -> Result<()> {
    stdout.set_color(ColorSpec::new().set_fg(Some(color)))?;
    write!(stdout, "{}", label)?;
    stdout.reset()?;
    writeln!(stdout, "{}", value)?;
    Ok(())
}


// generate the .pc file
fn generate_pc_file(lib_target_path: &Path, pc_path: &Path, library_name: &str, version: &str, description: &str) -> Result<()> {
    // check if the target path is `lib` or `lib64`
    let libdir_suffix = if let Some(component) = lib_target_path.components().last() {
        let dir = component.as_os_str().to_str().unwrap_or("");
        if dir == "lib" || dir == "lib64" {
            dir
        } else {
            "lib"
        }
    } else {
        "lib"
    };

    let prefix_path = lib_target_path.parent().unwrap_or(lib_target_path);

    let content = format!(
        "prefix={}\nlibdir=${{prefix}}/{}\nincludedir=${{prefix}}/include\n\nName: {}\nDescription: {}\nVersion: {}\nLibs: -L${{libdir}} -l{}\nCflags: -I${{includedir}}",
        prefix_path.display(),
        libdir_suffix,
        library_name,
        description,
        version,
        library_name
    );
    
    fs::write(pc_path.join(format!("{}.pc", library_name)), content)
        .context("Failed to write .pc file")?;
    Ok(())
}


// copy library files
fn copy_lib_files(source: &Path, target: &Path) -> Result<()> {
    if !target.exists() {
        return Err(anyhow::anyhow!("target lib directory not exists: {}", source.display()));
    }
    for entry in fs::read_dir(source).context("Failed to read source directory")? {
        let path = entry?.path();
        if path.is_dir() {
            continue; // skip directories
        }
        if let Some(ext) = path.extension() {
            if ext == "a" || ext == "dylib" || ext == "d" {
                fs::copy(&path, target.join(path.file_name().unwrap()))
                    .with_context(|| format!("Failed to copy file from {:?} to {:?}", path, target))?;
            }
        }
    }
    Ok(())
}

// copy header files
fn copy_header_files(source: &Path, target: &Path) -> Result<()> {
    if !target.exists() {
        fs::create_dir_all(target).context("Failed to create target directory")?;
    }

    for entry in fs::read_dir(source).context("Failed to read source directory")? {
        let path = entry?.path();
        if path.is_dir() {
            continue; // skip directories
        }

        if let Some(ext) = path.extension() {
            if ext == "h" {
                fs::copy(&path, target.join(path.file_name().unwrap()))
                    .with_context(|| format!("Failed to copy file from {:?} to {:?}", path, target))?;
            }
        }
    }
    Ok(())
}


// get the pkg-config path
fn get_pc_path() -> Result<PathBuf> {
    let output = SysCommand::new("pkg-config")
        .arg("--variable=pc_path")
        .arg("pkg-config")
        .output()
        .context("Failed to execute pkg-config command")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("pkg-config command failed to execute successfully"));
    }

    let path_str = String::from_utf8_lossy(&output.stdout).split(':').next().unwrap().trim().to_string();
    Ok(PathBuf::from(path_str))
}

// get the system library path
fn get_system_lib_path() -> Result<PathBuf> {
    let env_path = std::env::var("LD_LIBRARY_PATH")
        .or_else(|_| std::env::var("DYLD_LIBRARY_PATH"))
        .or_else(|_| std::env::var("DYLD_FALLBACK_LIBRARY_PATH"))
        .unwrap_or_else(|_| String::from("/usr/local/lib"));

    let path = PathBuf::from(&env_path);
    if path.exists() {
        Ok(path)
    } else {
        let fallbacks = ["/usr/local/lib", "/usr/lib", "/lib"];
        fallbacks.iter()
            .map(|&p| PathBuf::from(p))
            .find(|p| p.exists())
            .ok_or_else(|| anyhow::anyhow!("No suitable library path found"))
    }
}
