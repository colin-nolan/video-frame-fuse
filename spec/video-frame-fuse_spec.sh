Describe "Example specfile"
  setup() { :; }
  cleanup() { :; }

  BeforeEach "setup"
  AfterEach "cleanup"

  Describe "hello()"
    tool() {
      "${TOOL:=video-frame-fuse}" "$@"
    }

    It "puts greeting, but not implemented"
      When call tool --version
      The output should eq "hello world"
    End
  End
End
