{
  description = "quiqr-tui (Reveal) — kiosked WP5.1-style TUI CMS frontend for Quiqr";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # TODO(author): wire in YOUR Quiqr flake. You author the Quiqr Nix module +
    # package, so replace this with the real input. The e2e VM imports its
    # NixOS module to run Quiqr Server.
    #
    #   quiqr.url = "github:quiqr/quiqr-nix";   # <-- replace with the real ref
    #   quiqr.inputs.nixpkgs.follows = "nixpkgs";
    #
    # Until then the e2e VM is scaffolded but the Quiqr service block is a
    # placeholder (see checks.e2e below).
  };

  outputs = inputs@{ flake-parts, nixpkgs, rust-overlay, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];

      perSystem = { system, pkgs, lib, ... }:
        let
          pkgs' = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          rustToolchain = pkgs'.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "clippy" "rustfmt" ];
          };

          # Tools the agent and developers need in the shell.
          # NOTE: beans, openspec, and the agent CLI (claude) may not be in
          # nixpkgs. Prefer pinned flake inputs or small derivations over
          # dropping them. Placeholders below — the agent resolves these in E1.
          devTools = with pkgs'; [
            rustToolchain
            hugo
            git
            nodejs_22        # openspec + beans (if npm-distributed) run on Node
            # TODO(E1): add `beans`, `openspec`, and the `claude` CLI here via
            # the appropriate input/derivation. Document the choice in flake.
          ];
        in
        {
          _module.args.pkgs = pkgs';

          devShells.default = pkgs'.mkShell {
            packages = devTools;
            shellHook = ''
              echo "quiqr-tui dev shell — run 'beans prime' then read CLAUDE.md"
            '';
          };

          # `nix build`
          packages.default = pkgs'.rustPlatform.buildRustPackage {
            pname = "quiqr-tui";
            version = "0.0.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;  # exists after E1 scaffolding
          };

          # `nix flake check` runs unit/integration tests on the host.
          checks.unit = pkgs'.rustPlatform.buildRustPackage {
            pname = "quiqr-tui-tests";
            version = "0.0.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            doCheck = true;
            buildPhase = "cargo test --workspace --no-run";
            checkPhase = "cargo test --workspace";
          };

          # End-to-end: boot a VM running Quiqr Server, provision a sample site,
          # run qtui headless with the fake agent, assert the full flow.
          # E1 scaffolds this to boot + build-check; E7 fills the scenario.
          checks.e2e = pkgs'.testers.runNixOSTest {
            name = "quiqr-tui-e2e";
            nodes.machine = { config, pkgs, ... }: {
              imports = [
                # TODO(author): import YOUR Quiqr Server NixOS module:
                #   inputs.quiqr.nixosModules.default
              ];

              # TODO(author): enable + configure Quiqr Server, e.g.:
              #   services.quiqr-server.enable = true;
              #   services.quiqr-server.dataDir = "/var/lib/quiqr";
              #   services.quiqr-server.users = [ ... ];  # the user JSON

              environment.systemPackages = [ pkgs.hugo ];
              virtualisation.memorySize = 2048;
            };

            testScript = ''
              start_all()
              machine.wait_for_unit("multi-user.target")

              # TODO(E7): wait_for_unit for the Quiqr Server service, provision a
              # sample Quiqr site + quiqr/model schema into the data dir, then:
              #   1. run qtui headless (--script) with the fake agent configured,
              #   2. assert: site listed, hugo server reachable, schema Menu has a
              #      Single + Collection, file opens read-only, "Ask AI" injects
              #      intent, fake agent writes a new content file with the
              #      sentinel,
              #   3. assert the new file exists on disk.
              # For E1 this just proves the VM boots.
              machine.succeed("true")
            '';
          };
        };
    };
}
