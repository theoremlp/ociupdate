# ociupdate

Companion CLI for working with a [custom lockfile format](oci.lock.schema.json) for [rules_oci](https://github.com/aspect-build/rules_oci).

## Updating Dependencies

ociupdate supports a simple strategy for updating ECR images in the lockfile assuming git-describe style versioning and immutable tags:

```sh
ociupdate --lockfile ./oci.lock.json update
```

Optionally allow snapshot versions (of the form `d.d.d-n-gcommitish`) by adding the flag `--allow-snapshots` after the `update` subcommand.
