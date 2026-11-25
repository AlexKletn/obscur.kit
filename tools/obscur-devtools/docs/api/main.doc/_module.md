The `obscur-devtools` CLI entry point (spec §7.6): a thin argument parser over
[`obscur_devtools::cli`] (the client interface) and [`obscur_devtools::host`]
(a self-contained demo host to attach to).

```text
obscur-devtools serve [--tcp | --unix <path>] [--token <t>] [--scopes <list>]
obscur-devtools inspect <target|all> --tcp <addr> | --unix <path> [--token <t>]
obscur-devtools subscribe <topic>    --tcp <addr> | --unix <path> [--token <t>]
obscur-devtools ping                 --tcp <addr> | --unix <path> [--token <t>]
```
