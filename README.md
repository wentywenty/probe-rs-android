# probe-rs-android

English | [简体中文](README.zh-CN.md)

Run `probe-rs` on Android to enumerate USB debug probes, attach to ARM Cortex-M targets, and flash firmware over SWD.

The project was initially validated on a **Seeed Studio XIAO nRF54LM20A** board. It also uses the probe drivers already provided by probe-rs, including CMSIS-DAP, J-Link, ST-Link, and WCH-Link.

---

## Status

Validated on:

- Phone: Xiaomi 23049RAD8C, Android 15, arm64, KernelSU root
- Probe: Seeed Studio XIAO nRF54LM20A CMSIS-DAP (`2886:0068`)
- Target: XIAO nRF54LM20A
- Firmware: `robopoly-xiao/gateway`, Zephyr `xiao_nrf54lm20a/nrf54lm20a/cpuapp`
- Result: probe enumeration, target attach, and ELF flashing succeeded

WCH-Link (`1a86:8012`) is also detected as CMSIS-DAP and is accessible.

---

## Why this repository exists

`probe-rs` already supports CH347, CMSIS-DAP, and other probes, but its USB backend `nusb` disables Android usbfs enumeration and device opening by default.

This repository provides:

1. An Android executable CLI named `probe-rs-android`
2. An Android patch for `nusb` based on upstream main
3. A reproducible NDK build script

It does not modify the GKI kernel and does not require a CH347 kernel driver.

---

## Layout

```text
.
├── Cargo.toml
├── Cargo.lock
├── src/main.rs
├── scripts/build.sh
├── patches/0001-nusb-android-usbfs.patch
└── third_party/
    ├── probe-rs/           # git submodule, pinned probe-rs
    └── nusb/               # git submodule, wentywenty/nusb android-usbfs
```

---

## Submodules

```text
third_party/probe-rs
  https://github.com/probe-rs/probe-rs.git
  d77197c2b3a4063a69f43b6c580234c4a7f81d43

third_party/nusb
  https://github.com/wentywenty/nusb.git
  9428563d3f9d42240a095b109974de93c22042aa
```

`third_party/nusb` is based on upstream `nusb` main (`c46ccd0`) plus the Android usbfs patch. The fork is required because upstream main still contains:

```rust
#[cfg(target_os = "android")]
pub(crate) fn from_device_info(...) {
    Blocking::new(move || unimplemented!())
}
```

and `list_devices()` still excludes Android.

If upstream merges Android support, the submodule can be switched back to upstream without changing this CLI.

---

## Requirements

- Linux host
- Rust and Cargo
- `aarch64-linux-android` target
- Android NDK 26.1.10909125, or adjust the path in the build script
- Git submodules initialized

```bash
rustup target add aarch64-linux-android
```

---

## Build

```bash
./scripts/build.sh
```

Output:

```text
target/aarch64-linux-android/release/probe-rs-android
```

Manual build:

```bash
export ANDROID_NDK_HOME=$HOME/android-sdk/ndk/26.1.10909125
export TOOLCHAIN=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$TOOLCHAIN/bin/aarch64-linux-android30-clang
export CC_aarch64_linux_android=$CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER
export AR_aarch64_linux_android=$TOOLCHAIN/bin/llvm-ar
cargo build --target aarch64-linux-android --release
```

---

## Install on the phone

```bash
adb push target/aarch64-linux-android/release/probe-rs-android /data/local/tmp/
adb shell chmod 755 /data/local/tmp/probe-rs-android
```

Root is required to access `/dev/bus/usb`:

```bash
adb shell su -c '/data/local/tmp/probe-rs-android list'
```

---

## Usage

### List probes

```bash
probe-rs-android list
```

Example:

```text
Seeed Studio XIAO nRF54LM20A CMSIS-DAP
CmsisDapFactory
Accessible
```

### Flash

```bash
probe-rs-android flash \
  --chip nRF54LM20A \
  --file /data/local/tmp/xiao_nrf54lm20a.elf \
  --format elf \
  --protocol swd
```

Arguments:

```text
--chip       probe-rs target name, e.g. nRF54LM20A / SAMD21G18A / nRF52840_xxAA
--file       firmware file
--format     elf | hex | bin
--protocol   swd | jtag
--speed      SWD/JTAG clock in kHz
--probe      probe selector, e.g. 2886:0068 or a serial number
```

For BIN files, the current version uses the target's default boot address. Prefer ELF/HEX for production.

---

## Step-by-step tutorial

### 0. Prerequisites

- Linux host
- Rust / Cargo
- Android NDK 26.1.10909125
- Rooted Android phone
- USB debugging / ADB working
- CMSIS-DAP, WCH-Link, J-Link, or ST-Link probe

Install the Android Rust target:

```bash
rustup target add aarch64-linux-android
```

### 1. Clone the repository and submodules

```bash
git clone --recurse-submodules https://github.com/wentywenty/probe-rs-android.git
cd probe-rs-android
```

If already cloned without submodules:

```bash
git submodule update --init third_party/probe-rs third_party/nusb
```

### 2. Build the Android CLI

