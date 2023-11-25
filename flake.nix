{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-utils.url = "github:numtide/flake-utils";

  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        inherit (pkgs) lib;

        craneLib = crane.lib.${system};
        src = craneLib.path ./.;

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src;

          buildInputs = [
            pkgs.openssl
            pkgs.pkg-config
            # Add additional build inputs here
          ] ++ lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin; [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
            pkgs.darwin.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ]);

          # Additional environment variables can be set directly
          # MY_CUSTOM_VAR = "some value";
        };

        craneLibLLvmTools = craneLib.overrideToolchain
          (fenix.packages.${system}.complete.withComponents [
            "cargo"
            "llvm-tools"
            "rustc"
          ]);

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        my-crate = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });
      in
      {
        packages = {
          default = my-crate;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = my-crate;
        };

        devShells.default = pkgs.mkShell {
          # Extra inputs can be added here
          nativeBuildInputs = with pkgs; [
            cargo
            rustc
          ] ++ lib.optionals pkgs.stdenv.isDarwin (with pkgs; with pkgs.darwin; [
            # Additional darwin specific inputs can be set here
            libiconv
            Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ]);
        };
      });
}
