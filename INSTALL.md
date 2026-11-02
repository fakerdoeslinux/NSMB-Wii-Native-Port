# Installing the New Super Mario Bros. Wii native port

This guide builds the port on your machine from your own game dump. The build recompiles your
dump's PowerPC code into a native executable, so it has to run locally — no prebuilt binary can be
shared.

- [Before you start](#before-you-start)
- [Windows 10/11](#windows-1011)
- [Linux](#linux)
- [First run and configuration](#first-run-and-configuration)
- [Other game revisions](#other-game-revisions)
- [Troubleshooting](#troubleshooting)

---

## Before you start

**You need your own copy of the game.** The port takes *New Super Mario Bros. Wii* **USA**
(`SMNE01`), revision 1 or 2, as it appears on a disc you own. Nothing is downloaded for you and
nothing can be shared back.

### Extract the disc with Dolphin

1. Open Dolphin and add your disc image (or insert the disc).
2. Right-click the game → **Properties** → **Filesystem**.
3. Right-click the disc in the tree → **Extract Entire Disc…**.
4. Pick an empty folder, e.g. `D:\nsmbw` on Windows or `~/nsmbw` on Linux.

The result looks like this — that folder is what the scripts want:

```
nsmbw/
├── files/     ← game content
├── sys/
│   ├── main.dol
│   ├── fst.bin
│   └── boot.bin
└── ...
```

A `DATA/` folder holding `files/` and `sys/` is accepted too.

### Requirements

| | Linux | Windows |
|---|---|---|
| OS | x86-64 Linux, Vulkan driver | Windows 10/11 x64 |
| Disk | ~4 GB | ~4 GB |
| RAM | 4 GB+ (8 GB+ builds faster) | 4 GB+ |
| Tools | .NET 8 SDK, Python 3, git | .NET 8 SDK, Python 3, git, PowerShell |

---

## Windows 10/11

Tested against the scripts in `projects/nsmbw/tools/`; the Linux build is the one that has been
run end-to-end by this fork's maintainer, so expect the Windows path to follow upstream.

### 1. Get the repository

```powershell
git clone https://github.com/fakerdoeslinux/NSMB-Wii-Native-Port.git
cd NSMB-Wii-Native-Port
```

### 2. Install Python and .NET

```powershell
winget install Python.Python.3.12
winget install Microsoft.DotNet.SDK.8
```

Close and reopen PowerShell so `python` and `dotnet` are on `PATH`.

If PowerShell refuses to run the scripts, start it with
`powershell -ExecutionPolicy Bypass` for this session.

### 3. Download the pinned build toolchain

```powershell
.\Launcher\Prepare-PortableTools.ps1
```

This fetches llvm-mingw, CMake and Ninja into `Launcher\artifacts\portable-tools` (~2 GB of
downloads; they are reused by later builds).

### 4. Download the pinned third-party sources

```powershell
.\Launcher\Prepare-Dependencies.ps1 -Destination Dependencies
```

This is SDL3, Dawn and the other libraries the runtime compiles against.

### 5. Prepare the game's code

```powershell
python projects\nsmbw\tools\unpack_game.py D:\nsmbw
python projects\nsmbw\tools\prelink_rels.py
```

The first command copies `main.dol` and unpacks the four boot RELs into a `game` folder next to
the repository. The second links those RELs offline the way the game links them at boot. If
`unpack_game.py` warns that your `main.dol` hash differs from the known one, that is fine as long
as it says the revision was recognised — some dump variants differ.

### 6. Translate and build

```powershell
.\projects\nsmbw\tools\rebuild-all.ps1
```

This runs the four translator steps over ~40,000 functions and then compiles the native binary.
It takes a while the first time (tens of minutes depending on the machine); later runs only redo
what changed.

### 7. Run

```powershell
.\native-build\WiiCompiled.exe
```

Name your extracted game folder `game` and place it beside the exe, or let the game ask for the
folder on first start. Settings, saves and logs then live in
`%USERPROFILE%\Documents\MarioWiiSaveData`.

---

## Linux

Tested on Fedora with an AMD GPU. The same steps work on other distributions with their package
names.

### 1. Install the build dependencies

Fedora:

```bash
sudo dnf install -y clang lld cmake ninja-build llvm alsa-lib-devel vulkan-loader-devel \
                    libX11-devel libxkbcommon-devel wayland-devel python3 git
```

Debian/Ubuntu:

```bash
sudo apt install -y clang lld cmake ninja-build llvm libasound2-dev libvulkan-dev \
                    libx11-dev libxkbcommon-dev libwayland-dev python3 git
```

You also need the **.NET 8 SDK**. Either install it from your distribution
(`sudo dnf install dotnet-sdk-8.0`), or without root:

```bash
curl -sSL https://dot.net/v1/dotnet-install.sh | bash -s -- --channel 8.0 --install-dir ~/.dotnet
export PATH="$HOME/.dotnet:$PATH"
```

### 2. Get the repository

```bash
git clone https://github.com/fakerdoeslinux/NSMB-Wii-Native-Port.git
cd NSMB-Wii-Native-Port
```

### 3. Get the pinned toolchain

```bash
./Launcher/prepare-portable-tools.sh --arch x86_64
```

This downloads the pinned clang, CMake and Ninja into
`Launcher/artifacts/portable-tools/toolchain-x86_64` (~2 GB of downloads, ~550 MB kept). It keeps
the build independent of your system compiler. To use your own clang instead, point the build at
it with `TOOLCHAIN_BIN=/usr/bin` (it must contain `clang`, `clang++`, `cmake`, `ninja`, `llvm-ar`
and `llvm-ranlib`).

### 4. Prepare the game's code

```bash
python3 projects/nsmbw/tools/unpack_game.py ~/nsmbw
python3 projects/nsmbw/tools/prelink_rels.py
```

This writes a `game` folder next to the repository. A hash warning is fine as long as the revision
is recognised; see [Other game revisions](#other-game-revisions).

### 5. Translate and build

```bash
./projects/nsmbw/tools/build-linux.sh --game ~/nsmbw
```

The script translates the game (~40,000 functions), generates the native sources, configures
CMake with the pinned toolchain and compiles. Translated shards need about 2 GB of RAM each, so
the script lowers the parallelism on small machines; 8 GB+ makes it noticeably faster.

When it finishes:

```
Done: /home/you/nsmbw-local-build/WiiCompiled
```

### 6. Run

```bash
cd ../nsmbw-local-build
./WiiCompiled
```

The folder contains the executable, its data files and a `game` link to your extracted disc.
Settings and saves go to `~/.local/share/WiiCompiled`.

Rebuild after changing runtime sources by re-running the same command; the translator and Ninja
only redo what changed.

---

## First run and configuration

On first start the game asks for the folder holding your extracted disc (`sys/` and `files/`).
After that it remembers it.

The settings file is a small TOML:

- Linux: `~/.local/share/WiiCompiled/Config.toml`
- Windows: `%USERPROFILE%\Documents\MarioWiiSaveData\Config.toml`

```toml
[video]
widescreen = true           # 16:9 output; false gives the original 4:3 framing
resolution_multiplier = 1.0 # 1.0 = 480 render lines, 0 = match the window, 2.0 = 2x supersampling
show_fps = true             # frame counter in the corner
window_width = 854
window_height = 480
graphics_api = "auto"       # Linux: vulkan. Windows: auto / vulkan / d3d12 / d3d11
display_mode = "windowed"   # windowed / borderless / exclusive

[paths]
dvd_root = "/home/you/nsmbw"
```

`dvd_root` is the most common thing to fix: point it at the folder that contains `sys/` and
`files/`, not at `sys/` itself.

You can also change everything in-game from the settings overlay.

### FPS overlay with MangoHud

```bash
mangohud ./WiiCompiled
```

MangoHud shows FPS, frame times, the Vulkan driver, GPU name and utilisation. It reads
`~/.config/MangoHud/MangoHud.conf`; set `fps_only=0` there to see the full panel. On Windows use
your GPU driver's overlay instead (AMD: <kbd>Alt</kbd>+<kbd>R</kbd>, NVIDIA: <kbd>Alt</kbd>+<kbd>Z</kbd>).

---

## Other game revisions

This repository is configured for **Rev 1**, which is what has been tested end to end. The runtime
and the tools also handle **Rev 2**; switching means two edits in `projects/nsmbw/recomp.yml`:

```yaml
inputs:
  dol:
    sha256: <the hash unpack_game.py prints for your main.dol>
  rel:
    load_address: 0x80768500   # Rev 1 uses 0x807684C0
```

`unpack_game.py` prints the hash it computed whenever it differs from the known one, so run it
first and copy the value. `prelink_rels.py` needs no edit — it reads the revision from `main.dol`
and picks the matching REL base by itself.

Then rebuild as usual. A build only accepts the revision it was translated for; if you start it
with the other disc you get *"This build runs only revision N"*.

---

## Troubleshooting

| Symptom | Cause and fix |
|---|---|
| A black window, or it closes immediately | `dvd_root` points at the wrong folder. It must be the folder holding `sys/` and `files/`. |
| *"An error has occurred. Press the EJECT Button…"* | That is the game's own fatal screen, usually the BCA check. This fork answers it in the runtime, so a build from this source should not hit it. If it does, your build predates the fix. |
| *"This build runs only revision 1 or 2"* | The disc revision does not match the build. Rebuild with the matching `recomp.yml`, or use the right dump. |
| `CMake` cannot find X11 headers | Install the `-dev`/`-devel` packages above. `build-linux.sh` switches off the SDL extensions it cannot find. |
| `ld.lld not found` | The pinned toolchain is missing: re-run `prepare-portable-tools.sh --arch x86_64`. |
| Build runs out of memory | Lower the parallelism: `build-linux.sh --jobs 2`. Each translated shard takes ~2 GB. |
| Stutter when entering a course or the world map | The disc read cache is off (machines under 4 GB of RAM). Close other programs, or accept it: `NSMBW_NO_DVD_PRELOAD=1` is what disables it. |
| Low frame rate | Check `show_fps` and run with `mangohud` to see the GPU. A software Vulkan driver (`lavapipe`) is very slow; install your GPU's Vulkan driver. |
| A specific spot is slow | Set `NSMBW_LOG_FPS=1` and look for `slow frame` lines — they name the scene and split the time into game, GX, disc and present. |

Logs, when you need to report a problem:

- Linux: `~/.local/share/WiiCompiled/Logs/<newest>/console.log`
- Windows: `%USERPROFILE%\Documents\MarioWiiSaveData\Logs\<newest>\console.log`
