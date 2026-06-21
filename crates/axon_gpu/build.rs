// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// build.rs -- compile GLSL compute shaders to SPIR-V using naga.

use std::path::PathBuf;

fn main() {
    let shaders = ["add", "mul", "scale", "relu", "matmul"];
    let shader_dir = PathBuf::from("shaders");
    let out_dir    = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    for name in &shaders {
        let glsl_path = shader_dir.join(format!("{}.glsl", name));
        let spv_path  = out_dir.join(format!("{}.spv", name));

        println!("cargo:rerun-if-changed={}", glsl_path.display());

        let glsl = std::fs::read_to_string(&glsl_path)
            .unwrap_or_else(|e| panic!("read {}: {}", glsl_path.display(), e));

        // Parse GLSL → naga IR
        let mut frontend = naga::front::glsl::Frontend::default();
        let options = naga::front::glsl::Options {
            stage: naga::ShaderStage::Compute,
            defines: Default::default(),
        };
        let module = frontend.parse(&options, &glsl)
            .unwrap_or_else(|e| panic!("naga parse {}: {:?}", name, e));

        // Validate
        let info = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap_or_else(|e| panic!("naga validate {}: {:?}", name, e));

        // Emit SPIR-V
        let spv_options = naga::back::spv::Options {
            lang_version: (1, 3),
            ..Default::default()
        };
        let spv_words = naga::back::spv::write_vec(&module, &info, &spv_options, None)
            .unwrap_or_else(|e| panic!("naga spv {}: {:?}", name, e));

        // Write as binary
        let spv_bytes: Vec<u8> = spv_words.iter()
            .flat_map(|w| w.to_ne_bytes())
            .collect();
        std::fs::write(&spv_path, &spv_bytes)
            .unwrap_or_else(|e| panic!("write {}: {}", spv_path.display(), e));

        println!("cargo:warning=compiled shader: {} → {}", name, spv_path.display());
    }
}
