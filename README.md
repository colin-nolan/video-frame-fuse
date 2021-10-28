[![CI](https://github.com/colin-nolan/video-frame-fuse/workflows/CI/badge.svg)](https://github.com/colin-nolan/video-frame-fuse/actions)

# Video Frame FUSE
Will create `fuse-mount-location` if it does not exist.
```bash
USAGE:
    video-frame-fuse [FLAGS] [OPTIONS] <video-location> <fuse-mount-location>

FLAGS:
        --foreground    run in foreground (default is to daemonize)
    -h, --help          Prints help information
    -V, --version       Prints version information

OPTIONS:
        --logfile <logfile>    write logs to this location when demonized (not in foreground)

ARGS:
    <video-location>         location of the video file to use
    <fuse-mount-location>    location of directory to mount fuse

Setting RUST_LOG to one of {error, warn, info debug, trace} will set the logging verbosity, e.g. RUST_LOG=info
```

## Docker
TODO...
In the root directory of the project:
```bash
DOCKER_BUILDKIT=1 docker build --target production -t colinnolan/video-frame-fuse .
```

```bash
docker run --privileged --device /dev/fuse --cap-add SYS_ADMIN --rm colinnolan/video-frame-fuse <video-location> <fuse-mount-location>
``` 


## Dependencies
FUSE must be installed to build or run programs that use fuse-rs.

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
#### Unit
##### Local
```bash
 ./scripts/test/run-unit-tests.sh
```

##### Docker
```bash
DOCKER_BUILDKIT=1 docker build --target tester --tag colinnolan/video-frame-fuse:tester .
docker run -u $(id -u):$(id -g) -v "${PWD}:/repository" --rm colinnolan/video-frame-fuse:tester /repository/scripts/test/run-unit-tests.sh
```

#### Code Formatting
##### Local
```bash
 ./scripts/test/run-style-check.sh
```

##### Docker
```
DOCKER_BUILDKIT=1 docker build --target formatter --tag colinnolan/video-frame-fuse:formatter .
docker run -u $(id -u):$(id -g) -v "${PWD}:/repository" --rm --workdir /repository colinnolan/video-frame-fuse:formatter
```

#### Acceptance
##### Local
Before running the acceptance tests, build the software with:
```bash
cargo build
```

Run the tests:
```bash
./scripts/test/run-acceptance-tests.sh [shellspec-args]
```
*Note: see the [testing section in the Dockerfile](Dockerfile) for details about what tooling is required to run the 
tests*

##### Docker
```bash
# TODO
```


### Mac Development
[See README for OpenCV Rust library](https://github.com/twistedfall/opencv-rust#macos-package), which also includes a 
troubleshooting section. If the build fails with a `dyld: Library not loaded: @rpath/libclang.dylib` error message, and
you are using Command Line Tools, try setting:
```
export DYLD_FALLBACK_LIBRARY_PATH="$(xcode-select --print-path)/usr/lib/"
```
