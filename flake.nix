{
  description = "Rust development shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            # Toolchain
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer

            # Nice-to-haves
            bacon        # background checker: re-runs check/clippy/test on save
            cargo-watch  # run any cargo command on file changes
            cargo-edit   # cargo add / rm / upgrade helpers
          ];

          # Lets rust-analyzer find the standard library sources
          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

          # Full backtraces when something panics
          RUST_BACKTRACE = "1";
        };
      });
    };
}
