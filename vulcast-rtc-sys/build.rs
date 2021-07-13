extern crate bindgen;

use std::env;
use std::path::PathBuf;

use fs_extra::dir::CopyOptions;

fn main() {
    let current_dir = env::current_dir().unwrap();
    let prebuilt_path = current_dir
        .join("prebuilt")
        .join(env::var("TARGET").unwrap())
        .join(env::var("PROFILE").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // create bindgen bindings
    let bindings = bindgen::Builder::default()
        .header("src/wrapper.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("unable to generate bindings");
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("could not write bindings");

    if env::var("VULCAST_RTC_FROM_SOURCE").is_ok() {
        // build from source
        let webrtc_include_path =
            env::var("LIBWEBRTC_INCLUDE_PATH").expect("webrtc include path not set");
        let webrtc_binary_path =
            env::var("LIBWEBRTC_BINARY_PATH").expect("webrtc binary path not set");

        let dst = cmake::Config::new(".")
            .define("LIBWEBRTC_INCLUDE_PATH:PATH", &webrtc_include_path)
            .define("LIBWEBRTC_BINARY_PATH:PATH", &webrtc_binary_path)
            .build();

        let lib_path = dst.join("lib");

        println!("cargo:rustc-link-search=native={}", lib_path.display());

        if env::var("VULCAST_RTC_UPDATE_PREBUILTS").is_ok() {
            let _ = std::fs::remove_dir_all(&prebuilt_path);
            fs_extra::copy_items(
                &[lib_path],
                prebuilt_path,
                &CopyOptions {
                    overwrite: true,
                    copy_inside: true,
                    ..CopyOptions::default()
                },
            )
            .expect("failed to copy prebuilt libraries");
        }
    } else {
        // build from prebuilts
        println!("cargo:rustc-link-search=native={}", prebuilt_path.display());
    }

    println!("cargo:rustc-link-lib=static=vulcast-rtc");
    println!("cargo:rustc-link-lib=static=webrtc_broadcaster");

    match env::var("PROFILE").unwrap().as_str() {
        "release" => println!("cargo:rustc-link-lib=static=glog"),
        "debug" => println!("cargo:rustc-link-lib=static=glogd"),
        _ => panic!("invalid build PROFILE"),
    }

    println!("cargo:rustc-link-lib=static=webrtc");
    println!("cargo:rustc-link-lib=static=mediasoupclient");
    println!("cargo:rustc-link-lib=static=sdptransform");
}
