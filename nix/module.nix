{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.programs.tyt;
  yamlFormat = pkgs.formats.yaml { };
in {
  options.programs.tyt = {
    enable = mkEnableOption "Terminal-yt, a newsboat inspired youtube subs manager";

    config = mkOption {
      inherit (yamlFormat.tyt);
      default = { };
      # defaultText = literalExpression "{ }";
      # example = literalExpression "{ }";
      description = ''
        Config written to ~/.config/tyt/config.yml
      '';
    };

    channels = mkOption {
      type = types.listOf yamlFormat.type;
      default = [ ];
      defaultText = literalExpression "{ }";
      description = ''
        channels for ~/.config/tyt/subscriptions.yml
      '';
    };

    custom_channels = mkOption {
      type = types.listOf yamlFormat.type;
      default = [ ];
      # defaultText = literalExpression "{ }";
      description = ''
        Custom channels ~/.config/tyt/subscriptions.yml
      '';
    };
  };

  config = mkIf cfg.enable {
    home.packages = [ pkgs.tyt ];
    xdg.configFile."tyt/config.yml" = 
      mkIf (cfg.config != { }) {
        source = yamlFormat.generate "tyt-config" cfg.config;
      };

    xdg.configFile."tyt/subscriptions.yml" = 
      mkIf (cfg.channels != { }) {
        source = yamlFormat.generate "tyt-subs" { inherit (cfg) channels custom_channels; };
      };
  };
}

