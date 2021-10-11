![CI](https://github.com/colin-nolan/video-frame-fuse/workflows/CI/badge.svg)

# Video Frame FUSE

Will create `fuse-mount-location` if it does not exist.


## Docker
TODO...
In the root directory of the project:
```
DOCKER_BUILDKIT=1 docker build --target production -t colinnolan/video-frame-fuse .
```

```
docker run --privileged --device /dev/fuse --cap-add SYS_ADMIN --rm colinnolan/video-frame-fuse <video-location> <fuse-mount-location>
``` 


## Dependencies
FUSE must be installed to build or run programs that use fuse-rs

See more:
https://github.com/zargony/fuse-rs/blob/master/README.md#dependencies

### Linux (Debian)
To install on a Debian based system:
```sh 
apt install fuse
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

### Testing
```
DOCKER_BUILDKIT=1 docker build --target tester --tag colinnolan/video-frame-fuse:tester .
docker run -u $(id -u):$(id -g) -v "${PWD}:/repository" --rm colinnolan/video-frame-fuse:tester /repository/scripts/test/run-unit-tests.sh
```

### Code Formatting
```
DOCKER_BUILDKIT=1 docker build --target formatter --tag colinnolan/video-frame-fuse:formatter .
docker run -u $(id -u):$(id -g) -v "${PWD}:/repository" --rm --workdir /repository colinnolan/video-frame-fuse:formatter
```

### Mac Development
```
export DYLD_FALLBACK_LIBRARY_PATH=/usr/local/Cellar/llvm/*/Toolchains/LLVM*.xctoolchain/usr/lib
```

