use std::fs;
use std::path::Path;

use custom_error::custom_error;
use libloading::{Library, Symbol};
use log::*;

use crate::models::io::{ImageReader, ImageWriter};

custom_error! {pub PluginManagerError
    FailedToLoadLibrary{description: String} = "Failed to load library: {description}",
    InvalidPlugin{description: String} = "Invalid plugin: {description}",
    IOError{description: String} = "IOError: {description}"
}

pub trait ImageFormatSupportPlugin {

    fn format_name(&self) -> String;

    fn reader(&self) -> Box<dyn ImageReader>;
    fn writer(&self) -> Box<dyn ImageWriter>;
}

pub type PluginInit = unsafe fn () -> Box<dyn ImageFormatSupportPlugin>;

pub struct PluginManager {
    loaded_libraries: Vec<Library>,
}

impl PluginManager {

    pub fn new() -> Self {
        PluginManager {
            loaded_libraries: Vec::new(),
        }
    }

    pub fn load_plugins(&mut self, plugins_directory: Box<&Path>) -> Result<Vec<Box<dyn ImageFormatSupportPlugin>>, PluginManagerError> {
        info!("loading plugins...");
        
        let plugins: Vec<Box<dyn ImageFormatSupportPlugin>> = fs::read_dir(plugins_directory.as_ref())
            .map_err(|err| PluginManagerError::IOError { description: err.to_string() })?
            .into_iter()
            .filter_map(|v| v.ok())
            .map(|v| v.file_name().into_string())
            .filter_map(|v| v.ok())
            .filter(|v| v.to_lowercase().ends_with(".so") || v.to_lowercase().ends_with(".dll"))
            .map(|v| plugins_directory.join(v.clone()))
            .map(|v| (v.clone(), self.load_plugin(&v)))
            .filter_map(|(path, v)| match v {
                Ok(v) => {
                    info!("loaded plugin: support for {}",  v.format_name());
                    Some(v)
                },
                Err(err) => {
                    error!("failed to load plugin ({}): {}", path.to_string_lossy(), err);
                    None
                }
            })
            .collect();

        info!("loaded {} plugins", plugins.len());

        Ok(plugins)
    }

    pub fn load_plugin(&mut self, library_path: &Path) -> Result<Box<dyn ImageFormatSupportPlugin>, PluginManagerError> {
        Ok(unsafe {
            let lib = Library::new(library_path)
                .map_err(|err| PluginManagerError::FailedToLoadLibrary { description: err.to_string() })?;
            
            // it is important to prevent library from being deleted from memory
            self.loaded_libraries.push(lib);
            let lib = match self.loaded_libraries.last() {
                Some(v) => v,
                None => return Err(
                    PluginManagerError::FailedToLoadLibrary { description: "failed to get library from loaded libraries vec".to_string() }
                )
            };

            lib.get::<Symbol<PluginInit>>(b"_plugin_init")
                .map_err(|err| PluginManagerError::InvalidPlugin { description: format!("failed to invoke _plugin_init: {}", err) })?()
        })
    }
}