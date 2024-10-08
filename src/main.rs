use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom},
    net::Ipv6Addr,
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;
use directories::ProjectDirs;
use fd_lock::{RwLock, RwLockWriteGuard};
use serde::{Deserialize, Serialize};
use serde_yaml::from_reader;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use yggdrasil_keys::NodeIdentity;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    cache_size: Option<usize>,
    #[arg(short, long)]
    tries: Option<usize>,
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Deserialize, Default)]
struct Config {
    cache_size: Option<usize>,
    tries: Option<usize>,
}

#[derive(Serialize, Deserialize)]
struct CacheFile {
    keys: Vec<(String, u32)>,
}

struct Cache {
    keys: Vec<(NodeIdentity, u32)>,
    target_size: usize,
}

#[derive(Serialize, Deserialize)]
struct Output {
    public: String,
    secret: String,
    address: Ipv6Addr,
}

impl Cache {
    fn add_identity(&mut self, identity: NodeIdentity, strength: u32) {
        self.keys.push((identity, strength));
        self.keys.sort_by(|a, b| a.1.cmp(&b.1).reverse());
        while self.keys.len() > self.target_size {
            self.keys.pop();
        }
    }
    fn get_min_strength(&self) -> u32 {
        self.keys
            .last()
            .map(|(_key, strength)| *strength)
            .unwrap_or(0)
    }
    fn output(&mut self) -> Output {
        let identity = self.keys.remove(0).0;
        let (secret, public) = identity.to_hex_split();
        Output {
            public,
            secret,
            address: identity.address(),
        }
    }
    fn load(cache_file: CacheFile, target_size: usize) -> Self {
        Self {
            keys: cache_file
                .keys
                .iter()
                .map(|(key, strength)| (NodeIdentity::from_hex(key, None), strength))
                .filter(|(key, _strength)| key.is_ok())
                .map(|(key, strength)| (key.unwrap(), *strength))
                .collect(),
            target_size,
        }
    }

    fn new(target_size: usize) -> Self {
        Self {
            keys: Vec::new(),
            target_size,
        }
    }
}

impl From<Cache> for CacheFile {
    fn from(cache: Cache) -> CacheFile {
        CacheFile {
            keys: cache
                .keys
                .iter()
                .map(|(key, strength)| (key.to_hex_joined(), *strength))
                .collect(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let old_dirs = ProjectDirs::from("com", "Famedly GmbH", "Yggdrasil Key Generator")
        .context("Couldn't find old project directory, is $HOME set?")?;
    let dirs = ProjectDirs::from("", "", "yggdrasil-keygen")
        .context("Couldn't find project directory, is $HOME set?")?;

    let config_path = cli
        .config
        .unwrap_or_else(|| dirs.config_dir().join("config.yaml"));

    let config: Config = if config_path.exists() {
        let config_file = File::open(config_path)?;
        serde_yaml::from_reader(config_file)?
    } else {
        Default::default()
    };

    let old_cache_path = old_dirs.cache_dir().join("cache.yaml");
    let cache_dir = dirs.cache_dir();
    std::fs::create_dir_all(&cache_dir)?;
    let cache_path = cache_dir.join("cache.yaml");

    if old_cache_path.exists() {
        std::fs::rename(&old_cache_path, &cache_path)?;
    }

    let cache_size = cli
        .cache_size
        .or(config.cache_size)
        .unwrap_or(2usize.pow(8));
    let tries = cli.tries.or(config.tries).unwrap_or(2usize.pow(8));

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(cache_path)?;
    let mut lock = RwLock::new(file);
    let mut guard: RwLockWriteGuard<File> = lock.write().context("couldn't lock cache file")?;

    let cache: Cache = match guard.metadata()?.len() {
        0 => Cache::new(cache_size),
        _ => Cache::load(from_reader::<&std::fs::File, CacheFile>(&guard)?, 65536),
    };

    let min_strength = cache.get_min_strength();

    let (tx, rx) = unbounded_channel();

    let cache_handle = tokio::spawn(receive_keys(rx, cache));
    for _ in 0..tries {
        tokio::spawn(generate_identities(tx.clone(), min_strength));
    }
    drop(tx);

    let mut cache = cache_handle.await?;

    serde_json::to_writer_pretty(std::io::stdout(), &cache.output())?;

    guard.seek(SeekFrom::Start(0))?;

    serde_yaml::to_writer::<&File, CacheFile>(&guard, &cache.into())?;

    Ok(())
}

async fn receive_keys(mut rx: UnboundedReceiver<(NodeIdentity, u32)>, mut cache: Cache) -> Cache {
    while let Some(id) = rx.recv().await {
        cache.add_identity(id.0, id.1);
    }
    cache
}

async fn generate_identities(tx: UnboundedSender<(NodeIdentity, u32)>, min_strength: u32) {
    let sig = NodeIdentity::new(&mut rand::thread_rng());
    let strength = sig.strength();
    if strength > min_strength {
        tx.send((sig, strength))
            .map_err(|_| "Could not send keys!")
            .unwrap();
    }
}
