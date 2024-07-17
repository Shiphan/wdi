# Walk through Directory with Interface

## Install

### Linux

```bash
git clone https://github.com/Shiphan/wdi.git
cd wdi
cargo build --release
```
then add this to your `.bashrc`:
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