# **Partage Server**

## Command Line File Sharing
This is the server portion of the partage cli client. The server and the command line client are both built entirely in [Rust](https://www.rust-lang.org/).

## Install:
Currently the only way to install and use the server is to install it from source. You will need Rust in order to build and run the executable. 

> [!Note]
> You can install Rust using rustup from here: [rustup](https://rustup.rs/).

Once Rust is installed, clone the repository, cd into the directory and then run `cargo build --release`. Now you can run the executable. 

## Usage:

Currently, the default port used by the server is 3030. Changing the server's default port will be added in the future. 
