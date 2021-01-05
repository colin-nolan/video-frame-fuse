# Video Frame FUSE



## Dependencies
FUSE must be installed to build or run programs that use fuse-rs

See more:
https://github.com/zargony/fuse-rs/blob/master/README.md#dependencies

### Linux (Debian)
To install on a Debian based system:
```sh 
apt-get install fuse
```

### macOS
Installer packages can be downloaded from the [FUSE for macOS](https://osxfuse.github.io/).

To install using Homebrew:
```sh
brew cask install osxfuse
```

## Development
To build, FUSE libraries and headers are required. The package is usually called `libfuse-dev` or `fuse-devel`. 
Also `pkg-config` is required for locating libraries and headers.
