#![feature(box_syntax)]

#[macro_use] 
extern crate log;

use std::{env, fs::{self, File}};
use std::path::Path;
use std::io;

use env_logger::Env;
use core::{models::io::ImageWriterOptions, plugins::{ImageFormatSupportPlugin, PluginManager}};

const DEFAULT_LOGGING_LEVEL: &str = "info";
const PLUGINS_DIR: &str = "plugins";

type Plugins = Vec<Box<dyn ImageFormatSupportPlugin>>;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or(DEFAULT_LOGGING_LEVEL)).init();
    let args: Vec<String> = env::args().collect();
    debug!("args are: {:?}", args);

    let mut plugin_manager = PluginManager::new();
    if let Err(err) = fs::create_dir_all(PLUGINS_DIR) {
        error!("failed to create plugins directory: {}", err);
    }
    let plugins: Plugins = match plugin_manager.load_plugins(box Path::new(PLUGINS_DIR)) {
        Ok(v) => v,
        Err(err) => {
            error!("failed to load plugins: {}", err);
            return;
        }
    };

    if args.len() > 1 && args[1] == "plugins" {
        if args.len() > 2 && args[2] == "download" {
            if args.len() > 3 {
                install_plugin(&args[3]);
            } else {
                error!("please specify plugin name to download, for example: gif_support"); 
            }
        } else {
            error!("please specify plugins action, for example: download");
        }
    } else if argument_present(&args, "source") && argument_present(&args, "goal-format") {
        let from_file = argument_value(&args, "source")
            .expect("expected from file to be present because checked that argument is present");
        let to_format = argument_value(&args, "goal-format")
            .expect("expected to format to be present, because argument is present");

        convert_file(&plugins, &from_file, &to_format);
    } else {
        error!("please specify command:\nconverter --source=example.bmp --goal-format=gif\nconverter plugins install gif_support");
    }
}

fn convert_file(plugins: &Plugins, from_file: &str, to_format: &str) {
    info!("Converting file {} to {}", from_file, to_format);

    let file = match fs::read(&from_file) {
        Ok(v) => v,
        Err(err) => {
            error!("failed to read {}: {}", &from_file, err);
            return
        }
    };

    let extension = match Path::new(from_file).extension() {
        Some(v) => v.to_string_lossy().to_lowercase(),
        None => {
            error!("failed to detect extension of file {}", from_file);
            return;
        }
    };

    info!("source extension is {}", extension);
    let source_plugin = match plugins.iter()
        .find(|v| v.format_name().eq_ignore_ascii_case(&extension)) {
        Some(v) => v,
        None => {
            error!("Failed to find plugin to read {}. Did you install it? Try running \"converter plugins install {}_support\"", extension, extension);
            return;
        }
    };
    let target_plugin = match plugins.iter()
        .find(|v| v.format_name().eq_ignore_ascii_case(&to_format)) {
        Some(v) => v,
        None => {
            error!("Failed to find plugin to write {}. Did you install it? Try running \"converter plugins install {}_support\"", extension, extension);
            return;
        }
    };

    let images = match source_plugin.reader().read(&file) {
        Ok(v) => v,
        Err(err) => {
            error!("Failed to read image as {}: {}", extension, err);
            return;
        }
    };

    info!("done reading {} image{}", images.len(), if images.len() > 1 { "s" } else { "" });
    
    let mut counter = 0;
    for image in images {
        info!("Converting image #{} to {}", counter, &to_format);
        let converted = match target_plugin.writer().write(&image, &ImageWriterOptions::default()) {
            Ok(v) => v,
            Err(err) => {
                error!("Failed to convert image to {}: {}", &to_format, err);
                return;
            }
        };

        let save_to = format!("./result_{}.{}", counter, to_format);
        match fs::write(&save_to, &converted) {
            Ok(_) => info!("Result saved to {}", &save_to),
            Err(err) => {
                error!("Failed to save result: {}", err);
                return
            }
        };

        counter += 1;
    }
}

fn argument_value(args: &Vec<String>, argument_name: &str) -> Option<String> {
    args.iter()
        .find(|s| s.starts_with(&format!("--{}=", argument_name)))
        .map(|s| s[s.find("=").expect("expected equals sign to be present because checked for that in filter")+1..].to_string())
}

fn argument_present(args: &Vec<String>, argument_name: &str) -> bool {
    args.iter().find(|s| s.starts_with(&format!("--{}=", argument_name))).is_some()
}

fn install_plugin(plugin_name: &str) {
    let library_name = if cfg!(windows) {
        format!("{}.dll", plugin_name)
    } else {
        format!("lib{}.so", plugin_name)
    };
    let plugin_path = format!("{}/{}", PLUGINS_DIR, library_name);
    let plugin_path_as_path = Path::new(&plugin_path);
    if plugin_path_as_path.exists() {
        if let Err(err) = fs::remove_file(&plugin_path) {
            warn!("failed to remove existing file{} ", err);
        }
    }

    info!("downloading plugin \"{}\" to {}", plugin_name, plugin_path);

    let mut resp = reqwest::blocking::get(format!("https://turbocow.nikitavbv.com/plugins/{}", library_name))
        .expect("failed to download");
    if resp.status() == 404 {
        error!("Failed to download plugin. Plugin with name \"{}\" does not exist", plugin_name);
        return;
    } else if resp.status() != 200 {
        error!("Failed to download plugin, status code = {}", resp.status());
        return;
    }

    let mut file = File::create(plugin_path).expect("failed to create file");
    io::copy(&mut resp, &mut file).expect("failed to save downloaded file");

    info!("plugin \"{}\" downloaded", plugin_name);
}