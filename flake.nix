{
  description = "quiqr-tui (qiosq) — kiosked WP5.1-style TUI CMS frontend for Quiqr";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # rmux source (E7). The repo's `rmux` binary is both the CLI and the hidden
    # daemon that `rmux-sdk` connects to. It has no flake.nix yet, so we consume
    # it as a plain source tree and build the binary ourselves (see `rmux`
    # below). Swap to the official rmux flake's package output once it lands
    # (bean qiosq-rts9).
    rmux = {
      url = "github:mipmip/rmux";
      flake = false;
    };

    # Quiqr Server lives in the author's nixpkgs fork; the e2e VM imports its
    # NixOS module (nixos/modules/services/web-apps/quiqr-server.nix) and uses
    # `quiqr.server`. `quiqr-023` is a branch — pin a rev in flake.lock and bump
    # only on request.
    nixpkgs-quiqr.url = "github:mipmip/nixpkgs/quiqr-023";
  };

  outputs = inputs@{ flake-parts, nixpkgs, nixpkgs-quiqr, rust-overlay, rmux, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];

      perSystem = { self', system, pkgs, lib, ... }:
        let
          pkgs' = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          rustToolchain = pkgs'.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "clippy" "rustfmt" ];
          };

          # The rmux binary (CLI + hidden daemon) built from the source input.
          # `rmux-sdk`'s connect_or_start() spawns this, so it must be on PATH
          # for the real agent path. Interim: build from source until the
          # official rmux flake exposes a package (bean qiosq-rts9).
          rmuxPkg = pkgs'.rustPlatform.buildRustPackage {
            pname = "rmux";
            version = "0.6.1";
            src = rmux;
            cargoLock.lockFile = "${rmux}/Cargo.lock";
            doCheck = false; # upstream's own tests; we only need the binary
          };

          # Quiqr Server packages + module, from the author's nixpkgs fork.
          pkgs-quiqr = import nixpkgs-quiqr {
            inherit system;
            config.allowUnfree = true;
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
            rmuxPkg          # rmux CLI + daemon (for the real agent bridge, E6/E7)
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
          # run qtui headless with the fake agent, and assert the full flow puts
          # new content on disk. The data directory is shared flat files — qtui
          # reads it directly (Quiqr has no API), so the test points qtui's
          # config at the same dataFolder the quiqr-server service uses.
          checks.e2e =
            let
              # Both binaries (qtui + fake-agent) from our package.
              qtuiPkg = self'.packages.default;
              # The anonymized, theme-less real-site fixture (serveable by hugo).
              fixture = ./crates/qtui-storage/tests/fixtures/real-site;
              dataDir = "/var/lib/quiqr";
            in
            pkgs'.testers.runNixOSTest {
              name = "quiqr-tui-e2e";
              nodes.machine = { pkgs, ... }: {
                imports = [
                  "${nixpkgs-quiqr}/nixos/modules/services/web-apps/quiqr-server.nix"
                ];

                services.quiqr-server = {
                  enable = true;
                  package = pkgs-quiqr.quiqr.server;
                  settings.storage = {
                    type = "fs";
                    dataFolder = dataDir;
                  };
                };

                environment.systemPackages = [ pkgs.hugo qtuiPkg ];
                virtualisation.memorySize = 4096;
                virtualisation.diskSize = 4096;
              };

              testScript = ''
                start_all()
                machine.wait_for_unit("multi-user.target")

                # 1. Quiqr Server is up. (The service creates its data folder
                #    lazily on first use, so we do not assert it pre-exists; the
                #    provisioning step below creates it.)
                machine.wait_for_unit("quiqr-server.service")

                # 2. Provision the sample site into the Quiqr data directory.
                machine.succeed("mkdir -p '${dataDir}/examplesite'")
                machine.succeed("cp -r ${fixture}/. '${dataDir}/examplesite/'")
                machine.succeed("test -f '${dataDir}/examplesite/quiqr/model/base.yaml'")

                # 3. Write a qtui config pointing at that data dir, with the
                #    fake-agent (writes a file + prints the sentinel) as the agent.
                machine.succeed("""
                  cat > /root/qtui.toml <<EOF
                [storage]
                quiqr_data_dir = "${dataDir}"

                [preview]
                port_range = [13140, 13160]
                ready_timeout_ms = 60000

                [agent]
                command = "${qtuiPkg}/bin/fake-agent"
                completion_sentinel = "<<QTUI_TASK_DONE>>"
                EOF
                """)

                # 4. Run qtui headless through the full flow. The fake-agent writes
                #    the new content file (QTUI_FAKE_WRITE) when "asked".
                new_file = "${dataDir}/examplesite/content/post/agent-wrote-this.md"
                machine.succeed(
                    f"QTUI_FAKE_WRITE='{new_file}' QTUI_FAKE_SENTINEL='<<QTUI_TASK_DONE>>' "
                    "qtui --config /root/qtui.toml --site examplesite "
                    "--script open-site,open-file,ask-ai,await"
                )

                # 5. Assert the agent's new content file exists on disk.
                machine.succeed(f"test -f '{new_file}'")
                machine.succeed(f"grep -q 'written by fake-agent' '{new_file}'")
              '';
            };
        };
    };
}
