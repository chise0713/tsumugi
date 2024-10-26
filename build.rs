use std::{env, path::PathBuf, process::Command};

fn main() {
    let outdir = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed=src/protos/geoip.proto");
    prost_build::compile_protos(&["src/protos/geoip.proto"], &["src/"]).unwrap();
    println!("cargo:rerun-if-env-changed=GO111MODULE");
    println!("cargo:rerun-if-env-changed=GCCGO");
    println!("cargo:rerun-if-env-changed=GOARCH");
    println!("cargo:rerun-if-env-changed=GOBIN");
    println!("cargo:rerun-if-env-changed=GOCACHE");
    println!("cargo:rerun-if-env-changed=GOMODCACHE");
    println!("cargo:rerun-if-env-changed=GODEBUG");
    println!("cargo:rerun-if-env-changed=GOENV");
    println!("cargo:rerun-if-env-changed=GOFLAGS");
    println!("cargo:rerun-if-env-changed=GOINSECURE");
    println!("cargo:rerun-if-env-changed=GOOS");
    println!("cargo:rerun-if-env-changed=GOPATH");
    println!("cargo:rerun-if-env-changed=GOPROXY");
    println!("cargo:rerun-if-env-changed=GOPRIVATE");
    println!("cargo:rerun-if-env-changed=GONOPROXY");
    println!("cargo:rerun-if-env-changed=GONOSUMDB");
    println!("cargo:rerun-if-env-changed=GOROOT");
    println!("cargo:rerun-if-env-changed=GOSUMDB");
    println!("cargo:rerun-if-env-changed=GOTOOLCHAIN");
    println!("cargo:rerun-if-env-changed=GOTMPDIR");
    println!("cargo:rerun-if-env-changed=GOVCS");
    println!("cargo:rerun-if-env-changed=GOWORK");
    println!("cargo:rerun-if-changed=libsrs/");
    let status = Command::new("go")
        .current_dir(env::current_dir().unwrap().join("libsrs/"))
        .args([
            "build",
            "-buildmode=c-archive",
            "-o",
            &outdir
                .join(if cfg!(target_os = "windows") {
                    "srs.lib"
                } else {
                    "libsrs.a"
                })
                .to_string_lossy(),
        ])
        .status()
        .unwrap();
    if !status.success() {
        panic!("Failed to build Go code");
    }
    bindgen::Builder::default()
        .header(
            outdir
                .join(if cfg!(target_os = "windows") {
                    "srs.h"
                } else {
                    "libsrs.h"
                })
                .to_string_lossy(),
        )
        .allowlist_function("read_cidr_rule")
        .allowlist_function("write_cidr_rule")
        //.allowlist_function("read_domain_rule")
        //.allowlist_function("write_domain_rule")
        .generate()
        .unwrap()
        .write_to_file(outdir.join("libsrs.rs"))
        .unwrap();
    println!(
        "cargo:rustc-link-search=native={}",
        outdir.to_string_lossy()
    );
    println!("cargo:rustc-link-lib=static=srs");
}
