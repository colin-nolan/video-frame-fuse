# Video Frame FUSE

Will create `fuse-mount-location` if it does not exist.


## Docker
TODO...
In the root directory of the project:
```
DOCKER_BUILDKIT=1 docker build --target packager -t colin-nolan/video-frame-fuse .
```

```
docker run --privileged --device /dev/fuse --cap-add SYS_ADMIN --rm colin-nolan/video-frame-fuse <video-location> <fuse-mount-location>
``` 


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

```
DOCKER_BUILDKIT=1 docker build --target tester .
```

```
DOCKER_BUILDKIT=1 docker build --target formatter --tag colin-nolan/video-frame-fuse:formatter .
docker run -v "${PWD}:/repository:ro" --rm colin-nolan/video-frame-fuse:formatter /repository
```

```
DOCKER_BUILDKIT=1 docker build --target tester --tag colin-nolan/video-frame-fuse:tester .
docker run -u $(id -u):$(id -g) -v "${PWD}:/repository" --rm colin-nolan/video-frame-fuse:tester /repository/scripts/test/run-unit-tests.sh
```

```
export DYLD_FALLBACK_LIBRARY_PATH=/usr/local/Cellar/llvm/*/Toolchains/LLVM*.xctoolchain/usr/lib
```