# New Super Mario Bros. Wii — native PC port

A native x86-64 port of **New Super Mario Bros. Wii (USA, SMNE01)** for Linux and Windows.

The game's own PowerPC code is statically recompiled to C++ ahead of time and runs against a
small Wii API layer (DVD, VI, GX, AX, input). There is no emulator, no ISO, and no Wii: you point
the port at your own disc dump once and it runs the game natively.

**[→ Install guide (Linux and Windows)](INSTALL.md)**

---

## What works

| | |
|---|---|
| Game | *New Super Mario Bros. Wii* USA, `SMNE01`. Rev 1 is configured and tested; the tools also handle Rev 2 (see [INSTALL.md](INSTALL.md#other-game-revisions)) |
| Platforms | Linux x86-64 (tested on Fedora) · Windows 10/11 x64 |
| Renderer | Vulkan via [Aurora](https://github.com/aurora-wright/aurora) — Vulkan / D3D12 / D3D11 on Windows |
| Audio | Native AX / DSP HLE mixing |
| Frame rate | Locked 60 fps; ~2 ms of graphics encoding and ~7 ms of idle per frame |
| Input | SDL gamepads, Wii Remotes, keyboard |
| Extras | Widescreen, FPS counter, MangoHud, Discord presence, custom textures |

## What you need

- Your **own** copy of the game, dumped and extracted with Dolphin (details in the install guide).
- 4 GB+ RAM (the disc read cache uses ~335 MB of it) and ~4 GB of disk for a build.
- **Linux:** .NET 8 SDK, Python 3, clang/CMake/Ninja, Vulkan and ALSA headers.
- **Windows:** .NET 8 SDK, Python 3, PowerShell.

No game data is in this repository, and none is downloaded by it.

## Quick start

Linux (Fedora packages; see [INSTALL.md](INSTALL.md) for other distributions):

```bash
git clone https://github.com/fakerdoeslinux/NSMB-Wii-Native-Port.git
cd NSMB-Wii-Native-Port
sudo dnf install -y clang lld cmake ninja-build llvm alsa-lib-devel vulkan-loader-devel \
                    libX11-devel libxkbcommon-devel wayland-devel python3
./Launcher/prepare-portable-tools.sh --arch x86_64
python3 projects/nsmbw/tools/unpack_game.py ~/nsmbw/DATA
python3 projects/nsmbw/tools/prelink_rels.py
./projects/nsmbw/tools/build-linux.sh --game ~/nsmbw/DATA
../nsmbw-local-build/WiiCompiled
```

Windows 10/11 (PowerShell):

```powershell
git clone https://github.com/fakerdoeslinux/NSMB-Wii-Native-Port.git
cd NSMB-Wii-Native-Port
.\Launcher\Prepare-PortableTools.ps1
.\Launcher\Prepare-Dependencies.ps1 -Destination Dependencies
python projects\nsmbw\tools\unpack_game.py D:\nsmbw\DATA
python projects\nsmbw\tools\prelink_rels.py
.\projects\nsmbw\tools\rebuild-all.ps1
.\native-build\WiiCompiled.exe
```

The first launch asks for the folder holding your extracted disc (`sys/` and `files/`).

## Why this fork

- **Extracted discs work.** A pressed disc carries a BCA anti-piracy track that a dumped image
  does not have, so the stock build stops at *"An error has occurred. Press the EJECT Button…"*.
  This fork answers that check in the runtime instead of failing.
- **Revision-aware tooling.** `prelink_rels.py` reads `main.dol` and links the boot RELs at the
  right base (`0x807684C0` Rev 1, `0x80768500` Rev 2).
- **No load-time stutter.** Game files are cached in RAM once, in the background, so scene changes
  and the world map no longer hit the disk mid-frame.
- **Cleanup for a source release.** Upstream's CI cannot run without a game dump, so the
  always-failing workflows are gone.

Technical details, the full patch list and profiling numbers are in
[docs/PORT-NOTES.md](docs/PORT-NOTES.md).

## Repository layout

| Path | What it is |
|---|---|
| `runtime/` | The Wii API layer and main loop: DVD/VI/GX/AX HLE, memory, input, settings overlay |
| `aurora-main/` | Vendored graphics backend (Vulkan, D3D12, D3D11, Metal) |
| `translator/` | The PowerPC → C++ static recompiler (.NET 8) |
| `projects/nsmbw/` | Game-specific config: `recomp.yml`, `MAP.txt`, and every build tool |
| `Launcher/` | Portable toolchain download, installer and packaging scripts |
| `android/` | Android app shell; not used by the Linux/Windows instructions |
| `generated/`, `native-build/` | Build output, created by the build and ignored by git |
| `docs/` | Port notes, upstream macOS build notes |

## Documentation

- [INSTALL.md](INSTALL.md) — installing on Linux and Windows, first run, configuration, troubleshooting
- [docs/PORT-NOTES.md](docs/PORT-NOTES.md) — what was patched, performance measurements, profiling flags
- [projects/nsmbw/BUILDING.md](projects/nsmbw/BUILDING.md) — the long-form build reference (Android, Newer SMBW, all scripts)

## Legal

This is a **port toolchain, not a game distribution**. It contains no Nintendo code or assets, and
it will not run without a disc image you dumped yourself from a copy you own. Building the port
recompiles your own dump locally. Do not share the generated `game/` folder or build output.

- Game and characters: © Nintendo
- Runtime and recompiler: WiiCompiled contributors (see [LICENSE](LICENSE) and
  [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md))
- This fork's changes: see [docs/PORT-NOTES.md](docs/PORT-NOTES.md)

## Credits

- [WitherBlitz/NSMB-Wii-Multiplayer-Recompilation](https://github.com/WitherBlitz/NSMB-Wii-Multiplayer-Recompilation) — the NSMBW toolchain this is based on
- [Aurora](https://github.com/aurora-wright/aurora) — Vulkan/D3D12/Metal graphics backend
- [Dolphin Emulator](https://dolphin-emu.org) — disc extraction
