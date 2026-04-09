use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=shaders");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    let shaders_dir = Path::new("shaders");

    let mut compiler = shaderc::Compiler::new().expect("Failed to initialize shaderc compiler");
    let mut options =
        shaderc::CompileOptions::new().expect("Failed to initialize compiler options");

    options.set_include_callback(
        |requested_source, _include_type, _requesting_source, _depth| {
            let include_path = shaders_dir.join(requested_source);
            match fs::read_to_string(&include_path) {
                Ok(content) => Ok(shaderc::ResolvedInclude {
                    resolved_name: include_path.to_string_lossy().to_string(),
                    content,
                }),
                Err(err) => Err(format!(
                    "Failed to resolve include {:?}: {}",
                    include_path, err
                )),
            }
        },
    );

    options.set_target_env(
        shaderc::TargetEnv::Vulkan,
        shaderc::EnvVersion::Vulkan1_4 as u32,
    );

    if env::var("PROFILE").unwrap() == "release" {
        options.set_optimization_level(shaderc::OptimizationLevel::Performance);
    } else {
        options.set_generate_debug_info();
    }

    compile_shaders_recursively(shaders_dir, shaders_dir, out_dir, &mut compiler, &options);
}

fn compile_shaders_recursively(
    base_dir: &Path,
    current_dir: &Path,
    out_dir: &Path,
    compiler: &mut shaderc::Compiler,
    options: &shaderc::CompileOptions,
) {
    if !current_dir.is_dir() {
        return;
    }

    for entry in fs::read_dir(current_dir).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_dir() {
            compile_shaders_recursively(base_dir, &path, out_dir, compiler, options);
            continue;
        }

        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        let shader_kind = match extension {
            "vert" => shaderc::ShaderKind::Vertex,
            "frag" => shaderc::ShaderKind::Fragment,
            "comp" => shaderc::ShaderKind::Compute,
            "mesh" => shaderc::ShaderKind::Mesh,
            "task" => shaderc::ShaderKind::Task,
            "glsl" => continue,
            _ => continue,
        };

        let source_text = fs::read_to_string(&path).expect("Failed to read shader file");

        let compiled = compiler
            .compile_into_spirv(
                &source_text,
                shader_kind,
                &path.to_string_lossy(),
                "main",
                Some(options),
            )
            .unwrap_or_else(|err| panic!("Failed to compile shader {:?}:\n{}", path, err));

        let relative_path = path.strip_prefix(base_dir).unwrap();
        let out_path = out_dir
            .join(relative_path)
            .with_extension(format!("{}.spv", extension));

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create nested directories in OUT_DIR");
        }

        fs::write(&out_path, compiled.as_binary_u8()).expect("Failed to write SPIR-V file");
    }
}
