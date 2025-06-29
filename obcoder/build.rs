use std::env;
use std::ops::Deref;
use std::os::windows::fs::symlink_file;
use std::path::PathBuf;

fn main() {
    // env::set_var("OUT_DIR", "src");
    env::set_var("PKG_CONFIG_PATH" ,"D:/vcpkg/installed/x64-windows/lib/pkgconfig");
    // 为每个库单独创建配置并探测
    let libavcodec = pkg_config::Config::new()
        .atleast_version("1.0")
        .probe("libavcodec")
        .unwrap();

    let libavutil = pkg_config::Config::new()
        .atleast_version("1.0")
        .probe("libavutil")
        .unwrap();

    let libavformat = pkg_config::Config::new()
        .atleast_version("1.0")
        .probe("libavformat")
        .unwrap();

    let libswscale = pkg_config::Config::new()
        .atleast_version("1.0")
        .probe("libswscale")
        .unwrap();

    let libavfilter = pkg_config::Config::new()
        .atleast_version("1.0")
        .probe("libavfilter")
        .unwrap();

    let libswresample = pkg_config::Config::new()
        .atleast_version("1.0")
        .probe("libswresample")
        .unwrap();

    let libavdevice = pkg_config::Config::new()
        .atleast_version("1.0")
        .probe("libavdevice")
        .unwrap();

    // 如果需要使用这些库信息，可以继续处理
    println!("All FFmpeg libraries found successfully!");

    let bindings = bindgen::Builder::default()
        .header("src/include/wrapper.h")
        .clang_arg("-ID:/vcpkg/installed/x64-windows/include")
        .clang_arg("-Isrc/include")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(".");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");



    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // 假设您的动态库位于项目根目录下的 'lib' 文件夹
    let lib_path = manifest_dir.join("lib");

    // 打印链接搜索路径。
    // `native` 表示本地搜索路径，`dylib` 表示要链接一个动态库。
    println!("cargo:rustc-link-search=native={}", lib_path.display());

    // 打印要链接的动态库名称。
    // `dylib` 关键字是可选的，但明确指出链接动态库是个好习惯。
    println!("cargo:rustc-link-lib=dylib={}", "ob_codec");
    println!("cargo:rustc-link-lib=dylib={}", "ob_stream");
    println!("cargo:rustc-link-lib=dylib={}", "ob_video");
    println!("cargo:rustc-link-lib=dylib={}", "rb");

    println!("cargo:rerun-if-changed=lib");

}