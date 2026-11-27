{
  description = "Clippit: clipboard history, launcher, spell checker and local AI for GNOME on Wayland";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (pkgs: rec {
        clippit = pkgs.callPackage ./nix/package.nix { };
        default = clippit;
      });

      overlays.default = final: _prev: {
        clippit = final.callPackage ./nix/package.nix { };
      };

      homeManagerModules.default = import ./nix/home-manager.nix self;
      nixosModules.default = import ./nix/nixos.nix self;

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          inputsFrom = [ self.packages.${pkgs.stdenv.hostPlatform.system}.clippit ];
          packages = with pkgs; [
            rust-analyzer
            clippy
            rustfmt
            nodejs # `node --check` on the GNOME extension, like the CI
          ];
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };
      });

      formatter = forAllSystems (pkgs: pkgs.nixfmt);
    };
}
