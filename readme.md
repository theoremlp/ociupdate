# ociupdate

Companion CLI for working with a [custom lockfile format](oci.lock.schema.json) for [rules_oci](https://github.com/aspect-build/rules_oci).

## Updating Dependencies

ociupdate supports a simple strategy for updating ECR images in the lockfile assuming git-describe style versioning and immutable tags:

```sh
ociupdate --lockfile ./oci.lock.json update
```

Optionally allow snapshot versions (of the form `d.d.d-n-gcommitish`) by adding the flag `--allow-snapshots` after the `update` subcommand.

## Using with Bazel

Create a module extension:

```starlark
"OCI extension to translate lockfile to image pulls"

load("@rules_oci//oci:pull.bzl", "oci_pull")

lockfile = tag_class(
    attrs = {
        "lockfile": attr.label(mandatory = True, allow_single_file = True),
    },
)

def _extension(module_ctx):
    lockfiles = []
    for mod in module_ctx.modules:
        for tag in mod.tags.lockfile:
            lockfiles.append(tag.lockfile)

    for lockfile in lockfiles:
        images = json.decode(module_ctx.read(lockfile))
        if "v1" in images:
            for name, definition in images["v1"]["images"].items():
                oci_pull(
                    name = name,
                    digest = definition["digest"],
                    image = definition["image"],
                    platforms = definition.get("platforms"),
                )

oci = module_extension(
    implementation = _extension,
    tag_classes = {
        "lockfile": lockfile,
    },
)
```

In `MODULE.bazel`:
```
oci = use_extension("//tools/extensions:oci.bzl", "oci")
oci.lockfile(lockfile = "//:oci.lock.json")
use_repo(oci, "oci-image-1", "oci-image-2")
```
