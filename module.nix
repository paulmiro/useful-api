{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.useful-api;
  nixBuildCommand = ''nix --extra-experimental-features "nix-command flakes" --accept-flake-config build "./repo#useful-api" -o result'';
in
{
  options.services.useful-api = {
    enable = lib.mkEnableOption "enables the auto-updating useful-api service";
    repoUrl = lib.mkOption {
      type = lib.types.str;
      default = "https://github.com/paulmiro/useful-api";
      description = "URL of the git repository to pull from.";
    };
    port = lib.mkOption {
      type = lib.types.port;
      default = 3000;
      description = "Port to listen on.";
    };
    bindAddress = lib.mkOption {
      type = lib.types.str;
      default = "0.0.0.0";
      description = "Address to bind to.";
    };
  };

  config = lib.mkIf cfg.enable {
    users.users.useful-api = {
      description = "useful-api user";
      group = "useful-api";
      isSystemUser = true;
    };

    users.groups.useful-api = { };

    nix.settings = {
      allowed-users = [ "useful-api" ];
      trusted-substituters = [ "https://useful-api.cachix.org?priority=10" ];
      trusted-public-keys = [ "useful-api.cachix.org-1:vlsTmRlyPE64g57Ti+lBXwTHORQKRR2WmpcyGlv5LnI=" ];
    };

    systemd.tmpfiles.rules = [
      "d /var/lib/useful-api 0750 useful-api useful-api - -"
    ];

    systemd.services.useful-api-initial-setup = {
      description = "Initial clone and build of useful-api";
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];
      path = [
        pkgs.git
        pkgs.nix
      ];
      environment = {
        "NIX_CACHE_HOME" = "/var/lib/useful-api/.cache/nix";
      };
      serviceConfig = {
        Type = "oneshot";
        User = "useful-api";
        Group = "useful-api";
        WorkingDirectory = "/var/lib/useful-api";
        StateDirectory = "useful-api";
        RemainAfterExit = true;

        ExecStart = pkgs.writeShellScript "useful-api-initial-setup" ''
          set -euo pipefail
          if [ ! -d "repo/.git" ]; then
            echo "Cloning ${cfg.repoUrl} into repo..."
            git clone "${cfg.repoUrl}" repo
          else
            echo "Repo already cloned."
          fi
          if [ ! -e result ]; then
            echo "Building..."
            ${nixBuildCommand}
            echo "Building done."
          else
            echo "Result already built."
          fi
          echo "Done."
        '';
      };
    };

    systemd.services.useful-api = {
      description = "useful-api rocket server";
      after = [ "useful-api-initial-setup.service" ];
      wants = [ "useful-api-initial-setup.service" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "/var/lib/useful-api/result/bin/useful-api";
        Environment = [
          "USEFUL_API_PORT=${toString cfg.port}"
          "USEFUL_API_ADDRESS=${cfg.bindAddress}"
        ];
        User = "useful-api";
        Group = "useful-api";
        Restart = "on-failure";
        WorkingDirectory = "/var/lib/useful-api";
        StateDirectory = "useful-api";
      };
    };

    systemd.services.useful-api-restarter = {
      description = "Restart useful-api when the build result changes";
      wants = [ "useful-api-initial-setup.service" ];
      serviceConfig = {
        Type = "oneshot";
        ExecStart = pkgs.writeShellScript "useful-api-restarter" ''
          set -euo pipefail
          echo "Restarting useful-api..."
          systemctl restart useful-api
          echo "Done."
        '';
      };
    };

    systemd.services.useful-api-updater = {
      description = "Update useful-api git repository and rebuild";
      wants = [ "useful-api-initial-setup.service" ];
      onSuccess = [ "useful-api-restarter.service" ];
      path = [
        pkgs.git
        pkgs.nix
      ];
      environment = {
        "NIX_CACHE_HOME" = "/var/lib/useful-api/.cache/nix";
      };
      serviceConfig = {
        Type = "oneshot";
        WorkingDirectory = "/var/lib/useful-api/repo";
        StateDirectory = "useful-api";
        User = "useful-api";
        Group = "useful-api";
        ExecStart = pkgs.writeShellScript "useful-api-updater" ''
          set -euo pipefail
          git fetch
          if [ "$(git rev-parse HEAD)" != "$(git rev-parse '@{u}')" ]; then
            echo "New changes detected, updating..."
            git pull --rebase --force
            cd ..
            echo "Building..."
            ${nixBuildCommand}
            echo "Done."
          else
            echo "Already up to date."
            exit 1
          fi
        '';
      };
    };

    systemd.timers.useful-api-updater = {
      description = "Run useful-api-updater every 5 minutes";
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnBootSec = "1m";
        OnUnitActiveSec = "5m";
        Unit = "useful-api-updater.service";
      };
    };
  };
}
