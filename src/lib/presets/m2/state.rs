use actix_web::HttpRequest;
use app_state::AppState;
use from_file::FromFile;

use presets::m2::bundle_config::BundleConfig;
use presets::m2::bundle_config::Module;
use presets::m2::config_gen;
use presets::m2::opts::M2PresetOptions;
use presets::m2::requirejs_config::RequireJsBuildConfig;

pub fn gather_state(
    req: &HttpRequest<AppState>,
) -> Result<(RequireJsBuildConfig, Vec<Module>), String> {
    let modules = &req
        .state()
        .req_log
        .lock()
        .expect("should lock & unwrap req_log");

    let client_config = req
        .state()
        .rjs_client_config
        .lock()
        .expect("should lock & unwrap rjs_client_config");

    let maybe_opts = M2PresetOptions::get_opts(&req.state().program_config)
        .expect("should clone program config");

    let bundle_config = match maybe_opts.bundle_config {
        Some(bc_path) => BundleConfig::from_file(&bc_path),
        None => Ok(BundleConfig::default()),
    };

    match bundle_config {
        Err(e) => Err(e.to_string()),
        Ok(bundle_config) => {
            let module_blacklist = bundle_config.module_blacklist.clone().unwrap_or(vec![]);
            let mut blacklist = vec!["js-translation".to_string()];
            blacklist.extend(module_blacklist);

            let filtered = RequireJsBuildConfig::drop_blacklisted(&modules.to_vec(), &blacklist);
            let bundle_modules = config_gen::generate_modules(filtered, bundle_config);
            let mut derived_build_config = RequireJsBuildConfig::default();

            derived_build_config.deps = client_config.deps.clone();
            derived_build_config.map = client_config.map.clone();
            derived_build_config.config = client_config.config.clone();
            derived_build_config.shim = client_config.shim.clone();

            let mut c = client_config.paths.clone();
            derived_build_config.paths = RequireJsBuildConfig::strip_paths(&c);

            derived_build_config.modules = Some(bundle_modules.clone());

            Ok((derived_build_config, bundle_modules))
        }
    }
}
