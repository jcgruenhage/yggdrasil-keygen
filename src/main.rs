use std::{
    io::{Seek, SeekFrom},
    net::Ipv6Addr,
};

use anyhow::{Context, Result};
use directories_next::ProjectDirs;
use file_lock::FileLock;
use serde::{Deserialize, Serialize};
use serde_yaml::from_reader;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use yggdrasil_keys::{EncryptionKeys, SigningKeys};

const KEY_CACHE_SIZE: usize = 2usize.pow(8);
const KEY_GEN_TRIES: usize = 2usize.pow(16);

#[derive(Serialize, Deserialize)]
struct CacheFile {
    sig_keys: Vec<(String, u32)>,
    enc_keys: Vec<(String, u32)>,
}

#[derive(Default)]
struct Cache {
    sig_keys: Vec<(SigningKeys, u32)>,
    enc_keys: Vec<(EncryptionKeys, u32)>,
}

#[derive(Serialize, Deserialize)]
struct Output {
    sig_pub: String,
    sig_sec: String,
    enc_pub: String,
    enc_sec: String,
    address: Ipv6Addr,
}

impl Cache {
    fn add_sig_key(&mut self, key: SigningKeys, strength: u32) {
        self.sig_keys.push((key, strength));
        self.sig_keys.sort_by(|a, b| a.1.cmp(&b.1).reverse());
        if self.sig_keys.len() > KEY_CACHE_SIZE {
            self.sig_keys.pop();
        }
    }
    fn get_min_sig_strength(&self) -> u32 {
        self.sig_keys
            .last()
            .map(|(_key, strength)| *strength)
            .unwrap_or(0)
    }
    fn add_enc_key(&mut self, key: EncryptionKeys, strength: u32) {
        self.enc_keys.push((key, strength));
        self.enc_keys.sort_by(|a, b| a.1.cmp(&b.1).reverse());
        if self.enc_keys.len() > KEY_CACHE_SIZE {
            self.enc_keys.pop();
        }
    }
    fn get_min_enc_strength(&self) -> u32 {
        self.enc_keys
            .last()
            .map(|(_key, strength)| *strength)
            .unwrap_or(0)
    }
    fn output(&mut self) -> Output {
        let sig = self.sig_keys.remove(0).0;
        let (sig_pub, sig_sec) = sig.to_hex_split();
        let enc = self.enc_keys.remove(0).0;
        let (enc_pub, enc_sec) = enc.to_hex_split();
        Output {
            sig_pub,
            sig_sec,
            enc_pub,
            enc_sec,
            address: enc.node_id().address(),
        }
    }
}

impl Into<Cache> for CacheFile {
    fn into(self) -> Cache {
        Cache {
            sig_keys: self
                .sig_keys
                .iter()
                .map(|(key, strength)| (SigningKeys::from_hex(key, None), strength))
                .filter(|(key, _strength)| key.is_ok())
                .map(|(key, strength)| (key.unwrap(), *strength))
                .collect(),
            enc_keys: self
                .enc_keys
                .iter()
                .map(|(key, strength)| (EncryptionKeys::from_hex(key, None), strength))
                .filter(|(key, _strength)| key.is_ok())
                .map(|(key, strength)| (key.unwrap(), *strength))
                .collect(),
        }
    }
}

impl Into<CacheFile> for Cache {
    fn into(self) -> CacheFile {
        CacheFile {
            sig_keys: self
                .sig_keys
                .iter()
                .map(|(key, strength)| (key.to_hex_joined(), *strength))
                .collect(),
            enc_keys: self
                .enc_keys
                .iter()
                .map(|(key, strength)| (key.to_hex_joined(), *strength))
                .collect(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let dirs = ProjectDirs::from("com", "Famedly GmbH", "Yggdrasil Key Generator")
        .context("Couldn't find project directory, is $HOME set?")?;

    let cache_dir = dirs.cache_dir();
    std::fs::create_dir_all(&cache_dir)?;
    let cache_path = cache_dir.join("cache.yaml");

    let mut lock = FileLock::lock(
        cache_path
            .to_str()
            .context("couldn't convert cache path to string")?,
        true,
        true,
    )
    .context("couldn't lock cache file")?;

    let cache: Cache = match lock.file.metadata()?.len() {
        0 => Cache::default(),
        _ => from_reader::<&std::fs::File, CacheFile>(&lock.file)
            .context("couldn't read cache from locked file")?
            .into(),
    };

    let min_sig_strength = cache.get_min_sig_strength();
    let min_enc_strength = cache.get_min_enc_strength();

    let (sig_tx, sig_rx) = unbounded_channel();
    let (enc_tx, enc_rx) = unbounded_channel();

    let cache_handle = tokio::spawn(receive_keys(sig_rx, enc_rx, cache));
    for _ in 0..KEY_GEN_TRIES {
        tokio::spawn(generate_sig_keys(sig_tx.clone(), min_sig_strength));
        tokio::spawn(generate_enc_keys(enc_tx.clone(), min_enc_strength));
    }
    drop(sig_tx);
    drop(enc_tx);

    let mut cache = cache_handle.await?;

    serde_json::to_writer_pretty(std::io::stdout(), &cache.output())?;

    lock.file.seek(SeekFrom::Start(0))?;

    serde_yaml::to_writer::<&std::fs::File, CacheFile>(&lock.file, &cache.into())?;

    lock.unlock()?;
    Ok(())
}

async fn receive_keys(
    mut sig_rx: UnboundedReceiver<(SigningKeys, u32)>,
    mut enc_rx: UnboundedReceiver<(EncryptionKeys, u32)>,
    mut cache: Cache,
) -> Cache {
    while let Some(sig) = sig_rx.recv().await {
        cache.add_sig_key(sig.0, sig.1);
    }
    while let Some(enc) = enc_rx.recv().await {
        cache.add_enc_key(enc.0, enc.1);
    }
    cache
}

async fn generate_sig_keys(tx: UnboundedSender<(SigningKeys, u32)>, min_sig_strength: u32) {
    let sig = SigningKeys::new(&mut rand::thread_rng());
    let strength = sig.tree_id().strength();
    if strength > min_sig_strength {
        tx.send((sig, strength))
            .map_err(|_| "Could not send keys!")
            .unwrap();
    }
}

async fn generate_enc_keys(tx: UnboundedSender<(EncryptionKeys, u32)>, min_enc_strength: u32) {
    let enc = EncryptionKeys::new(&mut rand::thread_rng());
    let strength = enc.node_id().strength();
    if strength > min_enc_strength {
        tx.send((enc, strength))
            .map_err(|_| "Could not send keys!")
            .unwrap();
    }
}
