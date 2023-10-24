{ index-git-repositories, centerpiece }:
{
  lib,
  pkgs,
  config,
  ...
}:
let
  cfg = config.programs.centerpiece;
in
{
  options.programs.centerpiece = {
    enable = lib.mkEnableOption (lib.mdDoc "Centerpiece");
    services.index-git-repositories = {
      enable = lib.mkEnableOption (lib.mdDoc "enable timer");
      interval = lib.mkOption {
        default = "5min";
        type = lib.types.str;
        example = "hourly";
        description = lib.mdDoc ''
          Frequency of index creation.

          The format is described in
          {manpage}`systemd.time(7)`.
        '';
      };
    };
  };

  config = lib.mkMerge [
    (lib.mkIf cfg.enable { home.packages = [ centerpiece ]; })

    (lib.mkIf cfg.services.index-git-repositories.enable {
      systemd.user = {
        services.index-git-repositories = {
          Unit = {
            Description = "Centerpiece - your trusty";
            Documentation = "https://github.com/friedow/centerpiece";
          };
          script = "${index-git-repositories}/bin/index-git-repositories";
          serviceConfig = {
            Type = "oneshot";
          };
        };
        timers.index-git-repositories = {
          wantedBy = [ "timers.target" ];
          timerConfig.onBootSec = "0m";
          timerConfig.onUnitActiveSec = cfg.services.index-git-repositories.interval;
        };
      };
    })
  ];
}
