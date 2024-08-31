use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom},
    net::Ipv6Addr,
};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use fd_lock::{RwLock, RwLockWriteGuard};
use serde::{Deserialize, Serialize};
use serde_yaml::from_reader;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use yggdrasil_keys::NodeIdentity;

const KEY_CACHE_SIZE: usize = 2usize.pow(8);
const KEY_GEN_TRIES: usize = 2usize.pow(16);

#[derive(Serialize, Deserialize)]
struct CacheFile {
    keys: Vec<(String, u32)>,
}

impl CacheFile {
    fn new() -> Self {
        Self { keys: vec![] }
    }
}

#[derive(Default)]
struct Cache {
    keys: Vec<(NodeIdentity, u32)>,
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
        if self.keys.len() > KEY_CACHE_SIZE {
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
}

impl From<CacheFile> for Cache {
    fn from(cache_file: CacheFile) -> Self {
        Self {
            keys: cache_file
                .keys
                .iter()
                .map(|(key, strength)| (NodeIdentity::from_hex(key, None), strength))
                .filter(|(key, _strength)| key.is_ok())
                .map(|(key, strength)| (key.unwrap(), *strength))
                .collect(),
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
    let old_dirs = ProjectDirs::from("com", "Famedly GmbH", "Yggdrasil Key Generator")
        .context("Couldn't find old project directory, is $HOME set?")?;
    let dirs = ProjectDirs::from("", "", "yggdrasil-keygen")
        .context("Couldn't find project directory, is $HOME set?")?;

    let old_cache_path = old_dirs.cache_dir().join("cache.yaml");
    let cache_dir = dirs.cache_dir();
    std::fs::create_dir_all(&cache_dir)?;
    let cache_path = cache_dir.join("cache.yaml");

    if old_cache_path.exists() {
        std::fs::rename(&old_cache_path, &cache_path)?;
    }

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(cache_path)?;
    let mut lock = RwLock::new(file);
    let mut guard: RwLockWriteGuard<File> = lock.write().context("couldn't lock cache file")?;

    let cache: Cache = match guard.metadata()?.len() {
        0 => Cache::default(),
        _ => from_reader::<&std::fs::File, CacheFile>(&guard)
            .unwrap_or_else(|_| CacheFile::new())
            .into(),
    };

    let min_strength = cache.get_min_strength();

    let (tx, rx) = unbounded_channel();

    let cache_handle = tokio::spawn(receive_keys(rx, cache));
    for _ in 0..KEY_GEN_TRIES {
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
