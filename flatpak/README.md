# Flatpak build for Zkool

## Contents

- **cc.methyl.Zkool.yml** – Flatpak manifest (desktop file, icon, metainfo, sandbox permissions).
- **cc.methyl.Zkool.metainfo.xml** – AppData for app stores and `flatpak run`.
- **linux/cc.methyl.Zkool.desktop** – Desktop entry (menu integration).

## Prerequisites

- Flutter Linux bundle: `flutter build linux`
- Flatpak, flatpak-builder (1.4+), and AppStream for appstream-compose:
  ```bash
  # Ubuntu 24.04+ (flatpak-builder 1.4+ in default repos)
  sudo apt install flatpak flatpak-builder appstream appstream-compose appstream-util
  sudo flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
  flatpak install flathub org.freedesktop.Platform//25.08 org.freedesktop.Sdk//25.08
  ```
  On Ubuntu 22.04 you need flatpak-builder 1.4+ (compose runs on host): add the PPA and install:
  ```
  sudo add-apt-repository ppa:flatpak/stable
  sudo apt update
  sudo apt install --only-upgrade flatpak-builder
  sudo apt install flatpak-builder appstream appstream-compose appstream-util
  sudo flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
  flatpak install flathub org.freedesktop.Platform//25.08 org.freedesktop.Sdk//25.08
  ```
  On Fedora:
  ```
  sudo dnf install flatpak-builder
  ```
  On Arch:
  ```
  sudo pacman -S flatpak-builder
  ```

## Check flatpak-builder version

```bash
flatpak-builder --version
```

You need **1.4.0 or higher**. If you see 1.2.x:

## Build (local)

From the **repository root**:

```bash
flatpak-builder --force-clean flatpak-build flatpak/cc.methyl.Zkool.yml
```

## Export and single-file bundle

To create a repository (for hosting or Flathub):

```bash
flatpak build-export export flatpak-build
```

To create a single-file `.flatpak` for distribution:

```bash
flatpak build-bundle export zkool.flatpak cc.methyl.Zkool
```

Users can install with:

```bash
flatpak install --user zkool.flatpak
flatpak run cc.methyl.Zkool
```

## Run (without installing)

```bash
flatpak-builder --run flatpak-build flatpak/cc.methyl.Zkool.yml /app/bin/zkool-wrapper
```

## Install from local build

```bash
flatpak-builder --install --user --force-clean flatpak-build flatpak/cc.methyl.Zkool.yml
flatpak run cc.methyl.Zkool
```

## CI

The Linux build workflow (`.github/actions/linux`) builds the Flatpak after the Flutter Linux bundle and produces `zkool-<version>-<arch>.flatpak` in the release artifacts (with `.deb` and `.AppImage`).

## Manifest summary

| Item | Value |
|------|--------|
| App ID | `cc.methyl.Zkool` |
| Runtime | org.freedesktop.Platform 25.08 |
| Command | `zkool-wrapper` |
| Permissions | ipc, fallback-x11, wayland, dri, pulseaudio, network, home, xdg-documents |

Desktop file and icon are installed so the app appears in the application menu with the correct name and icon.

## Different runtime version

To use e.g. 24.08: set `runtime-version: '24.08'` in `cc.methyl.Zkool.yml` and install that runtime and SDK.
