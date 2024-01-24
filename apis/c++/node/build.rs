use std::path::{Component, Path, PathBuf};

fn main() {
    let mut bridge_files = vec![PathBuf::from("src/lib.rs")];
    #[cfg(feature = "ros2-bridge")]
    bridge_files.push(ros2::generate());

    let _build = cxx_build::bridges(dbg!(&bridge_files));
    println!("cargo:rerun-if-changed=src/lib.rs");

    #[cfg(feature = "ros2-bridge")]
    ros2::generate_ros2_message_header(bridge_files.last().unwrap());

    // to avoid unnecessary `mut`` warning
    bridge_files.clear();
}

#[cfg(feature = "ros2-bridge")]
mod ros2 {
    use std::path::{Component, Path, PathBuf};

    pub fn generate() -> PathBuf {
        use rust_format::Formatter;
        let paths = ament_prefix_paths();
        let generated = dora_ros2_bridge_msg_gen::gen(paths.as_slice(), true);
        let generated_string = rust_format::PrettyPlease::default()
            .format_tokens(generated)
            .unwrap();
        let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
        let target_file = out_dir.join("ros2_bindings.rs");
        std::fs::write(&target_file, generated_string).unwrap();
        println!(
            "cargo:rustc-env=ROS2_BINDINGS_PATH={}",
            target_file.display()
        );

        target_file
    }

    fn ament_prefix_paths() -> Vec<PathBuf> {
        let ament_prefix_path: String = match std::env::var("AMENT_PREFIX_PATH") {
            Ok(path) => path,
            Err(std::env::VarError::NotPresent) => {
                println!("cargo:warning='AMENT_PREFIX_PATH not set'");
                String::new()
            }
            Err(std::env::VarError::NotUnicode(s)) => {
                panic!(
                    "AMENT_PREFIX_PATH is not valid unicode: `{}`",
                    s.to_string_lossy()
                );
            }
        };
        println!("cargo:rerun-if-env-changed=AMENT_PREFIX_PATH");

        let paths: Vec<_> = ament_prefix_path.split(':').map(PathBuf::from).collect();
        for path in &paths {
            println!("cargo:rerun-if-changed={}", path.display());
        }

        paths
    }

    pub fn generate_ros2_message_header(source_file: &Path) {
        let out_dir = source_file.parent().unwrap();
        let relative_path = local_relative_path(&source_file)
            .ancestors()
            .nth(2)
            .unwrap()
            .join("out");
        let header_path = out_dir
            .join("cxxbridge")
            .join("include")
            .join("dora-node-api-cxx")
            .join(&relative_path)
            .join("ros2_bindings.rs.h");
        let code_path = out_dir
            .join("cxxbridge")
            .join("sources")
            .join("dora-node-api-cxx")
            .join(&relative_path)
            .join("ros2_bindings.rs.cc");

        // copy message files to target directory
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .unwrap();
        let target_path = root
            .join("target")
            .join("cxxbridge")
            .join("dora-node-api-cxx")
            .join("src")
            .join("ros2_bindings.rs.h");

        std::fs::copy(dbg!(&header_path), dbg!(&target_path)).unwrap();
        println!("cargo:rerun-if-changed={}", header_path.display());
        std::fs::copy(
            &code_path,
            target_path.with_file_name("ros2_bindings.rs.cc"),
        )
        .unwrap();
        println!("cargo:rerun-if-changed={}", code_path.display());
    }

    // copy from cxx-build source
    fn local_relative_path(path: &Path) -> PathBuf {
        let mut rel_path = PathBuf::new();
        for component in path.components() {
            match component {
                Component::Prefix(_) | Component::RootDir | Component::CurDir => {}
                Component::ParentDir => drop(rel_path.pop()), // noop if empty
                Component::Normal(name) => rel_path.push(name),
            }
        }
        rel_path
    }
}
