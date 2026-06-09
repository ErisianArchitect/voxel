This tool is used to modify the headers of source files within the project all at once.

Simply run with `cargo run` or `cargo run --release`. This will iterate through all the files whose
paths are included in in `tools/headers/locations.json` and set the header for that file based on
the text in `headers/src/header_text.txt`. The text will be wrapped and prepended with `//  `. for
correct wrapping of the text, ensure that you do not add any new lines to sequences that you want
wrapped.

The schema for `locations.json` is as follows:
```json
{
  "locations": [
    {
      "root": "relative/path/to/root/1",
      "include": [
        "**/*.rs"
      ]
    },
    {
      "root": "relative/path/to/root/2",
      "include": [
        "**/*.rs"
      ],
      "exclude": [
        "**/.exclude/*.rs"
      ]
    },
  ]
}
```
The location's root is relative to the workspace root. Each glob in `include`/`exclude`
is relative to the location's root. `exclude` is optional, but `include` is required.
Paths that do not match any of the globs in `include` will not be included. Any paths
that match any of the globs in `exclude` will also not be included.
