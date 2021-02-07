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

    Describe "CLI mount"
        SAMPLE_FILE=spec/resources/sample.mp4

        setup() {
            # Avoiding Mac/Linux issues with mktemp differences
            temp_directory="${SHELLSPEC_TMPBASE}/tmp.${RANDOM}.${RANDOM}"
            mkdir -p "${temp_directory}"
            mount_directory="${temp_directory}/mount"
        }

        cleanup() {
            case "$(uname -s)" in
                Darwin)
                    umount_command="diskutil umount force"
                    ;;
                *)
                    umount_command="umount -f"
                    ;;
            esac
            ${umount_command} "${mount_directory}" &> "${temp_directory}/umount.txt"
        }

        mount_and_wait_until_ready() {
            local mount_directory="${1:-"${mount_directory}"}"
            local video_file="${2:-"${SAMPLE_FILE}"}"

            RUST_LOG=info tool --logfile "${temp_directory}/mount.log" "${video_file}" "${mount_directory}"

            while [[ ! $(ls -A "${mount_directory}") ]]; do
                sleep 0.01
            done
        }

        extract_frame() {
            local frame_number="$1"
            local output_file="$2"
            local video_file="${3:-"${SAMPLE_FILE}"}"
            ffmpeg -i "${video_file}" -vf "select=eq(n\,${frame_number})" -vframes 1 "${output_file}" \
                &> "${temp_directory}/ffmpeg.${RANDOM}.out"
        }

        get_mount_frame_location() {
            local frame_number="$1"
            local frame_type="${2:-original}"
            local image_type="${3:-png}"
            echo "${mount_directory}/by-frame/frame-${frame_number}/${frame_type}/frame-${frame_number}.${image_type}"
        }

        calculate_image_similarity() {
            local image_1_location="$1"
            local image_2_location="$2"
            local decimal_places="${3:-8}"
            dssim "${image_1_location}" "${image_2_location}" \
                | awk "{printf \"%.${decimal_places}f\", \$1}"
        }

        BeforeEach "setup"
        AfterEach "cleanup"

        It "can mount to new directory"
            When call mount_and_wait_until_ready
            The status should equal 0
        End

        It "can mount to existing directory"
            BeforeCall "mkdir '${mount_directory}'"
            When call mount_and_wait_until_ready
            The status should equal 0
        End

        It "cannot mount to already mounted directory"
            BeforeCall "mount_and_wait_until_ready '${mount_directory}'"
            When call tool --foreground "${SAMPLE_FILE}" "${mount_directory}"
            The status should not equal 0
            The stderr should not equal ""
        End

        It "has expected directory structure"
            When call mount_and_wait_until_ready
            The status should equal 0
            The path "${mount_directory}/by-frame" should be directory
            The path "${mount_directory}/by-frame/frame-1" should be directory
            The path "${mount_directory}/by-frame/frame-1/original" should be directory
            The path "${mount_directory}/by-frame/frame-1/original/frame-1.jpg" should be file
        End

        It "can walk FUSE FS"
            BeforeCall mount_and_wait_until_ready
            When call find "${mount_directory}"
            The status should equal 0
            The output should not equal ""
        End

        Describe "frame views"
            Parameters
                original
                greyscale
                black-and-white
            End

            It "initialise $1 frame images"
                directory="${mount_directory}/by-frame/frame-27/$1"
                BeforeRun mount_and_wait_until_ready
                When run "${directory}/initialise.sh"
                The status should equal 0
                The stderr should not equal ""
                The path "${directory}/frame-27.png" should be file
                The path "${directory}/frame-27.jpg" should be file
                The path "${directory}/frame-27.webp" should be file
                The path "${directory}/frame-27.bmp" should be file
            End
        End

        It "can initialise more than once"
            directory="${mount_directory}/by-frame/frame-29/original"
            BeforeRun mount_and_wait_until_ready
            BeforeRun "${directory}/initialise.sh"
            When run "${directory}/initialise.sh"
            The status should equal 0
            The stderr should not equal ""
        End

        It "correct frame extracted"
            BeforeCall "extract_frame 1 '${temp_directory}/frame-1.png'"
            BeforeCall mount_and_wait_until_ready
            When call calculate_image_similarity "$(get_mount_frame_location 1)" "${temp_directory}/frame-1.png" 3
            The status should equal 0
            The output should equal 0.000
        End
    End
End
