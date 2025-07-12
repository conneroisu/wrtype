{
  description = "A development shell for rust";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    crane,
    treefmt-nix,
    ...
  }: let
    # Define systems
    systems = [
      "x86_64-linux"
      "aarch64-linux"
      "aarch64-darwin"
      "x86_64-darwin"
    ];

    # Helper function to generate per-system attributes
    forAllSystems = f: nixpkgs.lib.genAttrs systems f;
  in {
    # Define packages using crane to build
    packages = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      };
      craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.stable.latest.default);
      
      commonArgs = {
        src = ./.;
        strictDeps = true;
        
        buildInputs = with pkgs; [
          wayland
          wayland-protocols
          libxkbcommon
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.libiconv
        ];

        nativeBuildInputs = with pkgs; [
          pkg-config
        ];
        
        # Set environment variable for wayland-protocols location
        WAYLAND_PROTOCOLS_DIR = "${pkgs.wayland-protocols}/share/wayland-protocols";
      };
      
      cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
        # Force rebuild of dependencies with protocol files
        pname = "wrtype-deps";
      });
    in {
      default = craneLib.buildPackage (commonArgs // {
        inherit cargoArtifacts;
        
        meta = with pkgs.lib; {
          description = "A Rust implementation of wtype - xdotool type for Wayland";
          homepage = "https://github.com/conneroisu/wrtype";
          license = licenses.mit;
          maintainers = [ ];
          platforms = platforms.linux;
        };
      });
      
      wrtype = self.packages.${system}.default;
    });

    # Define devShells for all systems
    devShells = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      };
      # Optional: Initialize crane for building packages
      # craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.stable.latest.default);
      # Optional: Example crane package build (uncomment to use)
      # my-crate = craneLib.buildPackage {
      #   src = craneLib.cleanCargoSource ./.;
      #   strictDeps = true;
      # };
    in {
      default = pkgs.mkShell {
        name = "dev";
        # Available packages on https://search.nixos.org/packages
        buildInputs = with pkgs; [
          alejandra # Nix
          nixd
          statix
          deadnix
          just
          rust-bin.stable.latest.default
          # Wayland development dependencies
          wayland
          wayland-protocols
          libxkbcommon
          pkg-config
        ];
        shellHook = ''
          echo "Welcome to the rust devshell!"
        '';
      };
    });

    formatter = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      };
      treefmtModule = {
        projectRootFile = "flake.nix";
        programs = {
          alejandra.enable = true; # Nix formatter
          rustfmt.enable = true; # Rust formatter
        };
      };
    in
      treefmt-nix.lib.mkWrapper pkgs treefmtModule);
  };
}
