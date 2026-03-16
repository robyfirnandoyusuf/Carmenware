use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(
    name = "file_hider",
    version = "1.2",
    about = "Hide/unhide files inside images - supports batch mode and custom extensions"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List {
        #[arg(short, long, default_value = ".")]
        path: String,
    },

    Hide {
        #[arg(short, long)]
        input: Option<String>,

        #[arg(short = 'p', long)]
        path: Option<String>,

        #[arg(short, long)]
        cover: String,

        #[arg(short, long)]
        key: String,

        #[arg(short = 'o', long)]
        output_dir: Option<String>,

        #[arg(short = 'e', long, default_value = "jpg")]
        ext: String,
    },

    Unhide {
        #[arg(short, long)]
        input: Option<String>,

        #[arg(short = 'p', long)]
        path: Option<String>,

        #[arg(short, long)]
        key: String,

        #[arg(short = 'o', long)]
        output_dir: Option<String>,
    },
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { path } => list_files(&path),

        Commands::Hide {
            input,
            path,
            cover,
            key,
            output_dir,
            ext,
        } => {
            if let Some(dir) = path {
                batch_hide(&dir, &cover, &key, output_dir.as_deref(), &ext)
            } else if let Some(file) = input {
                hide_file(file, &cover, &key, output_dir.as_deref(), &ext)
            } else {
                Err("You must specify either --input or --path".into())
            }
        }

        Commands::Unhide {
            input,
            path,
            key,
            output_dir,
        } => {
            if let Some(file) = input {
                unhide_file(&file, &key, output_dir.as_deref())
            } else if let Some(dir) = path {
                batch_unhide(&dir, &key, output_dir.as_deref())
            } else {
                Err("You must specify either --input or --path".into())
            }
        }
    }
}

fn list_files(path: &str) -> Result<()> {
    let dir = Path::new(path);

    println!("Files in {}:", dir.display());

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();

        if p.is_file() {
            println!("  {}", p.display());
        }
    }

    Ok(())
}

fn xor(data: &[u8], key: &str) -> Vec<u8> {
    if key.is_empty() {
        return data.to_vec();
    }

    let k = key.as_bytes();

    data.iter()
        .enumerate()
        .map(|(i, &b)| b ^ k[i % k.len()])
        .collect()
}

fn hide_file(
    input_str: String,
    cover_str: &str,
    key: &str,
    out_dir_opt: Option<&str>,
    output_ext: &str,
) -> Result<()> {

    let input_path = Path::new(&input_str);

    if !input_path.is_file() {
        return Err(format!("Not a file: {}", input_str).into());
    }

    let cover_bytes = fs::read(cover_str)?;
    let orig_bytes = fs::read(input_path)?;

    let encrypted = xor(&orig_bytes, key);

    let orig_name = input_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let name_bytes = orig_name.as_bytes();

    let mut payload = Vec::new();

    payload.extend_from_slice(&(name_bytes.len() as u64).to_le_bytes());
    payload.extend_from_slice(name_bytes);
    payload.extend_from_slice(&(encrypted.len() as u64).to_le_bytes());
    payload.extend_from_slice(&encrypted);

    let mut combined = cover_bytes;

    combined.extend_from_slice(&payload);

    let payload_len = payload.len() as u64;

    combined.extend_from_slice(&payload_len.to_le_bytes());

    const MAGIC: &[u8] = b"CARMENWARE_WAS_HERE!!!";

    combined.extend_from_slice(MAGIC);

    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("hidden");

    let orig_ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let ext_suffix = if !orig_ext.is_empty() {
        format!("~{}", orig_ext)
    } else {
        String::new()
    };

    let out_filename = format!("{}{}.{}", stem, ext_suffix, output_ext);

    let out_dir = out_dir_opt.unwrap_or_else(|| {
        input_path.parent().and_then(|p| p.to_str()).unwrap_or(".")
    });

    let out_path = Path::new(out_dir).join(out_filename);

    fs::write(&out_path, combined)?;

    println!("Hidden → {}", out_path.display());

    fs::remove_file(input_path)?;

    println!("Original file deleted → {}", input_path.display());

    Ok(())
}

