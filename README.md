# SPM_TO_GRAPH

A very quick and dirty tool to generate graphviz diagrams from Swift Package Manager packages.

## Install

```bash
git clone <TODO>
cargo install --path .
```

graphviz _must_ be installed on your system.

## Build

```bash
brew install graphviz
```

## Usage

```bash
spm_to_graph <path-to-package> <output-file>
```

Output files can either be .dot, .svg or .png.
