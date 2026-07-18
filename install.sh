#!/bin/sh

set -eu

REPOSITORY="Richuchusave/symphony"
INSTALL_DIR="${SYMPHONY_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${SYMPHONY_VERSION:-latest}"
SKIP_DEPS="${SYMPHONY_SKIP_SYSTEM_DEPS:-0}"

info() {
    printf 'symphony: %s\n' "$*"
}

fail() {
    printf 'symphony: error: %s\n' "$*" >&2
    exit 1
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

run_as_root() {
    if [ "$(id -u)" -eq 0 ]; then
        "$@"
    elif command_exists sudo; then
        sudo "$@"
    else
        fail "installing mpv requires root access; install mpv and run this command again"
    fi
}

install_mpv() {
    [ "$SKIP_DEPS" = "1" ] && return 0
    command_exists mpv && return 0

    info "mpv is required for playback; installing it with the system package manager"
    if command_exists apt-get; then
        run_as_root apt-get update
        run_as_root apt-get install -y mpv
    elif command_exists dnf; then
        run_as_root dnf install -y mpv
    elif command_exists yum; then
        run_as_root yum install -y mpv
    elif command_exists pacman; then
        run_as_root pacman -Sy --needed --noconfirm mpv
    elif command_exists zypper; then
        run_as_root zypper --non-interactive install mpv
    elif command_exists apk; then
        run_as_root apk add mpv
    else
        fail "mpv is missing and no supported package manager was found"
    fi
}

configure_path() {
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) return 0 ;;
    esac

    case "${SHELL:-}" in
        */zsh)
            profile="$HOME/.zshrc"
            path_line='export PATH="$HOME/.local/bin:$PATH"'
            ;;
        */fish)
            profile="$HOME/.config/fish/config.fish"
            path_line='fish_add_path "$HOME/.local/bin"'
            mkdir -p "$HOME/.config/fish"
            ;;
        *)
            profile="$HOME/.bashrc"
            path_line='export PATH="$HOME/.local/bin:$PATH"'
            ;;
    esac

    if [ "$INSTALL_DIR" = "$HOME/.local/bin" ]; then
        if [ ! -f "$profile" ] || ! grep -F "$path_line" "$profile" >/dev/null 2>&1; then
            printf '\n# Added by the Symphony installer\n%s\n' "$path_line" >> "$profile"
            info "added $INSTALL_DIR to PATH in $profile"
        fi
    else
        info "$INSTALL_DIR is not in PATH; add it before running symphony"
    fi
}

os="$(uname -s)"
arch="$(uname -m)"

[ "$os" = "Linux" ] || fail "prebuilt releases currently support Linux only"

case "$arch" in
    x86_64|amd64)
        target="x86_64-unknown-linux-gnu"
        yt_dlp_asset="yt-dlp_linux"
        ;;
    aarch64|arm64)
        target="aarch64-unknown-linux-gnu"
        yt_dlp_asset="yt-dlp_linux_aarch64"
        ;;
    *) fail "unsupported CPU architecture: $arch" ;;
esac

command_exists curl || fail "curl is required"
command_exists tar || fail "tar is required"

archive="symphony-${target}.tar.gz"
if [ "$VERSION" = "latest" ]; then
    release_url="https://github.com/${REPOSITORY}/releases/latest/download"
else
    release_url="https://github.com/${REPOSITORY}/releases/download/${VERSION}"
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

info "downloading Symphony ${VERSION} for ${target}"
curl --proto '=https' --tlsv1.2 -fsSL "$release_url/$archive" -o "$tmp_dir/$archive"
curl --proto '=https' --tlsv1.2 -fsSL "$release_url/$archive.sha256" -o "$tmp_dir/$archive.sha256"

if command_exists sha256sum; then
    (cd "$tmp_dir" && sha256sum -c "$archive.sha256") >/dev/null
elif command_exists shasum; then
    expected="$(cut -d ' ' -f 1 "$tmp_dir/$archive.sha256")"
    actual="$(shasum -a 256 "$tmp_dir/$archive" | cut -d ' ' -f 1)"
    [ "$expected" = "$actual" ] || fail "checksum verification failed"
else
    fail "sha256sum or shasum is required to verify the download"
fi

tar -xzf "$tmp_dir/$archive" -C "$tmp_dir"
mkdir -p "$INSTALL_DIR"
install -m 0755 "$tmp_dir/symphony" "$INSTALL_DIR/symphony"

if ! command_exists yt-dlp && [ ! -x "$INSTALL_DIR/yt-dlp" ]; then
    info "installing yt-dlp for public music search"
    curl --proto '=https' --tlsv1.2 -fsSL \
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/$yt_dlp_asset" \
        -o "$INSTALL_DIR/yt-dlp"
    chmod 0755 "$INSTALL_DIR/yt-dlp"
fi

install_mpv
configure_path

info "installed successfully"
if command_exists symphony; then
    info "run: symphony"
else
    info "open a new terminal, then run: symphony"
fi
