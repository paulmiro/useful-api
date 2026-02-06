{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.useful-api;
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
  };

  config = lib.mkIf cfg.enable {
    users.users.useful-api = {
      description = "useful-api user";
      group = "useful-api";
      isSystemUser = true;
    };

    users.groups.useful-api = { };

    config.nix.settings.allowed-users = [ "useful-api" ];

    systemd.tmpfiles.rules = [
      "d /var/lib/useful-api 0750 useful-api useful-api - -"
    ];

    systemd.services.useful-api-initial-setup = {
      description = "Initial clone and build of useful-api";
      after = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];
      path = [
        pkgs.git
        pkgs.nix
      ];
      script = ''
        if [ ! -d "/var/lib/useful-api/repo/.git" ]; then
          git clone "${cfg.repoUrl}" "/var/lib/useful-api/repo"
        fi
        if [ ! -e "/var/lib/useful-api/result" ]; then
          export NIX_CACHE_HOME="/var/lib/useful-api/.cache/nix"
          nix build "/var/lib/useful-api/repo#useful-api" -o "/var/lib/useful-api/result"
        fi
      '';
      serviceConfig = {
        Type = "oneshot";
        User = "useful-api";
        Group = "useful-api";
        WorkingDirectory = "/var/lib/useful-api";
        StateDirectory = "useful-api";
        RemainAfterExit = true;
      };
    };

    systemd.services.useful-api = {
      description = "useful-api rocket server";
      after = [ "useful-api-initial-setup.service" ];
      wants = [ "useful-api-initial-setup.service" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        ExecStart = "/var/lib/useful-api/result/bin/useful-api";
        Environment = [ "USEFUL_API_PORT=${toString cfg.port}" ];
        User = "useful-api";
        Group = "useful-api";
        Restart = "on-failure";
        WorkingDirectory = "/var/lib/useful-api";
        StateDirectory = "useful-api";
        ReloadPropagatedFrom = [ "useful-api.path" ];
      };
      partOf = [ "useful-api.path" ];
    };

    systemd.paths."useful-api" = {
      description = "Watch for changes in the useful-api build result";
      wantedBy = [ "multi-user.target" ];
      pathConfig = {
        PathChanged = "/var/lib/useful-api/result";
        Unit = "useful-api.service";
      };
    };

    systemd.services.useful-api-updater = {
      description = "Update useful-api git repository and rebuild";
      path = [
        pkgs.git
        pkgs.nix
      ];
      serviceConfig = {
        Type = "oneshot";
        WorkingDirectory = "/var/lib/useful-api";
        StateDirectory = "useful-api";
        User = "useful-api";
        Group = "useful-api";
        ExecStart = pkgs.writeShellScript "useful-api-updater" ''
          set -euo pipefail
          cd "/var/lib/useful-api"
          git fetch
          if [ "$(git rev-parse HEAD)" != "$(git rev-parse '@{u}')" ]; then
            echo "New changes detected, updating..."
            git pull --rebase --force
            export NIX_CACHE_HOME="/var/lib/useful-api/.cache/nix"
            nix build "/var/lib/useful-api#useful-api" -o "/var/lib/useful-api/result-new"
            # Atomically replace the old binary link
            mv -f "/var/lib/useful-api/result-new" "/var/lib/useful-api/result"
          else
            echo "Already up to date."
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
