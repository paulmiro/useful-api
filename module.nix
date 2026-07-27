{ withSystem, ... }:
{
  flake.nixosModules.default =
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
        enable = lib.mkEnableOption "the useful-api service";
        package = lib.mkOption {
          type = lib.types.package;
          default = withSystem pkgs.stdenv.hostPlatform.system ({ config, ... }: config.packages.default);
          defaultText = lib.literalExpression "self.packages.\${system}.default";
          description = "The useful-api package to run.";
        };
        port = lib.mkOption {
          type = lib.types.port;
          default = 3000;
          description = "Port to listen on. Must be an unprivileged port.";
        };
        bindAddress = lib.mkOption {
          type = lib.types.str;
          default = "0.0.0.0";
          description = "Address to bind to.";
        };
      };

      config = lib.mkIf cfg.enable {
        assertions = [
          {
            assertion = cfg.port >= 1024;
            message = "services.useful-api.port must be >= 1024: the service runs unprivileged and cannot bind privileged ports.";
          }
        ];

        systemd.services.useful-api = {
          description = "useful-api rocket server";
          after = [ "network.target" ];
          wantedBy = [ "multi-user.target" ];
          environment = {
            USEFUL_API_PORT = toString cfg.port;
            USEFUL_API_ADDRESS = cfg.bindAddress;
          };
          serviceConfig = {
            ExecStart = lib.getExe cfg.package;
            DynamicUser = true;
            Restart = "on-failure";
          };
        };
      };
    };
}
