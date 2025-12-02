{
  description = "Ambiente de desarrollo para Kalk (Rust + Iced + Wayland)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
        };

        # Dependencias de Linux optimizadas para Wayland
        linuxDeps = with pkgs; lib.optionals stdenv.isLinux [
          pkg-config
          freetype
          expat
          fontconfig
          vulkan-loader # Iced usa WGPU, que corre sobre Vulkan en Wayland
          libxkbcommon  # Crucial para el teclado en Wayland
          wayland       # Protocolo base
          # Aunque usemos Wayland, mantenemos estas por si winit necesita símbolos
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];

        # Frameworks de Mac (por si compartes el código)
        darwinDeps = with pkgs; lib.optionals stdenv.isDarwin [
          libiconv darwin.apple_sdk.frameworks.AppKit darwin.apple_sdk.frameworks.CoreGraphics
          darwin.apple_sdk.frameworks.Foundation darwin.apple_sdk.frameworks.Metal
          darwin.apple_sdk.frameworks.QuartzCore
        ];

        runtimeLibs = linuxDeps;

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [ rustToolchain pkgs.just ] ++ darwinDeps ++ linuxDeps;

          # LÍNEA MÁGICA PARA LINUX:
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;

          # FORZAR WAYLAND:
          # Esto le dice a Winit (la ventana) que ignore X11 y use Wayland nativo.
          # Si estás en GNOME/KDE/Hyprland, esto hará que la ventana se dibuje nativa.
          WINIT_UNIX_BACKEND = "wayland";

          shellHook = ''
            echo "Entorno Kalk activado"
          '';
        };
      }
    );
}
