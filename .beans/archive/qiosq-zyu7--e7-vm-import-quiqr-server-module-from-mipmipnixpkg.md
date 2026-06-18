---
# qiosq-zyu7
title: 'E7 VM: import quiqr-server module from mipmip/nixpkgs fork'
status: completed
type: task
priority: normal
tags:
    - discovered
created_at: 2026-06-17T23:07:27Z
updated_at: 2026-06-18T00:39:07Z
parent: qiosq-s0ui
---

## Finding (realizable — investigated against the author's remote-install flake)
The Quiqr Server NixOS module lives INSIDE the nixpkgs fork, directly importable into a runNixOSTest VM — no elastinix/Terraform/EC2/AWS needed (that flake only wraps it for prod). Studied: /home/pim/tcCustomers/aws-technative/technative-awsaccounts-workloads/stack/ec2_compute5/{flake.nix,nix/quiqr.nix}.

## Recipe for flake.nix (replaces the TODO(author) in checks.e2e)
- Input: `nixpkgs-quiqr.url = "github:mipmip/nixpkgs/quiqr-023";` (pin a rev once stable). `elastinix.lib.run_as_vm` confirms the same config runs as a local QEMU VM.
- In the VM node:
  imports = [ "${nixpkgs-quiqr}/nixos/modules/services/web-apps/quiqr-server.nix" ];  (path EXISTS — verified)
  services.quiqr-server.enable = true;
  services.quiqr-server.package = pkgs-quiqr.quiqr.server;  (pkgs-quiqr = import nixpkgs-quiqr { allowUnfree = true; })
  services.quiqr-server.settings.storage = { type = "fs"; dataFolder = "/var/lib/quiqr"; };
  (skip auth/nginx/restic/age — production-only; bare service is enough for e2e)

## Module options (real, from quiqr-server.nix in the fork)
- port (default 5150) = Quiqr web UI; settings.storage.dataFolder (default ~/Quiqr) = the data dir qtui-storage enumerates; settings.preview.baseUrl default http://localhost:13131 (the port we avoid); type=fs; binaryPath; disableAutoHugoServe; serveDraftMode; settings.variables (e.g. NIX_EXEC).

## E7 plan
wait_for_unit the quiqr-server service, provision the notnix fixture into dataFolder, run qtui headless (--script) with the fake-agent, assert on-disk result. Per docs, the e2e uses the FAKE agent (not rmux/real LLM), so this VM half is NOT blocked on the rmux daemon (qiosq-rts9).

## Summary of Changes
Realized in E7: checks.e2e imports ${nixpkgs-quiqr}/nixos/modules/services/web-apps/quiqr-server.nix, enables services.quiqr-server (fs storage, dataFolder /var/lib/quiqr), and the VM boots Quiqr Server + runs the full qtui flow. nix flake check green.
