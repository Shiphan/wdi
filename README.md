# Walk through Directory with Interface

***wdi** is a TUI tool for you to walk through directory with interface.*  

If you are familiar with terminal, then using `cd` to get to where you want will be pretty simple. (and there's some tool to make this even faster)  
but for those people who are used to GUI or IDE, or you don't know what's inside this directory, then many `ls` and `cd` are required now, and **wdi** was born for these two situations.

- **NOTE:** This project is still in a very early stage, read the [**Current Stage**](#current-stage) part for more information.

## Usage
- `k` | `Up` - go up
- `j` | `Down` - go down
- `Enter` - enter the directory you selected
- `w` - quit and change directory
- `q` - quit and not change directory

## Install

### Rust

To use wdi, first make sure you have **rustc** and **cargo** installed, you can run these two commands to verify if they are installed:
```shell
rustc --version
cargo --version
```

If not, you can just install **rustup** and run:
```shell
rustup default stable
```

### Build
Find a good place to run this:
```bash
git clone https://github.com/Shiphan/wdi.git
cd wdi
```

Then,
- if you have added `.cargo/bin` to your `PATH`, you can simply run:
    ```
    cargo install --path .
    ```
- or manually build it and add `./target/release/wtdwi` to your `PATH`  
(it's `.\target\release\wtdwi.exe` if you are using windows)
    ```
    cargo build --release
    ```

## Set Profile

### Linux (Bash)

add this to your `~/.bashrc`:
```bash
wdi() {
    if [[ $# -le 1 ]]; then 
        dir=$(wtdwi "$1")
        [[ $? -eq 0 ]] && cd "$dir"
    else
        echo "too many arguments"
    fi
}
```

### Windows (PowerShell)

add this to your powershell profile  
(you can find it by running `echo $profile`, and if this file doesn't exist, you can create it)
```ps1
function wdi {
    param ( [String]$arg )
    $dir = wtdwi $arg
    if ($?) { cd $dir }
}
```

If the profile cannot be loaded by powershell, you can run this command:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned
```

## Current Stage

1. Missing some features that should be available:
    - search
    - better ui
2. Nice to have:
    - support for mouse
    - custom style