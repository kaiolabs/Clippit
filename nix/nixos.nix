self:
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.clippit;
in
{
  options.programs.clippit = {
    enable = lib.mkEnableOption "Clippit for every user (the settings stay per user: see the Home Manager module)";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      defaultText = lib.literalExpression "clippit.packages.\${system}.default";
      description = "The Clippit package.";
    };

    doctor = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Run `clippit-doutor` every 2 minutes to find and fix problems.";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    # The units come from the package (lib/systemd/user).
    systemd.packages = [ cfg.package ];
    systemd.user.services.clippit = {
      wantedBy = [ "default.target" ];
      path = [ "/run/current-system/sw" ];
    };
    systemd.user.services.clippit-doutor.path = [ "/run/current-system/sw" ];
    systemd.user.timers.clippit-doutor.wantedBy = lib.mkIf cfg.doctor [ "timers.target" ];
  };
}
