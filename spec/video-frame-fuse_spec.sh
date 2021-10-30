Describe "video-frame-fuse"
    script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" > /dev/null 2>&1 && pwd)"
    repository_root_directory="$(cd "${script_directory}" && git rev-parse --show-toplevel)"
    tool_location="${TOOL:="${repository_root_directory}/target/release/video-frame-fuse"}"

    tool() {
        if [[ ! -f "${tool_location}" ]]; then
            >&2 echo "Tool does not exist in location: ${tool_location}"
            exit 1
        fi
        "${tool_location}" "$@"
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
                    if [[ "${EUID}" -eq 0 ]]; then
                        umount_command="umount -f"
                    else
                        umount_command="umount"
                    fi
                    ;;
            esac
            ${umount_command} "${mount_directory}" &> "${temp_directory}/umount.txt"
        }

        mount_and_wait_until_ready() {
            local mount_directory="${1:-"${mount_directory}"}"
            local video_file="${2:-"${SAMPLE_FILE}"}"

            RUST_LOG=info tool --logfile "${temp_directory}/mount.log" "${video_file}" "${mount_directory}"

            timeout_at=$(( "$(date +%s)" + 5 ))
            while [[ ! $(ls -A "${mount_directory}" 2> /dev/null) ]]; do
                sleep 0.01
                if [[ "$(date +%s)" -gt "${timeout_at}" ]]; then
                    >&2 echo "Timed out waiting for mount to become available"
                    if [[ -f "${temp_directory}/mount.log" ]]; then
                        >&2 cat "${temp_directory}/mount.log"
                    else
                        >&2 echo "(No logs available)"
                    fi
                    exit 1
                fi
            done
        }

        extract_frame() {
            local frame_number="$1"
            local output_file="$2"
            local video_file="${3:-"${SAMPLE_FILE}"}"
            local extra_video_filters=""
            if [[ -n "$4" ]]; then
                extra_video_filters=",$4"
            fi

            local logs_location="${temp_directory}/ffmpeg.${RANDOM}.out"
            ffmpeg -i "${video_file}" -vf "select=eq(n\,${frame_number})${extra_video_filters}" -vframes 1 "${output_file}" \
                2> "${logs_location}"

            if [[ ! -f "${output_file}" ]]; then
                >&2 echo "Output file not produced - ffmpeg logs: $(cat "${logs_location}")"
                exit 1
            fi
        }

        extract_greyscale_frame() {
            local frame_number="$1"
            local output_file="$2"
            local video_file="${3:-"${SAMPLE_FILE}"}"
            extract_frame "${frame_number}" "${output_file}" "${video_file}" hue=s=0
        }

        extract_black_and_white_frame() {
            local frame_number="$1"
            local output_file="$2"
            local threshold_percentage="${3:-50}"
            local video_file="${4:-"${SAMPLE_FILE}"}"

            local grayscale_frame_location="${temp_directory}/bw-greyscale.${RANDOM}.${output_file##*.}"
            extract_greyscale_frame "${frame_number}" "${grayscale_frame_location}" "${video_file}"

            local logs_location="${temp_directory}/bw-convert.${RANDOM}.out"
            convert "${grayscale_frame_location}" -threshold "${threshold_percentage}%" "${output_file}" 2> "${logs_location}"

            if [[ ! -f "${output_file}" ]]; then
                >&2 echo "Output file not produced - ImageMagick convert logs: $(cat "${logs_location}")"
                exit 1
            fi
        }

        get_mount_frame_location() {
            local frame_number="$1"
            local frame_type="${2:-original}"
            local image_type="${3:-png}"
            echo "${mount_directory}/by-frame/frame-${frame_number}/${frame_type}/frame-${frame_number}.${image_type}"
        }

        change_config() {
            local frame_number="$1"
            local frame_type="$2"
            local property="$3"
            local value="$4"

            local config_location="${mount_directory}/by-frame/frame-${frame_number}/${frame_type}/config.yml"
            local temp_config_location="${temp_directory}/${RANDOM}.config.yml"
            cp "${config_location}" "${temp_config_location}"
            # The in-place flag does not work if the config is in the mount directory, as yq wants to write a file in
            # the same directory as the file, which in this case is read-only.
            yq eval ".${property} = ${value}" -i "${temp_config_location}"
            cp "${temp_config_location}" "${config_location}"
        }

        calculate_image_similarity() {
            local image_1_location="$1"
            local image_2_location="$2"
            local decimal_places="${3:-8}"

            dssim "${image_1_location}" "${image_2_location}" \
                | awk "{printf \"%.${decimal_places}f\", \$1}"
        }

        get_number_of_colours() {
            local image_location="$1"
            "${repository_root_directory}/tests/acceptance/scripts/image/get-image-colours.py" "${image_location}" \
                | jq length
        }

        math_value() {
            local test_operator="$1"
            local operand_2="$2"
            # The subject is stored in the same variable name as the function name
            local operand_1="${math_value:?}"

            case "${test_operator}" in
                -lt)
                    operator="<"
                    ;;
                -le)
                    operator="<="
                    ;;
                -eq)
                    operator="=="
                    ;;
                -ge)
                    operator=">="
                    ;;
                -gt)
                    operator=">"
                    ;;
                *)
                    >&2 echo "Unsupported math operator: ${test_operator}"
                    exit 1
                    ;;
            esac

            python -c "import sys; sys.exit(0 if ${operand_1} ${operator} ${operand_2} else 1)"
        }

        BeforeEach "setup"
        AfterEach "cleanup"

        It "can mount to a new directory"
            When call mount_and_wait_until_ready
            The status should equal 0
        End

        It "can mount to an existing directory"
            BeforeCall "mkdir '${mount_directory}'"
            When call mount_and_wait_until_ready
            The status should equal 0
        End

        It "cannot mount to an already mounted directory"
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

        Describe "can initialise"
            Parameters
                original
                greyscale
                black-and-white
            End

            It "$1 frame images"
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

            It "$1 images more than once"
                directory="${mount_directory}/by-frame/frame-29/$1"
                BeforeRun mount_and_wait_until_ready
                BeforeRun "${directory}/initialise.sh"
                When run "${directory}/initialise.sh"
                The status should equal 0
                The stderr should not equal ""
            End
        End

        Describe "can deliver"
            It "original frame contents"
                BeforeCall "extract_frame 42 '${temp_directory}/frame-42.png'"
                BeforeCall mount_and_wait_until_ready
                When call calculate_image_similarity "$(get_mount_frame_location 42 original)" "${temp_directory}/frame-42.png"
                The status should equal 0
                The output should satisfy math_value -lt 0.01
            End

            It "greyscale frame contents"
                BeforeCall "extract_greyscale_frame 36 '${temp_directory}/frame-36.png'"
                BeforeCall mount_and_wait_until_ready
                When call calculate_image_similarity "$(get_mount_frame_location 36 greyscale)" "${temp_directory}/frame-36.png"
                The status should equal 0
                The output should satisfy math_value -lt 0.01
            End

            It "black-and-white frame contents"
                BeforeCall "extract_black_and_white_frame 13 '${temp_directory}/frame-13.png' 50"
                BeforeCall mount_and_wait_until_ready
                BeforeCall "change_config 13 black-and-white threshold 128"
                When call calculate_image_similarity "$(get_mount_frame_location 13 black-and-white)" "${temp_directory}/frame-13.png" 2
                The status should equal 0
                The output should satisfy math_value -lt 0.1
            End

            It "greyscale frame with number of colours in correct range"
                BeforeCall mount_and_wait_until_ready
                When call get_number_of_colours "$(get_mount_frame_location 13 greyscale)"
                The status should equal 0
                The output should satisfy math_value -le 256
                The output should satisfy math_value -gt 2
            End

            It "black-and-white frame with only 2 colours"
                BeforeCall mount_and_wait_until_ready
                When call get_number_of_colours "$(get_mount_frame_location 13 black-and-white)"
                The status should equal 0
                The output should equal 2
            End
        End
    End
End
