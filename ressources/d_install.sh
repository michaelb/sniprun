#!/usr/bin/env bash
#
# Copyright: Copyright Martin Nowak 2015 -.
# License: Boost License 1.0 (www.boost.org/LICENSE_1_0.txt)
# Authors: Martin Nowak
# Documentation: https://dlang.org/install.html

_() {

# Returns false if the script is invoked from a Windows command prompt.
posix_terminal() {
    # If run from a POSIX terminal (including under MSYS2 or Cygwin) the TERM
    # variable is defined to something like "xterm". If run from a Windows
    # command prompt through bash.exe from an MSYS installation, TERM keeps
    # its default value, which is "cygwin".
    if [[ "${TERM:-noterm}" == "cygwin" ]]; then
        false
    else
        true
    fi
}

if ! posix_terminal; then
    # We have been invoked from Windows cmd. Run the login script.
    # (Cannot use --login bash option, as that runs /etc/bash.bash_logout
    # afterwards which typically clears the screen, removing any output.)
    if [ -r /etc/profile ]; then
        # shellcheck disable=SC1091
        source /etc/profile
    else
        fatal "Failed to source /etc/profile.";
    fi
fi

# Earliest opportunity for security settings, tolerate insecure /etc/profile.
set -ueo pipefail

# ------------------------------------------------------------------------------

log_() {
    local tilde='~'
    echo "${@//$HOME/$tilde}"
}

log() {
    if [ "$VERBOSITY" -gt 0 ]; then
        log_ "$@"
    fi
}

logV() {
    if [ "$VERBOSITY" -gt 1 ]; then
        log_ "$@"
    fi
}

logE() {
    log_ "$@" >&2
}

fatal() {
    logE "$@"
    exit 1
}

# ------------------------------------------------------------------------------
curl2() {
    : "${CURL_USER_AGENT:="installer/install.sh $(command curl --version | head -n 1)"}"
    TIMEOUT_ARGS=(--connect-timeout 5 --speed-time 30 --speed-limit 1024)
    command curl --fail "${TIMEOUT_ARGS[@]}" -L -A "$CURL_USER_AGENT" "$@"
}

curl() {
    if [ "$VERBOSITY" -gt 0 ]; then
        curl2 -# "$@"
    else
        curl2 -sS "$@"
    fi
}

retry() {
    for i in {0..4}; do
        if "$@"; then
            break
        elif [ $i -lt 4 ]; then
            sleep $((1 << i))
        else
            fatal "Failed to download '$url'"
        fi
    done
}

# path, verify (0/1), urls...
# the gpg signature is assumed to be url+.sig
download() {
    local path do_verify mirrors
    path="$1"
    do_verify="$2"
    mirrors=("${@:3}")

    try_all_mirrors() {
        for i in "${!mirrors[@]}" ; do
            if [ "$i" -gt 0 ] ; then
                log "Falling back to mirror: ${mirrors[$i]}"
            fi
            if curl "${mirrors[$i]}" -o "$path" ; then
                return
            fi
        done
        return 1
    }
    retry try_all_mirrors
    if [ "$do_verify" -eq 1 ]; then
        verify "$path" "${mirrors[@]/%/.sig}"
    fi
}

# path, urls...
download_with_verify() {
    download "$1" 1 "${@:2}"
}

# path, urls...
download_without_verify() {
    download "$1" 0 "${@:2}"
}

# urls...
fetch() {
    local mirrors path
    path=$(mktemp "$TMP_ROOT/XXXXXX")
    mirrors=("$@")

    try_all_mirrors() {
        for mirror in "${mirrors[@]}" ; do
            if curl2 -sS "$mirror" -o "$path" ; then
                return
            fi
        done
        return 1
    }
    retry try_all_mirrors
    cat "$path"
    rm "$path"
}

# ------------------------------------------------------------------------------

HAVE_CYGPATH=no
if command -v cygpath &>/dev/null; then
    HAVE_CYGPATH=yes
fi
posix_path() {
    if [[ "$HAVE_CYGPATH" == "yes" ]]; then
        cygpath "$1"
    else
        echo "$1"
    fi
}
display_path() {
    if [[ "$HAVE_CYGPATH" == "yes" ]]; then
        if posix_terminal; then
           cygpath "$1"
        else
           cygpath -w "$1"
        fi
    else
        echo "$1"
    fi
}

COMMAND=
COMPILER=dmd
VERBOSITY=1
GET_PATH_AUTO_INSTALL=0
GET_PATH_COMPILER=dc
# Set a default install path depending on the POSIX/Windows environment.
if posix_terminal; then
    ROOT=~/dlang
else
    # Default to a ROOT that is outside the POSIX-like environment.
    if [ -z "$USERPROFILE" ]; then
        fatal '%USERPROFILE% should not be empty on Windows.';
    fi
    ROOT=$(posix_path "$USERPROFILE")/dlang
fi
TMP_ROOT=
DUB_VERSION=
DUB_BIN_PATH=
case $(uname -s) in
    Darwin) OS=osx;;
    Linux) OS=linux;;
    FreeBSD) OS=freebsd;;
    *_NT-*) OS=windows;;
    *)
        fatal "Unsupported OS $(uname -s)"
        ;;
