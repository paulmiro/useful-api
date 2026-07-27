{ config, ... }:
{
  perSystem =
    { pkgs, lib, ... }:
    {
      checks = lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
        vm-test = pkgs.testers.runNixOSTest {
          name = "useful-api";

          nodes.machine = {
            imports = [ config.flake.nixosModules.default ];
            services.useful-api = {
              enable = true;
              port = 3000;
            };
          };

          testScript = ''
            machine.wait_for_unit("useful-api.service")
            machine.wait_for_open_port(3000)

            # curl's user agent is detected as a terminal client
            out = machine.succeed("curl -sS --fail http://localhost:3000/hello")
            assert "Hello, World!" in out, out

            out = machine.succeed("curl -sS --fail 'http://localhost:3000/hello?format=json'")
            assert '"message"' in out and "Hello, World!" in out, out

            # the service runs unprivileged
            assert machine.succeed(
                "systemctl show -p DynamicUser --value useful-api.service"
            ).strip() == "yes"
            assert "root" not in machine.succeed("ps -o user= -C useful-api")
          '';
        };
      };
    };
}
