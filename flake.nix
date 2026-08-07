{
  description = "Ambiente de desarrollo para Kalk (Rust + TUI)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Toolchain estable
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
          ];
        };

        # `nix run` levanta el arbol de trabajo, no un release pinneado:
        # sirve para probar lo que se esta editando.
        runScript = pkgs.writeShellScriptBin "kalk-dev" ''
          exec ${rustToolchain}/bin/cargo run "$@"
        '';
      in
      {
        apps.default = {
          type = "app";
          program = "${runScript}/bin/kalk-dev";
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.just
            pkgs.cargo-audit
            pkgs.cargo-deny
            pkgs.cargo-edit
            pkgs.cargo-machete
          ];

          shellHook = ''
            echo "> Entorno Kalk TUI activado"
          '';
        };
      }
    );
}
