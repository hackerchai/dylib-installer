# Dylib Installer

Dylib Installer is a tool for handling dylib directories and generating .pc files. It helps to automate the process of installing dynamic libraries and their associated headers, and it ensures that the required pkg-config files are generated correctly.

## Features

- Automatically detects and processes `.dylib` files in a specified directory.
- Generates `.pc` files for use with `pkg-config`.
- Copies library files to the target directory.
- Optionally copies header files to the target directory.

## Installation

To install Dylib Installer, ensure you have Rust installed on your system. You can install the project from source or using `cargo install`:

### From Source

```sh
git clone https://github.com/hackerchai/dylib-installer.git
cd dylib-installer
cargo build --release
```

The binary will be located in the `target/release` directory.

### Using Cargo Install

```sh
cargo install --git https://github.com/hackerchai/dylib-installer
```

## Usage

Dylib Installer requires several arguments to specify the paths and options. Here are the available options:

```sh
dylib_installer [OPTIONS] <dylib_path> [HEADERPATH]
```
### Arguments:
- `<dylib_path>`  Sets the directory where the dylib files are stored.(required)

- `[HEADERPATH]`  Sets the path to store the header files

### Options

- `-n, --name <NAME>`: Sets the name of the library. If not provided, it will be inferred from the dylib file name.
- `-i, --headerpath <HEADERPATH>`: Sets the path to store the header files.
- `-v, --version <VERSION>`: Sets the version of the library. Default is "0.1.0".
- `-c, --description <DESC>`: Sets the description of the library. Default is "No description provided".
- `-p, --pcpath <PCPATH>`: Sets the path to store the .pc file. If not provided, it will use the default pkg-config path.
- `-t, --libpath <LIBPATH>`: Sets the target path for the library files. If not provided, it will use the system library path.
- `-r, --header_target_path <HEADER_TARGET_PATH>`: Sets the target path for the header files.
- `-h, --help`: Print help.

### Example

**Recommend to use sudo to install the library to system path.**

- In most cases (without specifying headers):
    ```sh
    sudo dylib_installer /path/to/dylibs
    ```

- If you want to specify full options:
    ```sh
    dylib_installer /path/to/dylibs /path/to/headers \
        -n mylibrary \
        -v 0.1.0 \
        -c "My Library Description" \
        -p /path/to/pkgconfig \
        -t /usr/local/lib \
        -r /usr/local/include/mylibrary
    ```

If you do not provide a library name, the tool will attempt to infer it from the `.dylib` file names found in the specified directory. For example, if it finds a file named `libfuse.dylib` or `libfuse.2.1.dylib`, it will use `fuse` as the library name.

## Contributing

We welcome contributions! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the GPL License. See the [LICENSE](https://github.com/hackerchai/dylib-installer/blob/main/LICENSE) file for details.