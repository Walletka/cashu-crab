#!/bin/bash
BINDINGS_DIR="../../bindings"
TARGET_DIR="../../target"
PROJECT_DIR="cashu-sdk-kotlin-android"
PACKAGE_DIR="com/tchaika"
UNIFFI_BINDGEN_BIN="cargo run -p uniffi-bindgen"
ANDROID_NDK_ROOT="/home/cencul/android/android-ndk-r26"
LLVM_ARCH_PATH="linux-x86_64"
PATH="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/$LLVM_ARCH_PATH/bin:$PATH"

rustup target add x86_64-linux-android aarch64-linux-android armv7-linux-androideabi
echo "Building"
CFLAGS="-D__ANDROID_MIN_SDK_VERSION__=21" AR=llvm-ar CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="x86_64-linux-android21-clang" CC="x86_64-linux-android21-clang" cargo build --target x86_64-linux-android || exit 1
CFLAGS="-D__ANDROID_MIN_SDK_VERSION__=21" AR=llvm-ar CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="armv7a-linux-androideabi21-clang" CC="armv7a-linux-androideabi21-clang" cargo build --target armv7-linux-androideabi || exit 1
CFLAGS="-D__ANDROID_MIN_SDK_VERSION__=21" AR=llvm-ar CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="aarch64-linux-android21-clang" CC="aarch64-linux-android21-clang" cargo build --target aarch64-linux-android || exit 1
echo "--------------------------------"
$UNIFFI_BINDGEN_BIN generate src/cashu_sdk.udl --language kotlin -o "$BINDINGS_DIR"/"$PROJECT_DIR"/lib/src/main/kotlin/"$PACKAGE_DIR" || exit 1

JNI_LIB_DIR="$BINDINGS_DIR"/"$PROJECT_DIR"/lib/src/main/jniLibs/ 
mkdir -p $JNI_LIB_DIR/x86_64 || exit 1
mkdir -p $JNI_LIB_DIR/armeabi-v7a || exit 1
mkdir -p $JNI_LIB_DIR/arm64-v8a || exit 1
cp $TARGET_DIR/x86_64-linux-android/debug/libcashu_sdk_ffi.so $JNI_LIB_DIR/x86_64/ || exit 1
cp $TARGET_DIR/armv7-linux-androideabi/debug/libcashu_sdk_ffi.so $JNI_LIB_DIR/armeabi-v7a/ || exit 1
cp $TARGET_DIR/aarch64-linux-android/debug/libcashu_sdk_ffi.so $JNI_LIB_DIR/arm64-v8a/ || exit 1
