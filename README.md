# yggdrasil-keygen

A small executable to generate yggdrasil keys and output them to stdout as a
json blob. The tool keeps a cache of good keys generated so far, so that it
doesn't have to start from scratch each time, and the keys get better over
time. It generates 2^16 key pairs each time, keeping the best 2^8 key pairs in
it's cache. If a run get's lucky and outputs multiple very good keys in that
run, those are kept in the cache for the next time. At the end of each run, the
best pair of each type that's currently in the cache will be used for the json
output.

## Usage

```bash
$ yggdrasil-keygen
{
  "public": "00004799cfcbb26bdd56c1edcd684661db6b9f4e8dd0224c4936a42b77b8e04b",
  "secret": "f7355e4f4981e58830af206dc0c3e7731f597708f4364d1e50109b24f1339fab",
  "address": "211:e198:c0d1:3650:8aa4:f848:ca5e:e678"
}
```

If you're using `yggdrasil-keygen` with `yggdrasil-go`, you need to append the
public key to the private key in the config. `yggdrasil-go` uses the ed25519
implementation from the go stdlib, which appends the public key to the private
key.

## Contributing

Pull requests are welcome. For major changes, please open an issue first to
discuss what you would like to change.

Please make sure to update tests as appropriate.

## License
[AGPL-3.0-only](https://choosealicense.com/licenses/agpl-3.0/)
