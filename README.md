An attempt to make a minecraft launcher in Rust. Since Rust GUI libraries are in a pretty bad state right now, CLI only for the moment.

If anyone knows GUI dev, I can expose the library as a C API so the UI can be written in C/C++.

Current state: It downloads version manifests. Not much else.

Goals:
- Symlink/hardlink everything to save space on mods, resource packs, etc. (like pnpm)
- Built-in mod updating and downloading
- Support Modrinth, Curseforge, Github Releases, Gitlab releases, and building from source for mods
- Better modpack format than curseforge's scuffed format, with more features, better ability to update modpacks, and support for platforms other than curseforge
- Speed (there are currently 629 versions of minecraft, so this is spawning 629 tasks. The worst part is this actually has good performance compared to single threaded):
  https://gitlab.com/obj-obj/copper_rs/-/blob/cb177deb85e41589d074a15e454fae57d7837b63/copper_cli/src/main.rs#L74-80