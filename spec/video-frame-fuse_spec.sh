Describe "video-frame-fuse"
    tool() {
      "${TOOL:="$(cd "${script_directory}" && git rev-parse --show-toplevel)/target/release/video-frame-fuse"}" "$@"
    }

    Describe "CLI"
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

    Describe "CLI usage"
        SAMPLE_FILE=spec/resources/sample.mp4

        setup() {
            mount_directory="$(mktemp -d)"
        }

        cleanup() {
            umount -f "${mount_directory}" 2> /dev/null || true
            rm -rf "${mount_directory}"
        }

        mount_and_wait_until_ready() {
            local sample_file="${1:-"${SAMPLE_FILE}"}"
            local mount_directory="${2:-"${mount_directory}"}"
            tool "${sample_file}" "${mount_directory}"
            while [[ ! "$(ls -A "${mount_directory}")" ]];
                do sleep 0.01;
            done
        }

        BeforeEach "setup"
        AfterEach "cleanup"

        It "can successfully mount"
            When call mount_and_wait_until_ready
            The status should equal 0
        End

        It "has expected directory structure"
            # shellcheck disable=SC2119
            mount_and_wait_until_ready
            The path "${mount_directory}/by-frame" should be directory
            The path "${mount_directory}/by-frame/frame-1" should be directory
            The path "${mount_directory}/by-frame/frame-1/original" should be directory
            The path "${mount_directory}/by-frame/frame-1/original/frame-1.jpg" should be file
        End

        It "can traverse FUSE tree"
            tool "${SAMPLE_FILE}" "${mount_directory}"
            When call find "${mount_directory}"
            The status should equal 0
            The output should not equal ""
        End
    End
End
