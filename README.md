# SPM_TO_GRAPH

A very quick and dirty tool to generate graphviz diagrams from Swift Package Manager packages.

## Install

```bash
cargo install --git https://github.com/schwa/spm_to_graph
```

graphviz _must_ be installed on your system.

## Build

```bash
brew install graphviz
```

## Usage

```plaintext
Usage: spm_to_graph [OPTIONS] [INPUT] [OUTPUT]

Arguments:
  [INPUT]   Optional name to operate on
  [OUTPUT]

Options:
      --skip-test-targets
      --skip-product-dependencies
  -h, --help                       Print help
  -V, --version                    Print version
  ```

```bash
spm_to_graph <path-to-package> <output-file>
```

Output files can either be .dot, .svg or .png.