fn batch_hide(
    dir_str: &str,
    cover: &str,
    key: &str,
    out_dir_opt: Option<&str>,
    output_ext: &str,
) -> Result<()> {

    let dir = Path::new(dir_str);

    if !dir.is_dir() {
        return Err(format!("Not a directory: {}", dir_str).into());
    }

    println!("Hiding all non-image files in: {}", dir.display());

    for entry in fs::read_dir(dir)? {

        let entry = entry?;

        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let is_image = ["jpg","jpeg","png","gif","webp","bmp"].contains(&ext.as_str());

        if is_image || path.file_name().and_then(|n| n.to_str()).map_or(false, |n| n.starts_with('.')) {
            continue;
        }

        if let Err(e) = hide_file(
            path.to_string_lossy().into_owned(),
            cover,
            key,
            out_dir_opt,
            output_ext,
        ) {
            eprintln!("Failed to hide {} → {}", path.display(), e);
        }
    }

    println!("Batch hide completed.");

    Ok(())
}

fn unhide_file(
    input_str: &str,
    key: &str,
    out_dir_opt: Option<&str>,
) -> Result<()> {

    let input_path = Path::new(input_str);

    if !input_path.is_file() {
        return Err(format!("Not a file: {}", input_str).into());
    }

    let bytes = fs::read(input_path)?;

    const MAGIC: &[u8] = b"CARMENWARE_WAS_HERE!!!";

    if bytes.len() < MAGIC.len() + 8 {
        return Err("File too small".into());
    }

    let magic_start = bytes.len() - MAGIC.len();

    if &bytes[magic_start..] != MAGIC {
        return Err("Not a hidden file".into());
    }

    let len_start = magic_start - 8;

    let payload_len =
        u64::from_le_bytes(bytes[len_start..len_start + 8].try_into().unwrap()) as usize;

    if payload_len + 8 + MAGIC.len() > bytes.len() {
        return Err("Corrupted payload".into());
    }

    let payload_start = len_start - payload_len;

    let payload = &bytes[payload_start..len_start];

    let mut offset = 0;

    let name_len =
        u64::from_le_bytes(payload[offset..offset + 8].try_into().unwrap()) as usize;

    offset += 8;

    if offset + name_len > payload.len() {
        return Err("Corrupted payload".into());
    }

    let orig_name =
        String::from_utf8_lossy(&payload[offset..offset + name_len]).to_string();

    offset += name_len;

    let data_len =
        u64::from_le_bytes(payload[offset..offset + 8].try_into().unwrap()) as usize;

    offset += 8;

    if offset + data_len > payload.len() {
        return Err("Corrupted payload".into());
    }

    let enc_data = &payload[offset..offset + data_len];

    let decrypted = xor(enc_data, key);

    let out_dir = out_dir_opt.unwrap_or_else(|| {
        input_path
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or(".")
    });

    let out_path = Path::new(out_dir).join(&orig_name);

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&out_path, &decrypted)?;

    println!("Restored → {}", out_path.display());

    if out_path.exists() {
        fs::remove_file(input_path)?;
        println!("Container deleted → {}", input_path.display());
    }

    Ok(())
}

fn batch_unhide(
    dir_str: &str,
    key: &str,
    out_dir_opt: Option<&str>,
) -> Result<()> {

    let dir = Path::new(dir_str);

    if !dir.is_dir() {
        return Err(format!("Not a directory: {}", dir_str).into());
    }

    const MAGIC: &[u8] = b"CARMENWARE_WAS_HERE!!!";

    println!("Scanning for hidden files in: {}", dir.display());

    let mut count = 0;

    for entry in fs::read_dir(dir)? {

        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(_) => continue,
        };

        if bytes.len() < MAGIC.len() {
            continue;
        }

        let magic_start = bytes.len() - MAGIC.len();

        if &bytes[magic_start..] != MAGIC {
            continue;
        }

        if let Err(e) = unhide_file(path.to_str().unwrap(), key, out_dir_opt) {
            eprintln!("Failed {} → {}", path.display(), e);
        } else {
            count += 1;
        }
    }

    println!("Batch unhide completed. {} files restored.", count);

    Ok(())
}