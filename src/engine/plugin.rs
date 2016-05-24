/*================================================================================================*/
// Copyright 2016 Kyle Finlay
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
/*================================================================================================*/

extern crate glob;
extern crate libloading;

use self::glob::glob;
use self::libloading::{Library, Symbol};

use std::vec::Vec;

/*================================================================================================*/
/*------PLUGINTYPE ENUM---------------------------------------------------------------------------*/
/*================================================================================================*/

/// The Plugin Type enum.
#[derive (Copy, Clone, PartialEq, Debug)]
pub enum PluginType {

    /// Used by audio backends.
    AudioBackend,
    /// Used by rendering backends.
    RenderBackend,
    /// Used by the window backends.
    WindowBackend
}

/*================================================================================================*/
/*------PLUGINSTATE ENUM--------------------------------------------------------------------------*/
/*================================================================================================*/

/// Stores the current state of the plugin.
#[derive (Copy, Clone, PartialEq)]
pub enum PluginState {

    /// The plugin is unloaded.
    /// This is the default state.
    Unloaded,
    /// The plugin has been marked for loading.
    MarkedForLoad,
    /// The plugin is loaded.
    Loaded,
    /// The plugin is disabled, and cannot be used.
    /// This state is normally used when an error has occured.
    Disabled
}

/*================================================================================================*/
/*------PLUGIN STRUCT-----------------------------------------------------------------------------*/
/*================================================================================================*/

/// Stores the details of a plugin.
#[derive (Clone)]
pub struct Plugin {

    // Public
    /// The name of the plugin.
    pub name: String,
    /// The author of the plugin.
    pub author: String,
    /// A brief description of the plugin.
    pub description: String,
    /// The path to the plugin.
    pub path: String,
    /// The type of plugin.
    pub plugin_type: PluginType,
    /// The current state of the plugin.
    pub plugin_state: PluginState
}

/*================================================================================================*/
/*------PLUGINMANAGER STRUCT----------------------------------------------------------------------*/
/*================================================================================================*/

/// Manages the finding and loading of plugins.
#[derive (Clone)]
pub struct PluginManager {

    // Public
    /// The list of plugins
    pub plugin_list: Vec<Plugin>,

    // Private
    _plugin_ext: String
}

/*================================================================================================*/
/*------PLUGINMANAGER PUBLIC MEMBERS--------------------------------------------------------------*/
/*================================================================================================*/

impl PluginManager {

    /// Returns a new plugin manager.
    ///
    /// # Arguments
    /// * `plugin_dir` - The location of the plugin directory.
    ///
    /// # Return value
    /// A new instance of the plugin manager.
    pub fn new () -> PluginManager {

        // Set the platform extension
        let plug_ext = if cfg! (target_os = "windows") {".dll"}
                       else if cfg! (target_os = "linux") {".so"}
                       else if cfg! (target_os = "macos") {".dylib"}
                       else {panic! ("Platform unsupported")};

        PluginManager {plugin_list: Vec::new (),
                       _plugin_ext: plug_ext.to_owned ()}
    }

    /// Queries the plugin directory, and stores a list of plugins.
    ///
    /// # Arguments
    /// * `plugin_dir` - The directory which contains the plugins.
    ///
    /// # Return value
    /// An immutable reference to the list of plugins.
    pub fn query_plugins (&mut self, plugin_dir: &str) -> &Vec<Plugin> {

        // Clear the old plugin list
        self.plugin_list.clear ();

        info! ("Searching for plugins...");

        // Recurse through all items in the plugin directory
        for path in glob (&format! ("{}/*{}", plugin_dir, &self._plugin_ext)).unwrap ().filter_map (Result::ok) {

            // Load the library, and get function symbols
            let lib = Library::new (&path).unwrap ();

            let get_name: Symbol<unsafe extern fn () -> String> = unsafe {
                lib.get (b"get_name\0").unwrap ()
            };

            let get_author: Symbol<unsafe extern fn () -> String> = unsafe {
                lib.get (b"get_author\0").unwrap ()
            };

            let get_description: Symbol<unsafe extern fn () -> String> = unsafe {
                lib.get (b"get_description\0").unwrap ()
            };

            let get_type: Symbol<unsafe extern fn () -> PluginType> = unsafe {
                lib.get (b"get_type\0").unwrap ()
            };

            unsafe {

                // Add the plugin to the list
                self.plugin_list.push (Plugin {name: get_name (),
                                               author: get_author (),
                                               description: get_description (),
                                               path: path.to_str ().unwrap ().to_owned (),
                                               plugin_type: get_type (),
                                               plugin_state: PluginState::Unloaded});

                info! ("Found: {:?}", &path);
            }
        }

        if self.plugin_list.is_empty () {
            info! ("No plugins found.");
        }

        else {
            info! ("Plugin searching complete.");
        }

        &self.plugin_list
    }

    /// Returns an instance of a plugin
    ///
    /// # Arguments
    /// * `name` - The name of the plugin.
    ///
    /// # Return value
    /// A result contaning a reference the plugin.
    pub fn get_plugin (&self, name: &str) -> Result<&Plugin, ()> {

        let mut index = 0;

        // Loop through all plugins
        for p in &self.plugin_list {

            if p.name == name {
                return Ok (&self.plugin_list [index]);
            }

            index += 1;
        }

        Err (())
    }
}
