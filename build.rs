// Copyright (c) 2020-2021, Cosmos Rust authors (licensed under the Apache License Version 2.0)
// Modifications Copyright (c) 2021-present Crypto.org(licensed under the Apache License, Version 2.0)

//! Build CosmosSDK/Tendermint/IBC proto files. This build script clones the CosmosSDK version
//! specified in the COSMOS_REV constant and then uses that to build the required
//! proto files for further compilation. This is based on the proto-compiler code
//! in github.com/informalsystems/ibc-rs

#[cfg(feature = "grpc")]
mod grpc {
    use regex::Regex;
    use std::{
        ffi::OsStr,
        fs::{self, create_dir_all, remove_dir_all},
        io,
        path::{Path, PathBuf},
        process,
    };
    use walkdir::WalkDir;

    /// The Cosmos commit or tag to be cloned and used to build the proto files
    const COSMOS_REV: &str = "v0.42.0";
    // TODO: use version tag
    const TENDERMINT_REV: &str = "v0.34.8";

    // All paths must end with a / and either be absolute or include a ./ to reference the current
    // working directory.

    /// The directory generated proto files go into in this repo
    const COSMOS_SDK_PROTO_DIR: &str = "./src/proto";
    /// Directory where the submodule is located
    const COSMOS_SDK_DIR: &str = "./cosmos-sdk-go";
    const TENDERMINT_DIR: &str = "./tendermint";
    /// A temporary directory for proto building
    const TMP_BUILD_DIR: &str = "./protobuf/";

    // Patch strings used by `copy_and_patch`

    /// Protos belonging to these Protobuf packages will be excluded
    /// (i.e. because they are sourced from `tendermint-proto`)
    const EXCLUDED_PROTO_PACKAGES: &[&str] = &["gogoproto", "google"];
    /// Regex for locating instances of `tendermint-proto` in prost/tonic build output
    const TENDERMINT_PROTO_REGEX: &str = "(super::)+tendermint";
    /// Attribute preceeding a Tonic client definition
    const TONIC_CLIENT_ATTRIBUTE: &str = "#[doc = r\" Generated client implementations.\"]";
    /// Attributes to add to gRPC clients
    const GRPC_CLIENT_ATTRIBUTES: &[&str] = &[
        "#[cfg(feature = \"grpc\")]",
        "#[cfg_attr(docsrs, doc(cfg(feature = \"grpc\")))]",
        TONIC_CLIENT_ATTRIBUTE,
    ];

    #[cfg(feature = "grpc")]
    pub fn build() {
        let tmp_build_dir: PathBuf = TMP_BUILD_DIR.parse().unwrap();
        let proto_dir: PathBuf = COSMOS_SDK_PROTO_DIR.parse().unwrap();

        if tmp_build_dir.exists() {
            fs::remove_dir_all(&tmp_build_dir).unwrap();
        }

        fs::create_dir(tmp_build_dir.clone()).unwrap();

        update_submodule();
        output_sdk_version(&tmp_build_dir);
        compile_cosmos_protos(&tmp_build_dir);
        compile_proto_services(&tmp_build_dir);
        compile_tendermint_protos(&tmp_build_dir);
        copy_generated_files(&tmp_build_dir, &proto_dir);
        fs::remove_dir_all(&tmp_build_dir).unwrap();
    }

    fn run_git(args: impl IntoIterator<Item = impl AsRef<OsStr>>) {
        let exit_status = process::Command::new("git")
            .args(args)
            .status()
            .expect("git exit status missing");

        if !exit_status.success() {
            panic!("git exited with error code: {:?}", exit_status.code());
        }
    }

    fn update_submodule() {
        run_git(&["submodule", "update", "--init"]);
        run_git(&["-C", COSMOS_SDK_DIR, "fetch"]);
        run_git(&["-C", COSMOS_SDK_DIR, "reset", "--hard", COSMOS_REV]);
        run_git(&["-C", TENDERMINT_DIR, "fetch"]);
        run_git(&["-C", TENDERMINT_DIR, "reset", "--hard", TENDERMINT_REV]);
    }

    fn output_sdk_version(out_dir: &Path) {
        let path = out_dir.join("COSMOS_SDK_COMMIT");
        fs::write(path, COSMOS_REV).unwrap();
        let path = out_dir.join("TENDERMINT_COMMIT");
        fs::write(path, TENDERMINT_REV).unwrap();
    }

