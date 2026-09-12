#!/usr/bin/env bash
#
# Build dict and install it somewhere on your PATH, so that `dict <word>`
# works from any directory.
#
#   ./install.sh                     install to ~/.local/bin
#   ./install.sh --prefix /usr/local/bin
#   ./install.sh --uninstall
#
set -euo pipefail

DEFAULT_PREFIX="$HOME/.local/bin"
PREFIX="${PREFIX:-$DEFAULT_PREFIX}"
BINARY=dict
ACTION=install
PURGE=no

# ---------------------------------------------------------------------------
# Output helpers. Colour only when writing to a terminal, and never when
# NO_COLOR is set.
# ---------------------------------------------------------------------------
if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
    BOLD=$'\033[1m'; GREEN=$'\033[32m'; YELLOW=$'\033[33m'
    RED=$'\033[31m'; DIM=$'\033[2m';    RESET=$'\033[0m'
else
    BOLD=''; GREEN=''; YELLOW=''; RED=''; DIM=''; RESET=''
fi

say()  { printf '%s\n' "$*"; }
step() { printf '%s==>%s %s\n' "$BOLD" "$RESET" "$*"; }
ok()   { printf '%s  ok%s %s\n' "$GREEN" "$RESET" "$*"; }
warn() { printf '%s  !%s  %s\n' "$YELLOW" "$RESET" "$*" >&2; }
die()  { printf '%serror:%s %s\n' "$RED" "$RESET" "$*" >&2; exit 1; }

usage() {
    cat <<EOF
Build dict and install it on your PATH.

Usage: ./install.sh [OPTIONS]

Options:
  --prefix DIR   Install into DIR instead of $DEFAULT_PREFIX
  --uninstall    Remove an installed dict instead of installing one
  --purge        With --uninstall, also delete the cached entries
  -h, --help     Show this help

The install directory may also be set with the PREFIX environment variable.
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --prefix)     [ $# -ge 2 ] || die "--prefix needs a directory"; PREFIX="$2"; shift 2 ;;
        --prefix=*)   PREFIX="${1#*=}"; shift ;;
        --uninstall)  ACTION=uninstall; shift ;;
        --purge)      PURGE=yes; shift ;;
        -h|--help)    usage; exit 0 ;;
        *)            die "unknown option: $1 (try --help)" ;;
    esac
done

# Expand a leading ~ so --prefix=~/bin works even when quoted.
case "$PREFIX" in "~"|"~/"*) PREFIX="$HOME${PREFIX#\~}" ;; esac

# ---------------------------------------------------------------------------
# Work from the repository this script lives in, so it can be run from
# anywhere, including through a symlink.
# ---------------------------------------------------------------------------
source_path="${BASH_SOURCE[0]}"
while [ -L "$source_path" ]; do
    source_dir="$(cd -P "$(dirname "$source_path")" && pwd)"
    source_path="$(readlink "$source_path")"
    case "$source_path" in /*) ;; *) source_path="$source_dir/$source_path" ;; esac
done
REPO="$(cd -P "$(dirname "$source_path")" && pwd)"

target="$PREFIX/$BINARY"

# ---------------------------------------------------------------------------
# Uninstall
# ---------------------------------------------------------------------------
if [ "$ACTION" = uninstall ]; then
    if [ -e "$target" ]; then
        rm -f "$target"
        ok "removed $target"
    else
        warn "no $BINARY found at $target"
    fi

    cache="${XDG_CACHE_HOME:-$HOME/.cache}/dict"
    if [ "$PURGE" = yes ]; then
        if [ -d "$cache" ]; then
            rm -rf "$cache"
            ok "removed the cache at $cache"
        fi
    elif [ -d "$cache" ]; then
        say "${DIM}Cached definitions are still in $cache (--purge removes them).${RESET}"
    fi
    exit 0
fi

# ---------------------------------------------------------------------------
# Install
# ---------------------------------------------------------------------------
command -v cargo >/dev/null 2>&1 || die "cargo not found. Install Rust from https://rustup.rs and try again."
[ -f "$REPO/Cargo.toml" ] || die "no Cargo.toml in $REPO; run this script from inside the dict repository."

step "Building $BINARY (release)"
# --locked builds against the committed Cargo.lock, so the install is
# reproducible rather than picking up whatever versions resolve today.
if ! cargo build --release --locked --manifest-path "$REPO/Cargo.toml"; then
    die "the build failed; nothing was installed."
fi

built="$REPO/target/release/$BINARY"
[ -x "$built" ] || die "expected a binary at $built but found none."

step "Installing to $target"
mkdir -p "$PREFIX" || die "could not create $PREFIX"
if [ ! -w "$PREFIX" ]; then
    die "$PREFIX is not writable. Either pick another directory with --prefix, or re-run with sudo."
fi

# install(1) replaces the file atomically, which matters if dict is running.
install -m 755 "$built" "$target" || die "could not write $target"
ok "installed $("$target" --version)"

# ---------------------------------------------------------------------------
# Check that typing `dict` will actually reach what we just installed
# ---------------------------------------------------------------------------
on_path=no
case ":$PATH:" in *":$PREFIX:"*) on_path=yes ;; esac

if [ "$on_path" = no ]; then
    warn "$PREFIX is not on your PATH, so typing '$BINARY' will not find it yet."
    case "${SHELL##*/}" in
        zsh)  rc="~/.zshrc" ;;
        fish) rc="~/.config/fish/config.fish" ;;
        *)    rc="~/.bashrc" ;;
    esac
    say ""
    if [ "${SHELL##*/}" = fish ]; then
        say "  Add this to $rc, then open a new terminal:"
        say "    ${BOLD}fish_add_path $PREFIX${RESET}"
    else
        say "  Add this to $rc, then open a new terminal:"
        say "    ${BOLD}export PATH=\"$PREFIX:\$PATH\"${RESET}"
    fi
    say ""
else
    # Something earlier on PATH could still shadow the new binary.
    found="$(command -v "$BINARY" 2>/dev/null || true)"
    if [ -n "$found" ] && [ "$found" != "$target" ]; then
        warn "typing '$BINARY' runs $found, not the copy just installed."
        warn "that directory comes earlier on your PATH; remove the other copy or reorder PATH."
    else
        ok "'$BINARY' is on your PATH"
    fi
fi

say ""
say "Try it:"
say "  ${BOLD}$BINARY compelling${RESET}      definitions, pronunciation and usage labels"
say "  ${BOLD}$BINARY colour${RESET}          Canadian spelling guidance"
say "  ${BOLD}$BINARY toque${RESET}           a Canadianism, with its Canadian pronunciation"
say "  ${BOLD}$BINARY --help${RESET}"
say ""
say "${DIM}Uninstall with: ./install.sh --uninstall${RESET}"