```bash
export ANDROID_NDK_HOME=$HOME/android-sdk/ndk/26.1.10909125
./scripts/build.sh
```

Output:

```text
target/aarch64-linux-android/release/probe-rs-android
```

### 3. Connect the phone

```bash
adb connect 10.43.19.110:5555
adb devices
```

Expected:

```text
10.43.19.110:5555    device
```

### 4. Install the CLI on the phone

```bash
adb push target/aarch64-linux-android/release/probe-rs-android /data/local/tmp/
adb shell chmod 755 /data/local/tmp/probe-rs-android
```

Android needs root for USB access:

```bash
adb shell su -c '/data/local/tmp/probe-rs-android list'
```

### 5. Enumerate probes

```bash
adb shell su -c '/data/local/tmp/probe-rs-android list'
```

Expected:

```text
Seeed Studio XIAO nRF54LM20A CMSIS-DAP
CmsisDapFactory
Accessible
```

If nothing is found:

- Check USB OTG / cable
- Check `lsusb`
- Check `/dev/bus/usb`
- Make sure the shell is root

### 6. Build real firmware (optional)

Example for `robopoly-xiao/gateway` on XIAO nRF54LM20A:

```bash
cd /home/robo/zephyrproject
west build \
  -b xiao_nrf54lm20a/nrf54lm20a/cpuapp \
  /home/robo/roboparty_repo/android/robopoly-xiao/gateway \
  --pristine \
  -d /home/robo/roboparty_repo/android/robopoly-xiao/build-nrf54lm20a
```

Firmware:

```text
build-nrf54lm20a/zephyr/zephyr.elf
```

### 7. Flash

```bash
adb push build-nrf54lm20a/zephyr/zephyr.elf /data/local/tmp/xiao.elf
adb shell su -c '/data/local/tmp/probe-rs-android flash \
  --chip nRF54LM20A \
  --file /data/local/tmp/xiao.elf \
  --format elf \
  --protocol swd'
```

Success:

```text
flashed /data/local/tmp/xiao.elf to nRF54LM20A
```

### 8. Troubleshooting

#### `no probes found`

- Probe is not connected properly
- Android did not enumerate the USB device
- Shell is not root
- `/dev/bus/usb` is not accessible

#### `permission denied`

```bash
adb shell su -c 'ls -l /dev/bus/usb/002/*'
```

Root or correct usbfs permissions are required.

#### `failed to attach`

- Wrong `--chip`
- SWDIO / SWCLK / GND wiring is wrong
- Target is not powered
- Readout protection or debug lock is enabled
- NRST may be required

#### CH347 is not detected

probe-rs CH347 PIDs:

```text
1a86:55dd CH347T
1a86:55de CH347F
1a86:55e8
```

If you see `1a86:8012`, that is **WCH-Link**, which uses CMSIS-DAP, not the CH347 backend.

#### Submodules are missing

```bash
git submodule update --init third_party/probe-rs third_party/nusb
```

---

## nusb Android patch

Patch:

```text
patches/0001-nusb-android-usbfs.patch
```

Based on upstream `nusb` main plus a single Android commit, the patch changes:

- `src/lib.rs`: enable `list_devices()` / `request_device()` on Android
- `src/enumeration.rs`: expose Linux `DeviceInfo` / `BusInfo` fields and methods on Android
- `src/platform/linux_usbfs/mod.rs`: enable usbfs enumeration / hotplug on Android
- `src/platform/linux_usbfs/device.rs`: open `/dev/bus/usb/BUS/ADDR` instead of `unimplemented!()`

Check or apply with:

```bash
git apply --check patches/0001-nusb-android-usbfs.patch
# or
cd nusb && git am /path/to/0001-nusb-android-usbfs.patch
```

The patch is authored and signed off by:

```text
wentywenty <2321901849@qq.com>
```

> The current implementation uses Linux usbfs with root and does not use the Android USB Host API. It is not usable without root.
> Hotplug code compiles, but hotplug behavior on Android has not been validated separately.

---

## Validation record

- `probe-rs-android list` detects Seeed XIAO nRF54LM20A CMSIS-DAP
- `probe-rs-android flash --chip nRF54LM20A` attaches successfully
- `robopoly-xiao/gateway` ELF was flashed to nRF54LM20A successfully
- Firmware resource usage: FLASH 5.27%, RAM 3.78%

---

## Known limitations

- Root only
- Depends on `/sys/bus/usb` and `/dev/bus/usb`
- Hotplug is not validated
- Dynamically linked against bionic; not a fully static ELF
- Full `probe-rs-tools` CLI features such as reset, info, and GDB server are not implemented
- Native CH347 mode has not been tested; current tests use CMSIS-DAP
- RP2350 support depends on the pinned probe-rs target database

---

## TODOs

- Non-root access via Android USB Host API + `libusb_wrap_sys_device`
- Validate native CH347 mode
- Add `info` / `reset` / `erase` subcommands
- Add RP2350 and other XIAO ARM targets if missing
- Evaluate fully static linking

---

## License

- `probe-rs`: MIT / Apache-2.0
- `nusb`: MIT / Apache-2.0
- This repository's Android patch and CLI: MIT / Apache-2.0
