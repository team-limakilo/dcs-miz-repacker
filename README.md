# DCS Miz Repacker

This is a small Rust application with the following goals:

* Ability to repack missions with several different time presets
* Ability to repack custom mission files that DCS does not keep between re-saves (ie. spectator camera settings)
* Ability to configure random weather settings with weighted distribution
* Robust error handling, with readable error messages but also allowing automated runs from other scripts (ie. server restarter)

What is not supported yet:

* Changing wind values
* Dynamic weather settings (waiting for ED's implementation)

## Usage

* Place this .exe in your DCS missions folder
* Create a repack.toml configuration file with the presets and weather settings (see `example/repack.toml`)
* Optionally, create a repack folder and add any files you want to be automatically replaced inside of the generated .miz
* Drag and drop the miz file into the .exe

## Non-goals

* Modifying any actual mission objects (units, structures, etc) or mission triggers/scripting