esac

check_tools() {
    while [[ $# -gt 0 ]]; do
        if ! command -v "$1" &>/dev/null &&
                # detect OSX' liblzma support in libarchive
                ! { [ "$OS-$1" == osx-xz ] && otool -L /usr/lib/libarchive.*.dylib | grep -qF liblzma; }; then
            local msg="Required tool $1 not found, please install it."
            case $OS-$1 in
                osx-xz) msg="$msg http://macpkg.sourceforge.net";;
            esac
            fatal "$msg"
        fi
        shift
    done
}

# ------------------------------------------------------------------------------

mkdtemp() {
    mktemp -d "$TMP_ROOT/XXXXXX"
}

cleanup() {
    if [[ -n $TMP_ROOT ]]; then
        rm -rf "$TMP_ROOT";
    fi
}
trap cleanup EXIT

# ------------------------------------------------------------------------------

usage() {
    log 'Usage

  install.sh [<command>] [<args>]

Commands

  install       Install a D compiler (default command)
  uninstall     Remove an installed D compiler
  list          List all installed D compilers
  update        Update this dlang script
  get-path      Path of the installed compiler executable

Options

  -h --help     Show this help
  -p --path     Install location
                  POSIX default:   ~/dlang
                  Windows default: %USERPROFILE%\dlang
  -v --verbose  Verbose output

Run "install.sh <command> --help to get help for a specific command, or consult
https://dlang.org/install.html for documentation.
If no arguments are provided, the latest DMD compiler will be installed.
'
}

command_help() {
    if [ -z "${1-}" ]; then
        usage
        return
    fi

    local _compiler='Compiler

  dmd|gdc|ldc           latest version of a compiler
  dmd|gdc|ldc-<version> specific version of a compiler (e.g. dmd-2.071.1, ldc-1.1.0-beta2)
  dmd|ldc-beta          latest beta version of a compiler
  dmd-nightly           latest dmd nightly
  ldc-latest-ci         latest ldc CI build (with assertions enabled)
  dmd-2016-08-08        specific dmd nightly
'

    case $1 in
        install)
            log 'Usage

  install.sh install <compiler>

Description

  Download and install a D compiler.
  By default the latest release of the DMD compiler is selected.

Options

  -a --activate     Only print the path to the activate script

Examples

  install.sh
  install.sh dmd
  install.sh install dmd
  install.sh install dmd-2.071.1
  install.sh install ldc-1.1.0-beta2
'
            log "$_compiler"
            ;;

        uninstall)
            log 'Usage

  install.sh uninstall <compiler>

Description

  Uninstall a D compiler.

Examples

  install.sh uninstall dmd
  install.sh uninstall dmd-2.071.1
  install.sh uninstall ldc-1.1.0-beta2
'
            log "$_compiler"
            ;;

        list)
            log 'Usage

  install.sh list

Description

  List all installed D compilers.
'
            ;;

        update)
            log 'Usage

  install.sh update

Description

  Update the dlang installer itself and the keyring.
'
            ;;

        get-path)
            log 'Usage

  install.sh get-path <compiler>

Description

  Find the path of an installed D compiler.

Options

  --install         Install the compiler if it is not installed
  --dmd             Find the DMD-alike interface instead
  --dub             Find the DUB instance

Examples

  install.sh
  install.sh get-path dmd
  install.sh get-path dmd-2.093.0 --install
  install.sh get-path --dmd ldc-1.19.0
  install.sh get-path --dub ldc-1.19.0
  install.sh get-path ldc-1.19.0 --install
  install.sh get-path --dub dub-1.21.0
'
            log "$_compiler"
            ;;

    esac
}

# ------------------------------------------------------------------------------

parse_args() {
    local _help=
    local _installAction=
    local _getPathAction=
    local _autoInstallFlag=

    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h | --help)
                _help=1
                ;;

            -p | --path)
                if [ -z "${2:-}" ]; then
                    fatal '-p|--path must be followed by a path.';
                fi
                shift
                ROOT="$(posix_path "$1")";
                ;;

            -v | --verbose)
                VERBOSITY=2
                ;;

            -a | --activate)
                if [ -n "$_installAction" ]; then
                    fatal "$1 conflicts with --${_installAction}"
                fi
                _installAction="activate"
                ;;

            --dmd)
                if [ -n "$_getPathAction" ]; then
                    fatal "$1 conflicts with --${_getPathAction}"
                fi
                _getPathAction="dmd"
                ;;

            --dub)
                if [ -n "$_getPathAction" ]; then
                    fatal "$1 conflicts with --${_getPathAction}"
                fi
                _getPathAction="dub"
                ;;

            --install)
                _autoInstallFlag=1
                ;;

            use | install | uninstall | list | update)
                COMMAND=$1
                ;;

            get-path)
                COMMAND=$1
                ;;

            remove)
                COMMAND=uninstall
                ;;

            dmd | dmd-* | gdc | gdc-* | ldc | ldc-* | dub | dub-* | \
            dmd,* | ldc,* | gdc,* )
                COMPILER=$1
                ;;

            *)
                usage
                fatal "Unrecognized command-line parameter: $1"
                ;;
        esac
        shift
    done

    mkdir -p "$ROOT"
    TMP_ROOT=$(mktemp -d "$ROOT/.installer_tmp_XXXXXX")

    if [ -n "$_help" ]; then
        command_help $COMMAND
        exit 0
    fi
    local command_="${COMMAND:-install}"
    if [ -n "$_installAction" ] ; then
        if [ "$command_" == "install" ]; then
            VERBOSITY=0
        else
            logE "ERROR: --activate is not allowed for ${command_}."
            command_help $COMMAND
            exit 1
        fi
    fi
    if [ -n "$_autoInstallFlag" ] ; then
        if [ "$command_" == "get-path" ]; then
            GET_PATH_AUTO_INSTALL=1
        else
            logE "ERROR: --install is not allowed for ${command_}."
            command_help $COMMAND
            exit 1
        fi
    fi
    if [ -n "$_getPathAction" ] ; then
        if [ "$command_" == "get-path" ]; then
            case "$_getPathAction" in
                dmd|dub)
                    GET_PATH_COMPILER=$_getPathAction
                    ;;
                *)
                    fatal "Internal Error. Invalid get-path: $_getPathAction"
            esac
        else
            logE "ERROR: --${_getPathAction} is not allowed for ${command_}."
            command_help $COMMAND
            exit 1
        fi
    fi

    IFS="," read -r _compiler _dub <<< "$COMPILER"
    if [ -n "${_dub:-}" ] ; then
        COMPILER="$_compiler"
        DUB="$_dub"
    fi
}

