# yggdrasil-keygen

A small executable to generate yggdrasil keys and output them to stdout as a
json blob. The tool keeps a cache of good keys generated so far, so that it
doesn't have to start from scratch each time, and the keys get better over time.
It generates 2^16 key pairs each time, for both signing keys and encryption
keys, keeping the best 2^8 key pairs of each type in it's cache. If a run get's
lucky and outputs multiple very good keys in one run, those are kept in the
cache for the next time.  At the end of each run, the best pair of each type
that's currently in the cache will be used for the json output.

## Usage

```bash
$ yggdrasil-keygen
{
  "sig_pub": "0c05f9c92fbf3ca279000ff61b61a8b37a5f9431345449a45f8771386ca3d6ad",
  "sig_sec": "31a9eedc2aef067ea861f853fbb350f8308525f22fc87ccb89619988bf7c9e67",
  "enc_pub": "881fe25f513d056120442c9b1a34a4df17eae8be59391e43863827bfe5e93c7a",
  "enc_sec": "8456a4763e2e6e77143e6d79a2b25b39565bbdcdf807682683f65e75401cdc49",
  "address": "210:7ed:d745:d2bb:36d6:519:49c0:b18f"
}
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License
[AGPL-3.0-only](https://choosealicense.com/licenses/agpl-3.0/)
