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
    dataDir = lib.mkOption {
      type = lib.types.path;
      default = "/var/lib/useful-api";
      description = "Directory to store the git repository and build artifacts.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.useful-api-initial-setup = {
      description = "Initial clone and build of useful-api";
      after = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];
      script = ''
        if [ ! -d "${cfg.dataDir}/repo/.git" ]; then
          ${pkgs.git}/bin/git clone "${cfg.repoUrl}" "${cfg.dataDir}/repo"
        fi
        if [ ! -e "${cfg.dataDir}/result" ]; then
          ${pkgs.nix}/bin/nix build "${cfg.dataDir}/repo#useful-api" -o "${cfg.dataDir}/result"
        fi
      '';
      serviceConfig = {
        DynamicUser = true;
        Type = "oneshot";
        WorkingDirectory = cfg.dataDir;
        RemainAfterExit = true;
      };
    };

    systemd.services.useful-api = {
      description = "useful-api rocket server";
      after = [ "useful-api-initial-setup.service" ];
      wants = [ "useful-api-initial-setup.service" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        ExecStart = "${cfg.dataDir}/result/bin/useful-api";
        Environment = [ "USEFUL_API_PORT=${toString cfg.port}" ];
        DynamicUser = true;
        Restart = "on-failure";
        WorkingDirectory = cfg.dataDir;
        ReloadPropagatedFrom = [ "useful-api.path" ];
      };
      partOf = [ "useful-api.path" ];
    };

    systemd.paths."useful-api" = {
      description = "Watch for changes in the useful-api build result";
      wantedBy = [ "multi-user.target" ];
      pathConfig = {
        PathChanged = "${cfg.dataDir}/result";
        Unit = "useful-api.service";
      };
    };

    systemd.services.useful-api-updater =
      let
        updateScript = pkgs.writeShellScript "useful-api-updater" ''
          set -euo pipefail
          cd "${cfg.dataDir}"
          ${pkgs.git}/bin/git fetch
          if [ "$(${pkgs.git}/bin/git rev-parse HEAD)" != "$(${pkgs.git}/bin/git rev-parse '@{u}')" ]; then
            echo "New changes detected, updating..."
            ${pkgs.git}/bin/git pull --rebase --force
            ${pkgs.nix}/bin/nix build "${cfg.dataDir}#useful-api" -o "${cfg.dataDir}/result-new"
            # Atomically replace the old binary link
            mv -f "${cfg.dataDir}/result-new" "${cfg.dataDir}/result"
          else
            echo "Already up to date."
          fi
        '';
      in
      {
        description = "Update useful-api git repository and rebuild";
        serviceConfig = {
          Type = "oneshot";
          WorkingDirectory = cfg.dataDir;
          ExecStart = updateScript;
          DynamicUser = true;
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