# ------------------------------------------------------------------------------

# run_command command [compiler]
run_command() {
    case $1 in
        install)
            install_d "$1" "${2:-}"
            if posix_terminal; then
                if [ "$(basename "$SHELL")" = fish ]; then
                    local suffix=.fish
                fi
                if [ "$VERBOSITY" -eq 0 ]; then
                    echo "$ROOT/$2/activate${suffix:-}"
                else
                    log "
Run \`source $ROOT/$2/activate${suffix:-}\` in your shell to use $2.
This will setup PATH, LIBRARY_PATH, LD_LIBRARY_PATH, DMD, DC, and PS1.
Run \`deactivate\` later on to restore your environment."
                fi
            else
                if [ "$VERBOSITY" -eq 0 ]; then
                    display_path "$ROOT/$2/activate.bat"
                else
                    log "
Run \`$(display_path "$ROOT/$2/activate.bat")\` to add $2 to your PATH."
                fi
            fi
            ;;

        get-path)
            if [ $GET_PATH_AUTO_INSTALL -eq 0 ] ; then
                if [ ! -d "$ROOT/$2" ]; then
                    fatal "Requested $2 is not installed. Install or rerun with --install.";
                fi
                if [ "$GET_PATH_COMPILER" == "dub" ] ; then
                    local dubBinPath
                    dubBinPath="$(binexec_for_dub_compiler "$2")"
                    if [ ! -f "$dubBinPath" ] ; then
                        fatal "Requested DUB is not installed. Install or rerun with --install.";
                    fi
                fi
            fi
            previousVerbosity=$VERBOSITY
            VERBOSITY=-1
            install_d "$1" "${2:-}"
            VERBOSITY=$previousVerbosity
            case "$GET_PATH_COMPILER" in
                dmd)
                    if [[ $COMPILER =~ ^dub ]] ; then
                        fatal "ERROR: DUB is not a compiler."
                    fi
                    echo "$ROOT/$2/$(binexec_for_dmd_compiler "$2")"
                    ;;
                dc)
                    if [[ $COMPILER =~ ^dub ]] ; then
                        fatal "ERROR: DUB is not a compiler."
                    fi
                    echo "$ROOT/$2/$(binexec_for_dc_compiler "$2")"
                    ;;
                dub)
                    binexec_for_dub_compiler "$2"
                    ;;
                *)
                fatal "Unknown value for GET_PATH_COMPILER encountered."
            esac
            ;;

        uninstall)
            if [ -z "${2:-}" ]; then
                fatal "Missing compiler argument for $1 command.";
            fi
            uninstall_compiler "$2"
            ;;

        list)
            list_compilers
            ;;

        update)
            install_dlang_installer
            ;;
    esac
}

install_d() {
    local commandName="$1"
    if [ -z "${2:-}" ]; then
        fatal "Missing compiler argument for $commandName command.";
    fi
    local compilerName="$2"
    check_tools curl
    if [ ! -f "$ROOT/install.sh" ] || [ ! -f "$ROOT/d-keyring.gpg" ] ; then
        install_dlang_installer
    fi
    if [ -d "$ROOT/$compilerName" ]; then
        log "$compilerName already installed";
    else
        install_compiler "$compilerName"
    fi

    # Only try to install dub if it wasn't the main compiler
    if ! [[ $COMPILER =~ ^dub ]] ; then
        if [ -n "${DUB:-}" ] ; then
            # A dub version was explicitly specified with ,dub-1.8.0
            DUB_BIN_PATH="${ROOT}/${DUB}"
            # Check whether the latest dub version needs to be resolved
            if ! [[ $DUB =~ ^dub- ]] ; then
                resolve_latest "$DUB"
                install_dub "dub-$DUB_VERSION"
            else
                install_dub "$DUB"
            fi
        else
            # compiler was installed without requesting dub
            local -r binpath=$(binpath_for_compiler "$compilerName")
            if [ -f "$ROOT/$2/$binpath/dub" ]; then
                if [[ $("$ROOT/$2/$binpath/dub" --version) =~ ([0-9]+\.[0-9]+\.[0-9]+(-[^, ]+)?) ]]; then
                    log "Using dub ${BASH_REMATCH[1]} shipped with $compilerName"
                else
                    log "Using dub shipped with $compilerName"
                fi
            else
                # no dub bundled - manually installing
                DUB_BIN_PATH="${ROOT}/dub"
                resolve_latest dub
                install_dub "dub-$DUB_VERSION"
            fi
        fi
    fi

    write_env_vars "$compilerName"
}

install_dlang_installer() {
    local tmp mirrors
    tmp=$(mkdtemp)
    local mirrors=(
        "https://dlang.org/install.sh"
        "https://s3-us-west-2.amazonaws.com/downloads.dlang.org/other/install.sh"
    )
    local keyring_mirrors=(
        "https://dlang.org/d-keyring.gpg"
        "https://s3-us-west-2.amazonaws.com/downloads.dlang.org/other/d-keyring.gpg"
    )

    mkdir -p "$ROOT"
    log "Downloading https://dlang.org/d-keyring.gpg"
    if [ ! -f "$ROOT/d-keyring.gpg" ]; then
        download_without_verify "$tmp/d-keyring.gpg" "${keyring_mirrors[@]}"
    else
        download_with_verify "$tmp/d-keyring.gpg" "${keyring_mirrors[@]}"
    fi
    mv "$tmp/d-keyring.gpg" "$ROOT/d-keyring.gpg"
    log "Downloading ${mirrors[0]}"
    download_with_verify "$tmp/install.sh" "${mirrors[@]}"
    mv "$tmp/install.sh" "$ROOT/install.sh"
    rmdir "$tmp"
    chmod +x "$ROOT/install.sh"
    log "The latest version of this script was installed as $ROOT/install.sh.
It can be used it to install further D compilers.
Run \`$ROOT/install.sh --help\` for usage information.
"
}

resolve_latest() {
    local input_compiler=${1:-$COMPILER}
    case $input_compiler in
        dmd)
            local mirrors=(
                "http://downloads.dlang.org/releases/LATEST"
                "http://ftp.digitalmars.com/LATEST"
            )
            logV "Determing latest dmd version (${mirrors[0]})."
            COMPILER="dmd-$(fetch "${mirrors[@]}")"
            ;;
        dmd-beta)
            local mirrors=(
                "http://downloads.dlang.org/pre-releases/LATEST"
                "http://ftp.digitalmars.com/LATEST_BETA"
            )
            logV "Determing latest dmd-beta version (${mirrors[0]})."
            COMPILER="dmd-$(fetch "${mirrors[@]}")"
            ;;
        dmd-*) # nightly master or feature branch
            # dmd-nightly, dmd-master, dmd-branch
            # but not: dmd-2016-10-19 or dmd-branch-2016-10-20
            #          dmd-2.068.0 or dmd-2.068.2-5
            #          dmd-2.064 or dmd-2.064-0
            if [[ ! $input_compiler =~ -[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] &&
               [[ ! $input_compiler =~ -[0-9][.][0-9]{3}[.][0-9]{1,3}(-[0-9]{1,3})? ]] &&
               [[ ! $input_compiler =~ -[0-9][.][0-9]{3}(.[0-9]{1,3})? ]]; then
                local url=http://downloads.dlang.org/nightlies/$input_compiler/LATEST
                logV "Determing latest $input_compiler version ($url)."
                COMPILER="dmd-$(fetch "$url")"
            # rewrite dmd-2016-10-19 -> dmd-master-2016-10-19 (default branch for nightlies)
            elif [[ $COMPILER =~ ^dmd-([0-9]{4}-[0-9]{2}-[0-9]{2})$ ]]; then
                COMPILER="dmd-master-${BASH_REMATCH[1]}"
            fi
            ;;
        ldc)
            local url=https://ldc-developers.github.io/LATEST
            logV "Determing latest ldc version ($url)."
            COMPILER="ldc-$(fetch $url)"
            ;;
        ldc-beta)
            local url=https://ldc-developers.github.io/LATEST_BETA
            logV "Determining latest ldc-beta version ($url)."
            COMPILER="ldc-$(fetch $url)"
            ;;
        ldc-latest-ci)
            local url=https://thecybershadow.net/d/github-ldc
            logV "Finding latest ldc CI binary package (at $url)."
            local package
            package="$(fetch $url)"
            if [[ $package =~ ldc2-([0-9a-f]*)-$OS-$ARCH. ]]; then
                COMPILER="ldc-${BASH_REMATCH[1]}"
            else
                fatal "Could not find ldc CI binaries (OS: $OS, arch: $ARCH)"
            fi
            ;;
        ldc-*)
            ;;
        gdc)
            local url=https://gdcproject.org/downloads/LATEST
            logV "Determing latest gdc version ($url)."
            COMPILER="gdc-$(fetch $url)"
            ;;
        gdc-*)
            ;;
        dub)
            local mirrors=(
                "https://dlang.github.io/dub/LATEST"
                "https://code.dlang.org/download/LATEST"
            )
            logV "Determining latest dub version (${mirrors[0]})."
            DUB_VERSION="$(fetch "${mirrors[@]}" | sed 's/^v//')"
            local DUB="dub-${DUB_VERSION}"
            if [ "$COMPILER" == "dub" ] ; then
                COMPILER="$DUB"
            fi
            ;;
        dub-*)
            ;;
        *)
            fatal "Invalid compiler: $COMPILER"
    esac
}

install_compiler() {
    local compiler="$1"
    # dmd-2.065, dmd-2.068.0, dmd-2.068.1-b1
    if [[ $1 =~ ^dmd-2\.([0-9]{3})(\.[0-9])?(-.*)?$ ]]; then
        local basename="dmd.2.${BASH_REMATCH[1]}${BASH_REMATCH[2]}${BASH_REMATCH[3]}"
        local ver="2.${BASH_REMATCH[1]}"

        if [[ $ver > "2.064z" ]]; then
            basename="$basename.$OS"
            ver="$ver${BASH_REMATCH[2]}"
            if [ $OS = freebsd ]; then
                basename="$basename-$MODEL"
            fi
        fi

        if [[ $OS == windows && $ver > "2.068.0" ]] && command -v 7z &>/dev/null; then
            local ext=".7z"
        elif [[ $ver > "2.068.0z" && $OS != windows ]]; then
            local ext=".tar.xz"
        else
            local ext=".zip"
        fi

        if [[ $ARCH != x86* ]]; then
            fatal "no DMD binaries available for $ARCH"
        fi

        local mirrors
        if [ -n "${BASH_REMATCH[3]}" ]; then # pre-release
            mirrors=(
                "http://downloads.dlang.org/pre-releases/2.x/$ver/$basename$ext"
                "http://ftp.digitalmars.com/$basename$ext"
            )
        else
            mirrors=(
                "http://downloads.dlang.org/releases/2.x/$ver/$basename$ext"
                "http://ftp.digitalmars.com/$basename$ext"
            )
        fi

        download_and_unpack_with_verify "$ROOT/$compiler" "${mirrors[@]}"

    # dmd-2015-11-20, dmd-feature_branch-2016-10-20
    elif [[ $1 =~ ^dmd(-(.*))?-[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]; then
        local branch=${BASH_REMATCH[2]:-master}
        local basename="dmd.$branch.$OS"
        if [ $OS = freebsd ]; then
            basename="$basename-$MODEL"
        fi
        if [[ $ARCH != x86* ]]; then
            fatal "no DMD binaries available for $ARCH"
        fi
        local ext
        test $OS = windows && ext=.7z || ext=.tar.xz
        local url="http://downloads.dlang.org/nightlies/$1/$basename$ext"

        download_and_unpack_with_verify "$ROOT/$compiler" "$url"

    # ldc-0.12.1 or ldc-0.15.0-alpha1
    elif [[ $1 =~ ^ldc-(([0-9]+)\.([0-9]+)\.[0-9]+(-.*)?)$ ]]; then
        local ver=${BASH_REMATCH[1]}
        local vernum=$((BASH_REMATCH[2] * 1000 + BASH_REMATCH[3]))
        local ext
        test $OS = windows && ext=.7z || ext=.tar.xz
        local url="https://github.com/ldc-developers/ldc/releases/download/v$ver/ldc2-$ver-$OS-$ARCH$ext"
        if [[ $OS == windows && $vernum -lt 1007 ]]; then
            url="https://github.com/ldc-developers/ldc/releases/download/v$ver/ldc2-$ver-win$MODEL-msvc.zip"
        fi

        download_and_unpack_without_verify "$ROOT/$compiler" "$url"

    # ldc-latest-ci: ldc-8c0abd52
    elif [[ $1 =~ ^ldc-([0-9a-f]+) ]]; then
        local package_hash=${BASH_REMATCH[1]}
        local ext
        test $OS = windows && ext=.7z || ext=.tar.xz
        local url="https://github.com/ldc-developers/ldc/releases/download/CI/ldc2-$package_hash-$OS-$ARCH$ext"

        # Install into 'ldc-8c0abd52-20171222' directory.
        download_and_unpack_without_verify "$ROOT/$compiler" "$url"

    # gdc-4.8.2, gdc-4.9.0-alpha1, gdc-5.2, or gdc-5.2-alpha1
    elif [[ $1 =~ ^gdc-([0-9]+\.[0-9]+(\.[0-9]+)?(-.*)?)$ ]]; then
        local name=${BASH_REMATCH[0]}
        if [ $OS != linux ]; then
            fatal "no gdc binaries available for $OS"
        fi
        case $ARCH in
            x86_64) local triplet=x86_64-linux-gnu;;
            x86) local triplet=i686-linux-gnu;;
        esac
        local url="https://gdcproject.org/downloads/binaries/$triplet/$name.tar.xz"

        download_and_unpack_without_verify "$ROOT/$compiler" "$url"

        url=https://raw.githubusercontent.com/D-Programming-GDC/GDMD/a67179d54611ae8cfb1d791cf7ab8e36c3224b76/dmd-script
        log "Downloading gdmd $url"
        download_without_verify "$ROOT/$1/bin/gdmd" "$url"
        chmod +x "$ROOT/$1/bin/gdmd"

    elif [[ $1 =~ ^dub-v?([0-9]+\.[0-9]+(\.[0-9]+)?(-.*)?)$ ]]; then
        install_dub "$1"
    else
        fatal "Unknown compiler '$compiler'"
    fi
}

find_gpg() {
    if command -v gpg2 &>/dev/null; then
        echo gpg2
    elif command -v gpg &>/dev/null; then
        echo gpg
    else
        echo "Warning: No gpg tool found to verify downloads." >&2
    fi
}

unpack_zip() {
    local zip=$1
    local dir=$2

    (
        # Avoid invoking Windows programs with absolute paths, as
        # POSIX environments on Windows are unreliable in converting
        # POSIX paths to Windows paths on programs' command lines.
        # Use relative paths instead.
        cd "$ROOT"
        zip=${zip/$ROOT\//}
        dir=${dir/$ROOT\//}

        if command -v 7z &>/dev/null; then
            7z x -o"$dir" "$zip" 1>&2
        else
            # Allow unzip to exit with code 1, which it uses to signal warnings
            # such as ".zip appears to use backslashes as path separators".
            unzip -q -d "$dir" "$zip" || [ $? -le 1 ]
        fi
    )
}

# path, verify (0/1), urls...
# the gpg signature is assumed to be url+.sig
download_and_unpack() {
    local path do_verify urls
    path="$1"
    do_verify="$2"
    urls=("${@:3}")
    local tmp name
    tmp=$(mkdtemp)
    name="$(basename "${urls[0]}")"

    check_tools curl
    if [[ $name =~ \.tar\.xz$ ]]; then
        check_tools tar xz
    elif [[ $name =~ \.7z$ ]]; then
        check_tools 7z
    elif [[ $name =~ \.zip$ ]]; then
        if ! command -v 7z &>/dev/null; then
            check_tools unzip
        fi
    fi

    log "Downloading and unpacking ${urls[0]}"
    download "$tmp/$name" "$do_verify" "${urls[@]}"
    if [[ $name =~ \.tar\.xz$ ]]; then
        tar --strip-components=1 -C "$tmp" -Jxf "$tmp/$name"
    elif [[ $name =~ \.tar\.gz$ ]]; then
        tar --strip-components=1 -C "$tmp" -xf "$tmp/$name"
    else
        mkdir "$tmp"/target
        unpack_zip "$tmp/$name" "$tmp"/target

        local files=("$tmp"/target/*)
        if [[ "${#files[@]}" -eq 1 && -d "${files[0]}" ]]; then
            # Single directory at the top of the archive,
            # as is common with .tar archives. Move it out.
            find "$tmp"/target -mindepth 2 -maxdepth 2 -exec sh -c "exec mv "\$@" '$tmp'" sh {} \+
            rmdir "$tmp"/target/*
        else
            find "$tmp"/target -mindepth 1 -maxdepth 1 -exec sh -c "exec mv "\$@" '$tmp'" sh {} \+
        fi
        rmdir "$tmp"/target
    fi
    rm "$tmp/$name"
    mv "$tmp" "$path"
}

# path, urls...
download_and_unpack_with_verify() {
    download_and_unpack "$1" 1 "${@:2}"
}

# path, urls...
download_and_unpack_without_verify() {
    download_and_unpack "$1" 0 "${@:2}"
}

# path, urls...
verify() {
    local path urls
    path="$1"
    urls=("${@:2}")
    : "${GPG:=$(find_gpg)}"
    if [ -z "$GPG" ]; then
        return
    fi
    if ! $GPG --list-keys >/dev/null; then
        fatal "Broken GPG installation"
    fi
    local out
    if ! out=$(fetch "${urls[@]}" | $GPG -q --verify --keyring "$ROOT/d-keyring.gpg" --no-default-keyring - "$path" 2>&1); then
        rm "$path" # delete invalid files
        logE "$out"
        fatal "Invalid signature ${urls[0]}"
    fi
}

binpath_for_compiler() {
    case $1 in
        dmd*)
            local suffix=
            [ $OS = osx ] || [ $OS = windows ] || suffix=$MODEL
            local -r binpath=$OS/bin$suffix
            ;;

        ldc*)
            local -r binpath=bin
            ;;

        gdc*)
            local -r binpath=bin
            ;;
        dub*)
            local -r binpath=
            ;;
    esac
    echo "$binpath"
}

# Path to the compiler executable with a DMD-compatible interface
binexec_for_dmd_compiler() {
    local binPath
    binPath=$(binpath_for_compiler "$1")/
    case $1 in
        dmd*)
            binPath+=dmd
            ;;
        ldc*)
            binPath+=ldmd2
            ;;
        gdc*)
            binPath+=gdmd
            ;;
    esac
    echo "$binPath"
}

# Path to the compiler executable with its custom interface
binexec_for_dc_compiler() {
    local binPath
    binPath=$(binpath_for_compiler "$1")/
    case $1 in
        dmd*)
            binPath+=dmd
            ;;
        ldc*)
            binPath+=ldc2
            ;;
        gdc*)
            binPath+=gdc
            ;;
    esac
    echo "$binPath"
}

binexec_for_dub_compiler() {
    local dub binPath
    dub="${DUB:-}"
    if [[ "$dub" =~ ^dub- ]] ; then
        binPath="$ROOT/$DUB/dub"
    elif [ "$dub" == "dub" ] ; then
        binPath="$ROOT/$DUB/dub"
    else
        binPath="$ROOT/$1/$(binpath_for_compiler "$1")"
        case $1 in
            dub*)
                binPath+=dub
                ;;
            dmd*|ldc*|gdc*)
                binPath+=/dub
                ;;
        esac
        # check for old compiler which ship without dub
        if [ ! -f "$binPath" ]; then
            binPath="$ROOT/dub/dub"
        fi
    fi
    echo "$binPath"
}

write_env_vars() {
    local -r binpath=$(binpath_for_compiler "$1")
    case $1 in
        dmd*)
            local suffix=
            [ $OS = osx ] || suffix=$MODEL
            local libpath=$OS/lib$suffix
            local dc=dmd
            local dmd=dmd
            ;;

        ldc*)
            local libpath=lib
            local dc=ldc2
            local dmd=ldmd2
            ;;

        gdc*)
            if [ -d "$ROOT/$1/lib$MODEL" ]; then
                local libpath=lib$MODEL
            else
                # older gdc releases only ship 64-bit libs
                local libpath=lib
            fi
            local dc=gdc
            local dmd=gdmd
            ;;
        dub*)
            local libpath=
            local dc=
            local dmd=
            ;;
    esac

    logV "Writing environment variables to $ROOT/$1/activate"
{
    # when activate is called twice, deactivate the prior context first
    echo "if [ ! -z \${_OLD_D_PATH+x} ] ; then deactivate; fi"
    echo
    echo "deactivate() {"
    echo "    export PATH=\"\$_OLD_D_PATH\""
    if [ -n "$libpath" ] ; then
    echo "    export LIBRARY_PATH=\"\$_OLD_D_LIBRARY_PATH\""
    echo "    export LD_LIBRARY_PATH=\"\$_OLD_D_LD_LIBRARY_PATH\""
    echo "    unset _OLD_D_LIBRARY_PATH"
    echo "    unset _OLD_D_LD_LIBRARY_PATH"
    fi
    if [ -n "$dmd" ] ; then
        echo "    unset DMD"
        echo "    unset DC"
    fi

    echo "    export PS1=\"\$_OLD_D_PS1\""
    echo "    unset _OLD_D_PATH"
    echo "    unset _OLD_D_PS1"
    echo "    unset -f deactivate"
    echo "}"
    echo
    echo "_OLD_D_PATH=\"\${PATH:-}\""

    if [ -n "$libpath" ] ; then
        echo "_OLD_D_LIBRARY_PATH=\"\${LIBRARY_PATH:-}\""
        echo "_OLD_D_LD_LIBRARY_PATH=\"\${LD_LIBRARY_PATH:-}\""
        echo "export LIBRARY_PATH=\"$ROOT/$1/$libpath\${LIBRARY_PATH:+:}\${LIBRARY_PATH:-}\""
        echo "export LD_LIBRARY_PATH=\"$ROOT/$1/$libpath\${LD_LIBRARY_PATH:+:}\${LD_LIBRARY_PATH:-}\""
    fi

    echo "_OLD_D_PATH=\"\${PATH:-}\""
    echo "_OLD_D_PS1=\"\${PS1:-}\""
    echo "export PS1=\"($1)\${PS1:-}\""
    echo "export PATH=\"${DUB_BIN_PATH}${DUB_BIN_PATH:+:}$ROOT/$1/$binpath\${PATH:+:}\${PATH:-}\""

    if [ -n "$dmd" ] ; then
        echo "export DMD=$dmd"
        echo "export DC=$dc"
    fi
} > "$ROOT/$1/activate"

    logV "Writing environment variables to $ROOT/$1/activate.fish"
{
    echo "function deactivate"
    echo "    set -gx PATH \$_OLD_D_PATH"

    if [ -n "$libpath" ] ; then
        echo "    set -gx LIBRARY_PATH \$_OLD_D_LIBRARY_PATH"
        echo "    set -gx LD_LIBRARY_PATH \$_OLD_D_LD_LIBRARY_PATH"
        echo "    set -e _OLD_D_LIBRARY_PATH"
        echo "    set -e _OLD_D_LD_LIBRARY_PATH"
    fi

    echo "    functions -e fish_prompt"
    echo "    functions -c _old_d_fish_prompt fish_prompt"
    echo "    functions -e _old_d_fish_prompt"
    echo
    echo "    set -e _OLD_D_PATH"

    if [ -n "$dmd" ] ; then
        echo "    set -e DMD"
        echo "    set -e DC"
    fi
    echo "    functions -e deactivate"
    echo "end"
    echo

    echo "set -g _OLD_D_PATH \$PATH"
    echo "set -g _OLD_D_PS1 \$PS1"
    echo
    echo "set -gx PATH ${DUB_BIN_PATH:+\'}${DUB_BIN_PATH}${DUB_BIN_PATH:+\' }'$ROOT/$1/$binpath' \$PATH"

    if [ -n "$libpath" ] ; then
        echo "set -g _OLD_D_LIBRARY_PATH \$LIBRARY_PATH"
        echo "set -g _OLD_D_LD_LIBRARY_PATH \$LD_LIBRARY_PATH"
        echo "set -gx LIBRARY_PATH '$ROOT/$1/$libpath' \$LIBRARY_PATH"
        echo "set -gx LD_LIBRARY_PATH '$ROOT/$1/$libpath' \$LD_LIBRARY_PATH"
    fi

    if [ -n "$dmd" ] ; then
        echo "set -gx DMD $dmd"
        echo "set -gx DC $dc"
    fi

    echo "functions -c fish_prompt _old_d_fish_prompt"
    echo "function fish_prompt"
    echo "    printf '($1)%s' (_old_d_fish_prompt)"
    echo "end"
} > "$ROOT/$1/activate.fish"

    if [[ $OS == windows ]]; then
        logV "Writing environment variables to $ROOT/$1/activate.bat"
{
    local -r winpath=$(cygpath -w "$ROOT/$1/$binpath")
    echo "@echo off"
    echo "if not \"%PATH:${winpath}=%\"==\"%PATH%\" ("
    echo "    echo $1 is already active."
    echo "    goto :EOF"
    echo ")"
    echo "echo Adding $1 to your PATH."
    echo "echo Run \`set PATH=%%_OLD_D_PATH%%\` later on to restore your PATH."
    echo "set _OLD_D_PATH=%PATH%"
    echo "set PATH=${DUB_BIN_PATH:+$(cygpath -w "${DUB_BIN_PATH}");}${winpath};%PATH%"
    if [[ $PROCESSOR_ARCHITECTURE != x86 ]]; then
        echo "set PATH=${winpath}64;%PATH%"
    fi
} > "$ROOT/$1/activate.bat"
    fi
}

uninstall_compiler() {
    if [ ! -d "$ROOT/$1" ]; then
        fatal "$1 is not installed in $ROOT"
    fi
    log "Removing $(display_path "$ROOT/$1")"
    rm -rf "${ROOT:?}/$1"
}

list_compilers() {
    check_tools find
    if [ -d "$ROOT" ]; then
        find "$ROOT" \
             -mindepth 1 \
             -maxdepth 1 \
             -not -name 'dub*' \
             -not -name install.sh \
             -not -name d-keyring.gpg \
             -not -name '.*' \
             -exec basename {} \; | \
            grep . # fail if none found
    fi
}

install_dub() {
    if [ $OS != linux ] && [ $OS != windows ] && [ $OS != osx ]; then
        log "no dub binaries available for $OS"
        return
    fi
    local dub="$1"
    if [ -d "$ROOT/$dub" ]; then
        log "$dub already installed"
        return
    fi
    local tmp url
    tmp=$(mkdtemp)
    dubVersion=${dub##dub-}
    local ext=.tar.gz arch=$ARCH
    if [[ $OS == windows ]]; then
        ext=.zip
        if [[ "$dubVersion" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+) ]]; then
            local vernum=$((BASH_REMATCH[1] * 1000000 + BASH_REMATCH[2] * 1000 + BASH_REMATCH[3]))
            if [[ $vernum -lt 1014000 ]]; then
                # Older releases, pre-1.14.0, were on code.dlang.org,
                # and did not have x86_64 versions.
                arch=x86
            elif [[ $vernum -lt 1020000 ]]; then
                # Releases since 1.14.0 have x86_64 archives on GitHub.
                # However, these are corrupted (https://github.com/dlang/dub/issues/1795).
                # At this point it's unclear if the archives are going to be fixed,
                # so just install them for now.
                :
            fi
        fi
    fi
    local mirrors=(
        "https://github.com/dlang/dub/releases/download/v${dubVersion}/dub-v${dubVersion}-$OS-$arch$ext"
        "https://code.dlang.org/files/$dub-$OS-$arch$ext"
    )
    log "Downloading and unpacking ${mirrors[0]}"
    download_without_verify "$tmp/dub$ext" "${mirrors[@]}"
    if [[ $ext == .tar.gz ]]; then
        tar -C "$tmp" -zxf "$tmp/dub$ext"
    else
        unpack_zip "$tmp/dub$ext" "$tmp"
    fi
    logV "Removing old dub symlink"
    rm -rf "$ROOT/dub"
    mv "$tmp" "$ROOT/$dub"
    logV "Linking $ROOT/dub -> $ROOT/$dub"
    ln -s "$dub" "$ROOT/dub"
}

# ------------------------------------------------------------------------------

parse_args "$@"

case $(uname -m) in
    x86_64|amd64) ARCH=x86_64; MODEL=64;;
    aarch64) ARCH=aarch64; MODEL=64;;
    i*86) ARCH=x86; MODEL=32;;
    *)
        fatal "Unsupported Arch $(uname -m)"
        ;;
esac
if [[ $OS-$ARCH = windows-x86_64 && $COMPILER = ldc* ]]; then
    ARCH=x64
fi

resolve_latest "$COMPILER"
run_command ${COMMAND:-install} "$COMPILER"
}

_ "$@"
