{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = inputs:
    with inputs;
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [cargo2nix.overlays.default];
          };
          workspaceShell = rustPkgs.workspaceShell {
            packages = [pkgs.alejandra pkgs.rustfmt];
            shellHook = ''
              export PS1="\033[0;31m☠dev-shell☠ $ \033[0m";
            '';
          };
          rustPkgs = pkgs.rustBuilder.makePackageSet {
            rustVersion = "1.75.0"; # TODO: is this the MSRV?
            packageFun = import ./Cargo.nix;
          };
        in rec {
          devShells = {
            default = workspaceShell;
          };
          packages = {
            # replace hello-world with your package name
            mirafetch = rustPkgs.workspace.mirafetch {};
            default = packages.mirafetch;
          };
        }
      );
}