    fn find_proto_files(proto_paths: Vec<String>) -> Vec<PathBuf> {
        let mut protos: Vec<PathBuf> = vec![];
        for proto_path in &proto_paths {
            protos.append(
                &mut WalkDir::new(proto_path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_type().is_file()
                            && e.path().extension().is_some()
                            && e.path().extension().unwrap() == "proto"
                    })
                    .map(|e| e.into_path())
                    .collect(),
            );
        }
        protos
    }

    fn compile_tendermint_protos(out_dir: &Path) {
        let tendermint_dir = Path::new(TENDERMINT_DIR);
        let proto_paths = vec![format!("{}/proto/tendermint/rpc", tendermint_dir.display())];
        let proto_includes_paths = vec![
            format!("{}/proto/", tendermint_dir.display()),
            format!("{}/third_party/proto", tendermint_dir.display()),
        ];
        let protos = find_proto_files(proto_paths);
        let includes: Vec<PathBuf> = proto_includes_paths.iter().map(PathBuf::from).collect();

        // Compile all proto files
        if let Err(e) = tonic_build::configure()
            .build_client(true)
            .build_server(true)
            .format(true)
            .out_dir(out_dir)
            .compile(&protos, &includes)
        {
            eprintln!("[error] couldn't compile protos: {}", e);
            panic!("protoc failed!");
        }
    }

    fn compile_cosmos_protos(out_dir: &Path) {
        let sdk_dir = Path::new(COSMOS_SDK_DIR);

        // Paths
        let proto_paths = vec![
            format!("{}/proto/ibc", sdk_dir.display()),
            format!("{}/proto/cosmos/tx", sdk_dir.display()),
            format!("{}/proto/cosmos/bank", sdk_dir.display()),
            format!("{}/proto/cosmos/base", sdk_dir.display()),
            format!("{}/proto/cosmos/staking", sdk_dir.display()),
        ];

        let proto_includes_paths = vec![
            format!("{}/proto", sdk_dir.display()),
            format!("{}/third_party/proto", sdk_dir.display()),
        ];

        // List available proto files
        let protos = find_proto_files(proto_paths);

        // List available paths for dependencies
        let includes: Vec<PathBuf> = proto_includes_paths.iter().map(PathBuf::from).collect();

        // Compile all proto files
        let mut config = prost_build::Config::default();
        config.out_dir(out_dir);
        config.extern_path(".tendermint", "crate::proto::tendermint");

        if let Err(e) = config.compile_protos(&protos, &includes) {
            eprintln!("[error] couldn't compile protos: {}", e);
            panic!("protoc failed!");
        }
    }

    fn compile_proto_services(out_dir: impl AsRef<Path>) {
        // let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let sdk_dir = PathBuf::from(COSMOS_SDK_DIR);

        let proto_includes_paths = [
            // root.join("../proto"),
            sdk_dir.join("proto"),
            sdk_dir.join("third_party/proto"),
        ];

        // List available paths for dependencies
        let includes = proto_includes_paths
            .iter()
            .map(|p| p.as_os_str().to_os_string())
            .collect::<Vec<_>>();

        let proto_services_path = [
            sdk_dir.join("proto/cosmos/auth/v1beta1/query.proto"),
            sdk_dir.join("proto/cosmos/staking/v1beta1/query.proto"),
            sdk_dir.join("proto/cosmos/tx/v1beta1/service.proto"),
            sdk_dir.join("proto/cosmos/tx/v1beta1/tx.proto"),
        ];

        // List available paths for dependencies
        let services = proto_services_path
            .iter()
            .map(|p| p.as_os_str().to_os_string())
            .collect::<Vec<_>>();

        // Compile all proto client for GRPC services
        println!("[info ] Compiling proto clients for GRPC services!");
        tonic_build::configure()
            .build_client(true)
            .build_server(false)
            .format(true)
            .out_dir(out_dir)
            .compile(&services, &includes)
            .unwrap();

        println!("[info ] => Done!");
    }

    fn copy_generated_files(from_dir: &Path, to_dir: &Path) {
        println!(
            "[info ] Copying generated files into '{}'...",
            to_dir.display()
        );

        // Remove old compiled files
        remove_dir_all(&to_dir).unwrap_or_default();
        create_dir_all(&to_dir).unwrap();

        let mut filenames = Vec::new();

        // Copy new compiled files (prost does not use folder structures)
        let errors = WalkDir::new(from_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| {
                let filename = e.file_name().to_os_string().to_str().unwrap().to_string();
                filenames.push(filename.clone());
                copy_and_patch(e.path(), format!("{}/{}", to_dir.display(), &filename))
            })
            .filter_map(|e| e.err())
            .collect::<Vec<_>>();

        if !errors.is_empty() {
            for e in errors {
                eprintln!("[error] Error while copying compiled file: {}", e);
            }

            panic!("[error] Aborted.");
        }
    }

    fn copy_and_patch(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> io::Result<()> {
        // Skip proto files belonging to `EXCLUDED_PROTO_PACKAGES`
        for package in EXCLUDED_PROTO_PACKAGES {
            if let Some(filename) = src.as_ref().file_name().and_then(OsStr::to_str) {
                if filename.starts_with(&format!("{}.", package)) {
                    return Ok(());
                }
            }
        }

        let contents = fs::read_to_string(src)?;

        // `prost-build` output references types from `tendermint-proto` crate
        // relative paths, which we need to munge into `tendermint_proto` in
        // order to leverage types from the upstream crate.
        let contents = Regex::new(TENDERMINT_PROTO_REGEX)
            .unwrap()
            .replace_all(&contents, "crate::proto::tendermint");

        // Patch each service definition with a feature attribute
        let patched_contents =
            contents.replace(TONIC_CLIENT_ATTRIBUTE, &GRPC_CLIENT_ATTRIBUTES.join("\n"));

        fs::write(dest, patched_contents)
    }
}

fn main() {
    #[cfg(feature = "grpc")]
    grpc::build();
}
