# Goonie Buddy

Virtual strippers are the new black.

## Building

Windows is the only supported build target, Linux/macOS might work.

Rust depends on a few Visual Studio components, the individual components
"Windows 10/11 SDK" and "MSVC vXXX VS 20XX C++ x64/x86 build tools".

Then just "cargo build"

**Note that** as of writing the spawned window is both transparent and lacks decorations, so
it's completely invisible bar a taskbar icon. So you have to kill it via Ctrl+Cing in the terminal,
basically.