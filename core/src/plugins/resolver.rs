use std::{fs::File, io, path::Path};
use std::fs;

use log::*;

use super::plugins::{ImageFormatSupportPlugin, PluginManager, PluginManagerError};

pub struct PluginResolver<'a> {

    plugins_dir: Box<&'a Path>,
    // DO NOT change order of fields here. Plugins need to be freed earliner than underlying libraries.
    plugins: Vec<Box<dyn ImageFormatSupportPlugin>>,
    manager: PluginManager,
}

impl <'a> PluginResolver<'a> {

    pub fn new(plugins_dir: Box<&'a Path>) -> Result<Self, PluginManagerError> {
        let mut manager = PluginManager::new();
        let plugins = manager.load_plugins(plugins_dir.clone())?;
        
        Ok(Self {
            plugins_dir,
            manager,
            plugins,
        })
    }

    pub fn resolve_or_install_image_support(&mut self, image_format: &str) -> &Box<dyn ImageFormatSupportPlugin> {
        if self.resolve_image_support(image_format).is_none() {
            let plugin_name = format!("{}_support", image_format.to_lowercase());
            let plugin_path = plugin_installation_path(
                self.plugins_dir.to_str().expect("Cannot get string path of plugin installation dir"), 
                &plugin_name
            );
    

            self.install_plugin(&plugin_name);

            let loaded_plugin = self.manager.load_plugin(&box Path::new(&plugin_path))
                .expect("Failed to load plugin which has just been installed");
            &self.plugins.push(loaded_plugin);
        }

        self.resolve_image_support(image_format)
            .expect("Expected plugin to be present now")
    }

    pub fn resolve_image_support(&self, image_format: &str) -> Option<&Box<dyn ImageFormatSupportPlugin>> {
        self.plugins.iter()
            .find(|v| v.format_name().eq_ignore_ascii_case(&image_format))
    }

    fn install_plugin(&self, plugin_name: &str) {
        let plugin_path = plugin_installation_path(
            self.plugins_dir.to_str().expect("Cannot get string path of plugin installation dir"), 
            plugin_name
        );
        if is_plugin_installed(&plugin_path, plugin_name) {
            if let Err(err) = fs::remove_file(&plugin_path) {
                warn!("failed to remove existing file{} ", err);
            }
        }

        info!("downloading plugin \"{}\" to {}", plugin_name, plugin_path);

        let mut resp = reqwest::blocking::get(format!("https://turbocow.nikitavbv.com/plugins/{}", plugin_library_name(plugin_name)))
            .expect("failed to download");
        if resp.status() == 404 {
            panic!("Failed to download plugin. Plugin with name \"{}\" does not exist", plugin_name);
        } else if resp.status() != 200 {
            panic!("Failed to download plugin, status code = {}", resp.status());
        }

        let mut file = File::create(plugin_path).expect("failed to create file");
        io::copy(&mut resp, &mut file).expect("failed to save downloaded file");

        info!("plugin \"{}\" downloaded", plugin_name);
    }
}

fn is_plugin_installed(plugins_dir: &str, name: &str) -> bool {
    let plugin_path = plugin_installation_path(plugins_dir, name);
    let plugin_path_as_path = Path::new(&plugin_path);
    plugin_path_as_path.exists()
}

fn plugin_installation_path(plugins_dir: &str, plugin_name: &str) -> String {
    format!("{}/{}", plugins_dir, plugin_library_name(plugin_name))
}

fn plugin_library_name(plugin_name: &str) -> String {
    if cfg!(windows) {
        format!("{}.dll", plugin_name)
    } else {
        format!("lib{}.so", plugin_name)
    }
}