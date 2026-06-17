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
          # Resolution decided in E1 (poc-foundation):
          #   - beans   -> nixpkgs#beans (hmans/beans, the markdown issue
          #                tracker — NOT the unrelated "Rust Type Kit" rtk).
          #   - openspec-> nixpkgs#openspec (Fission-AI/OpenSpec).
          # Both are packaged in nixpkgs-unstable, so no extra flake input or
          # build-from-source derivation is needed for the foundation.
          #
          # The coding agent CLI (`claude`) is intentionally NOT pinned here:
          # it is only exercised by E6/E7, and the deterministic tests use the
          # `fake-agent` (built in-tree) rather than a real LLM. When E6 lands,
          # add the agent + `rmux-sdk` via a pinned input/derivation rather than
          # weakening the flake. Until then the shell ships everything E1–E5
          # need.
          devTools = with pkgs'; [
            rustToolchain
            hugo
            git
            nodejs_22        # runtime some agent/spec tooling expects
            beans            # nixpkgs#beans — agent-first issue tracker
            openspec         # nixpkgs#openspec — spec-driven development CLI
          ];
        in
        {
          _module.args.pkgs = pkgs';

          devShells.default = pkgs'.mkShell {
            packages = devTools;
            shellHook = ''
              echo "quiqr-tui dev shell"
              echo "  rust: $(rustc --version 2>/dev/null)"
              echo "  beans $(beans version 2>/dev/null | head -n1) | openspec $(openspec --version 2>/dev/null)"
              echo "  -> run 'beans prime' and read CLAUDE.md before coding."
            '';
          };

          # `nix build`
          packages.default = pkgs'.rustPlatform.buildRustPackage {
            pname = "quiqr-tui";
            version = "0.0.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;  # exists after E1 scaffolding
          };

          # `nix flake check` runs the workspace unit + integration tests. This
          # is a *check* derivation, not a package: we build the test binaries,
          # run them, and emit a trivial $out (the cargo install hook is skipped
          # because there is nothing to install for a test-only build).
          checks.unit = pkgs'.rustPlatform.buildRustPackage {
            pname = "quiqr-tui-tests";
            version = "0.0.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            doCheck = true;
            buildPhase = "cargo test --workspace --no-run --release";
            checkPhase = "cargo test --workspace --release";
            installPhase = "touch $out";
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
