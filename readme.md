# File Hider

<img src="https://i.pinimg.com/736x/5c/d4/2f/5cd42f50a3a602bd80186f6ebed3fe60.jpg" width="200px">

A simple Rust CLI tool to hide files inside image containers and restore them later.

This tool appends encrypted data to an image file, allowing the image to remain viewable while secretly storing another file inside it.

---


>This project is dedicated to **Carmen** from **Heart2Heart** in celebration of her birthday.
>Happy Birthday, Carmen!

---

# Features

- Hide files inside image containers
- Restore hidden files
- Batch hide/unhide mode
- Custom output image extension
- Automatic file name restoration
- Optional password-based XOR obfuscation
- CLI interface

---

# Installation

Make sure you have **Rust** installed.

https://rustup.rs

Then clone the repository and build:

```bash
git clone https://github.com/yourrepo/file_hider
cd file_hider
cargo build --release
```

Binary will be located at:
`
target/release/file_hider
`

## Usage
### encrypt files
`file_hider.exe hide --path .\directory --cover cover.jpg --key <key>`

### decrypt files
`file_hider.exe unhide --path .\directory --key <key>`