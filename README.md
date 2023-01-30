An attempt to make a minecraft launcher in Rust.

Goals:

- Interface with a Quilt mod, for mod updating/downloading ingame
- Support Modrinth, GitHub/GitLab, and compiling from source
- Symlink/hardlink everything to save space on mods, resource packs, etc. (like pnpm)
- (Maybe) make an extended version of Modrinth's modpack format
- Speed:
  https://github.com/obj-obj/copper_rs/blob/7103b0ab2bbc80df5c2646983bf4884a708e521d/copper_cli/src/main.rs#L83-L90
