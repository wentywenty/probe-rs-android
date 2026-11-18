English | [简体中文](README.zh-CN.md)

# probe-rs-android

在 Android 上运行 `probe-rs`，通过 USB 调试探针枚举、连接和烧录 ARM Cortex-M 目标。

当前主要用于 **XIAO nRF54LM20A** 的 SWD 烧录验证，也支持 probe-rs 已有的 CMSIS-DAP / J-Link / ST-Link / WCH-Link 等探针。

---

## 状态

已在以下环境实测：

- 手机：Xiaomi 23049RAD8C，Android 15，arm64，KernelSU root
- 探针：Seeed Studio XIAO nRF54LM20A CMSIS-DAP（`2886:0068`）
- 目标：XIAO nRF54LM20A
- 固件：`robopoly-xiao/gateway`，Zephyr `xiao_nrf54lm20a/nrf54lm20a/cpuapp`
- 结果：probe 枚举、target attach、ELF 烧录均成功

WCH-Link（`1a86:8012`）也能被识别为 CMSIS-DAP 并访问。

---

## 为什么需要这个仓库

`probe-rs` 本身已经支持 CH347、CMSIS-DAP 等探针，但它的 USB 后端 `nusb` 默认关闭了 Android 的 usbfs 枚举和设备打开。

本仓库提供：

1. 一个 Android 可执行 CLI：`probe-rs-android`
2. 一份对 `nusb 0.2.9` 的 Android patch
3. 可复现的 NDK 构建脚本

没有修改 GKI 内核，也没有依赖 CH347 内核驱动。

---

## 目录

```text
.
├── Cargo.toml
├── Cargo.lock
├── src/main.rs
├── scripts/build.sh
├── patches/0001-nusb-android-usbfs.patch
└── third_party/
    ├── probe-rs/           # git submodule, pinned probe-rs
    └── nusb/               # git submodule, upstream nusb
```

---

## 构建

需要：

- Rust + Cargo
- `aarch64-linux-android` target
- Android NDK 26.1.10909125（或修改脚本中的路径）
- Git submodule 已初始化

```bash
rustup target add aarch64-linux-android
./scripts/build.sh
```

产物：

```text
target/aarch64-linux-android/release/probe-rs-android
```

也可以手动构建：

```bash
git submodule update --init third_party/probe-rs third_party/nusb
rm -rf build/nusb
mkdir -p build
cp -a third_party/nusb build/nusb
rm -rf build/nusb/.git
patch -d build/nusb -p1 < patches/0001-nusb-android-usbfs.patch

export ANDROID_NDK_HOME=$HOME/android-sdk/ndk/26.1.10909125
export TOOLCHAIN=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$TOOLCHAIN/bin/aarch64-linux-android30-clang
export CC_aarch64_linux_android=$CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER
export AR_aarch64_linux_android=$TOOLCHAIN/bin/llvm-ar
cargo build --target aarch64-linux-android --release
```

---

## 初始化子模块

首次 clone 后：

```bash
git submodule update --init third_party/probe-rs third_party/nusb
```

或直接运行构建脚本，它会自动初始化这两个 submodule。

## 安装到手机

```bash
adb push target/aarch64-linux-android/release/probe-rs-android /data/local/tmp/
adb shell chmod 755 /data/local/tmp/probe-rs-android
```

Android 上需要 root 访问 `/dev/bus/usb`：

```bash
adb shell su -c '/data/local/tmp/probe-rs-android list'
```

---

## 使用

### 枚举探针

```bash
probe-rs-android list
```

示例：

```text
Seeed Studio XIAO nRF54LM20A CMSIS-DAP
CmsisDapFactory
Accessible
```

### 烧录

```bash
probe-rs-android flash \
  --chip nRF54LM20A \
  --file /data/local/tmp/xiao_nrf54lm20a.elf \
  --format elf \
  --protocol swd
```

参数：

```text
--chip       probe-rs target 名称，如 nRF54LM20A / SAMD21G18A / nRF52840_xxAA
--file       固件文件
--format     elf | hex | bin
--protocol   swd | jtag
--speed      SWD/JTAG 频率，单位 kHz
--probe      探针选择器，如 2886:0068 或序列号
```

BIN 需要地址时，当前版本使用目标默认 boot 地址；生产使用建议优先 ELF/HEX。

---

## 保姆教程：从零到烧录

下面按顺序执行即可。

### 0. 前置条件

- Linux 主机
- Rust / Cargo
- Android NDK 26.1.10909125
- 已 root 的 Android 手机
- USB 调试 / ADB 可用
- CMSIS-DAP、WCH-Link、J-Link 或 ST-Link 等探针

安装 Android Rust target：

```bash
rustup target add aarch64-linux-android
```

### 1. 克隆仓库和子模块

```bash
git clone --recurse-submodules https://github.com/wentywenty/probe-rs-android.git
cd probe-rs-android
```

如果已经 clone 但没带子模块：

```bash
git submodule update --init third_party/probe-rs third_party/nusb
```

### 2. 构建 Android CLI

```bash
export ANDROID_NDK_HOME=$HOME/android-sdk/ndk/26.1.10909125
./scripts/build.sh
```

产物：

```text
target/aarch64-linux-android/release/probe-rs-android
```

### 3. 连接手机

```bash
adb connect 10.43.19.110:5555
adb devices
```

应该看到：

```text
10.43.19.110:5555    device
```

### 4. 安装 CLI 到手机

```bash
adb push target/aarch64-linux-android/release/probe-rs-android /data/local/tmp/
adb shell chmod 755 /data/local/tmp/probe-rs-android
```

