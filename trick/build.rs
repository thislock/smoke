use std::{env, path::Path};

// this just makes sure the sdl3 lib files are compiled and present without you having to do anything

fn main() {
  let out_dir = env::var("OUT_DIR").unwrap();
  let sdl_path = Path::new(&out_dir).join("SDL");

  // Clone SDL3 repo if missing
  if !sdl_path.exists() {
    println!("cargo:compiling=Cloning SDL3 repository...");
    let repo_url = "https://github.com/libsdl-org/SDL.git";
    let mut repo_builder = git2::build::RepoBuilder::new();
    repo_builder
      .clone(repo_url, &sdl_path)
      .expect("Failed to clone SDL3");
  }

  // Use cmake to configure and build SDL3
  println!("cargo:compiling=Building SDL3 with CMake...");
  let dst = cmake::Config::new(&sdl_path)
    .define("SDL_STATIC", "ON") // Build static library
    .define("SDL_SHARED", "OFF") // Disable shared lib
    .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
    .build();

  // The cmake crate puts build output under dst/lib
  let lib_dir = dst.join("lib");

  // Link search path + static lib
  println!("cargo:rustc-link-search=native={}", lib_dir.display());
  println!("cargo:rustc-link-lib=static=SDL3");

  // Common dependencies for linking on Linux
  println!("cargo:rustc-link-lib=dylib=dl");
  println!("cargo:rustc-link-lib=dylib=pthread");
  println!("cargo:rustc-link-lib=dylib=m");
  println!("cargo:rustc-link-lib=dylib=stdc++");

  // Re-run build.rs if this file or git repo changes
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed={}", sdl_path.display());
}
