extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let current_dir = env::current_dir().unwrap();
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

    let libwebrtc_path = current_dir.join("deps/libwebrtc");
    let webrtc_include_path = env::var("LIBWEBRTC_INCLUDE_PATH").unwrap_or(
        libwebrtc_path
            .join("include")
            .into_os_string()
            .into_string()
            .unwrap(),
    );
    let webrtc_binary_path = env::var("LIBWEBRTC_BINARY_PATH").unwrap_or(
        libwebrtc_path
            .join("lib")
            .join(env::var("TARGET").unwrap())
            .into_os_string()
            .into_string()
            .unwrap(),
    );

    let dst = cmake::Config::new(".")
        .define("LIBWEBRTC_INCLUDE_PATH:PATH", &webrtc_include_path)
        .define("LIBWEBRTC_BINARY_PATH:PATH", &webrtc_binary_path)
        .define("MEDIASOUPCLIENT_LOG_DEV", "ON")
        .build();

    let lib_path = dst.join("lib");

    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-search=native={}", webrtc_binary_path);

    println!("cargo:rustc-link-lib=static=vulcast-rtc");
    println!("cargo:rustc-link-lib=static=webrtcextra");

    match env::var("PROFILE").unwrap().as_str() {
        "release" => println!("cargo:rustc-link-lib=static=glog"),
        "debug" => println!("cargo:rustc-link-lib=static=glogd"),
        _ => panic!("invalid build PROFILE"),
    }

    println!("cargo:rustc-link-lib=static=webrtc");
    println!("cargo:rustc-link-lib=static=mediasoupclient");
    println!("cargo:rustc-link-lib=static=sdptransform");
}
