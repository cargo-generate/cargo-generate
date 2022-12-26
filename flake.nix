{
  description = "cargo, make me a project";
  inputs = {
    flake-compat = {
      flake = false;
      url = "github:edolstra/flake-compat";
    };
    nci = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:yusdacra/nix-cargo-integration";
    };
    nix-filter.url = "github:numtide/nix-filter";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs = inputs:
    let
      name = "cargo-generate";
      nix-filter = import inputs.nix-filter;
      pkgs = common: packages:
        builtins.map (element: common.pkgs.${element}) packages;
    in inputs.nci.lib.makeOutputs {
      config = common: {
        cCompiler = { package = common.pkgs.clang; };
        outputs = {
          defaults = {
            app = name;
            package = name;
          };
        };
        runtimeLibs = pkgs common [ "openssl" ];
      };
      pkgConfig = common:
        let
          override = {
            buildInputs = pkgs common [ "openssl" "perl" "pkg-config" ];
          };
        in {
          ${name} = {
            app = true;
            build = true;
            depsOverrides = { inherit override; };
            overrides = { inherit override; };
            profiles = { release = false; };
          };
        };
      root = nix-filter {
        root = ./.;
        include = [ ./Cargo.lock ./Cargo.toml ./README.md ./src ];
      };
    };
}