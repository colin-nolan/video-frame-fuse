Describe "Example specfile"
  setup() { :; }
  cleanup() { :; }

  BeforeEach "setup"
  AfterEach "cleanup"

  tool() {
    "${TOOL:="$(cd "${script_directory}" && git rev-parse --show-toplevel)/target/release/video-frame-fuse"}" "$@"
  }
  
  It "has a --version flag"
    When call tool --version
    The status should equal 0
    The output should not equal ""
  End
  
  It "has a --help flag"
    When call tool --help
    The status should equal 0
    The output should not equal ""
  End
End