Android 需要 root 访问 USB：

```bash
adb shell su -c '/data/local/tmp/probe-rs-android list'
```

### 5. 枚举探针

```bash
adb shell su -c '/data/local/tmp/probe-rs-android list'
```

正常会看到类似：

```text
Seeed Studio XIAO nRF54LM20A CMSIS-DAP
CmsisDapFactory
Accessible
```

如果没有：

- 检查 USB OTG / 线
- 检查 `lsusb`
- 检查 `/dev/bus/usb`
- 确认 root

### 6. 编译一个真实固件（可选）

以 `robopoly-xiao/gateway` 的 XIAO nRF54LM20A 为例：

```bash
cd /home/robo/zephyrproject
west build   -b xiao_nrf54lm20a/nrf54lm20a/cpuapp   /home/robo/roboparty_repo/android/robopoly-xiao/gateway   --pristine   -d /home/robo/roboparty_repo/android/robopoly-xiao/build-nrf54lm20a
```

固件：

```text
build-nrf54lm20a/zephyr/zephyr.elf
```

### 7. 烧录

```bash
adb push build-nrf54lm20a/zephyr/zephyr.elf /data/local/tmp/xiao.elf
adb shell su -c '/data/local/tmp/probe-rs-android flash \
  --chip nRF54LM20A \
  --file /data/local/tmp/xiao.elf \
  --format elf \
  --protocol swd'
```

成功输出：

```text
flashed /data/local/tmp/xiao.elf to nRF54LM20A
```

### 8. 常见问题

#### `no probes found`

- 探针没插好
- Android 没识别 USB
- 不是 root
- `/dev/bus/usb` 不可访问

#### `permission denied`

```bash
adb shell su -c 'ls -l /dev/bus/usb/002/*'
```

需要 root 或正确的 usbfs 权限。

#### `failed to attach`

- `--chip` 名字不对
- SWDIO/SWCLK/GND 没接对
- 目标没供电
- 目标有读保护/调试锁
- 需要接 NRST

#### CH347 没出现

probe-rs 的 CH347 PID 是：

```text
1a86:55dd CH347T
1a86:55de CH347F
1a86:55e8
```

如果看到 `1a86:8012`，那是 **WCH-Link**，走 CMSIS-DAP，不是 CH347 backend。

#### 子模块没初始化

```bash
git submodule update --init third_party/probe-rs third_party/nusb
```

---

## Patch 签名

`patches/0001-nusb-android-usbfs.patch` 使用标准 `git format-patch` 格式：

```text
From: wentywenty <2321901849@qq.com>
Subject: [PATCH] nusb: add Android usbfs enumeration and device open
Signed-off-by: wentywenty <2321901849@qq.com>
```

应用到 nusb：

```bash
cd nusb
git am /path/to/0001-nusb-android-usbfs.patch
```

或只检查：

```bash
git apply --check /path/to/0001-nusb-android-usbfs.patch
```

## 依赖版本

`third_party/probe-rs` 固定到：

```text
d77197c2b3a4063a69f43b6c580234c4a7f81d43
```

`third_party/nusb` 固定到上游 main：

```text
c46ccd0183844b6299b5354a4a13d6837d5fcf63
```

Android patch 由 `scripts/build.sh` 应用到 `build/nusb` 临时目录，submodule 保持干净。

## nusb Android patch

patch：

```text
patches/0001-nusb-android-usbfs.patch
```

基于 `nusb 0.2.9`，修改：

- `src/lib.rs`：Android 启用 `list_devices()` / `request_device()`
- `src/enumeration.rs`：Android 暴露 Linux 的 `DeviceInfo` / `BusInfo` 字段和方法
- `src/platform/linux_usbfs/mod.rs`：Android 启用 usbfs enumeration / hotplug
- `src/platform/linux_usbfs/device.rs`：实现 `/dev/bus/usb/BUS/ADDR` 打开，替换 `unimplemented!()`

校验：

```bash
git apply --check patches/0001-nusb-android-usbfs.patch
```

> 注意：当前实现走 **Linux usbfs + root**，没有使用 Android USB Host API，因此非 root 环境不可用。  
> hotplug 代码能编译，但 Android 上的热插拔行为尚未单独验证。

---

## 验证记录

- `probe-rs-android list` 能识别 Seeed XIAO nRF54LM20A CMSIS-DAP
- `probe-rs-android flash --chip nRF54LM20A` attach 成功
- `robopoly-xiao/gateway` 的 ELF 已成功烧录到 nRF54LM20A
- 资源占用：FLASH 5.27%，RAM 3.78%

---

## 已知限制

- 仅 root
- 依赖 `/sys/bus/usb` 和 `/dev/bus/usb`
- hotplug 未验证
- 动态链接 bionic，不是完全静态 ELF
- 尚未实现 probe-rs-tools 的完整 CLI（如 reset、info、gdb server）
- CH347 原生模式未实测；当前测试使用的是 CMSIS-DAP
- RP2350 target 尚未在当前 probe-rs 目标库中确认

---

## 后续

- 非 root USB Host API + `libusb_wrap_sys_device`
- 验证 CH347 原生模式
- 增加 `info` / `reset` / `erase` 子命令
- 补 RP2350 / 其他 XIAO ARM target
- 评估完全静态链接

---

## License

- `probe-rs`：MIT / Apache-2.0
- `nusb`：MIT / Apache-2.0
- 本仓库的 Android patch 与 CLI：MIT / Apache-2.0
