self:
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.clippit;
  inherit (lib) mkOption mkEnableOption types;

  defaultPackage = self.packages.${pkgs.stdenv.hostPlatform.system}.default;

  confValue = types.oneOf [
    types.bool
    types.int
    types.float
    types.str
    (types.listOf types.str)
  ];

  tomlFormat = pkgs.formats.toml { };

  knownKeys = builtins.fromJSON (builtins.readFile ./known-keys.json);

  leafPaths =
    prefix: attrs:
    lib.concatLists (
      lib.mapAttrsToList (
        name: value:
        let
          path = prefix ++ [ name ];
        in
        if lib.isAttrs value then leafPaths path value else [ (lib.concatStringsSep "." path) ]
      ) attrs
    );

  unknownKeys =
    known: keys: lib.filter (key: !(lib.elem key known)) keys;

  unknownSettings = unknownKeys knownKeys.corretor (lib.attrNames cfg.settings);
  unknownHistorySettings = unknownKeys knownKeys.config (leafPaths [ ] cfg.historySettings);

  plan = pkgs.writeText "clippit-settings.json" (
    builtins.toJSON {
      inherit (cfg) settings historySettings;
      freeAltSpace = cfg.freeAltSpace;
      extension = if cfg.gnomeExtension then cfg.package.passthru.extensionUuid else null;
      historyShortcut =
        if cfg.historyShortcut == null then
          null
        else
          {
            # The profile path survives updates; a store path would go stale.
            command = "${config.home.profileDirectory}/bin/clippit-historico";
            binding = cfg.historyShortcut;
          };
      dconf = lib.getExe pkgs.dconf;
      systemctl = "${pkgs.systemd}/bin/systemctl";
    }
  );

  python = pkgs.python3.withPackages (ps: [ ps.tomli-w ]);

  # systemd's PATH for user services does not include the user's profile, and
  # the doctor needs gnome-extensions, systemctl and gsettings from there.
  servicePath = lib.concatStringsSep ":" [
    "${config.home.profileDirectory}/bin"
    "/run/current-system/sw/bin"
    "/usr/local/bin"
    "/usr/bin"
    "/bin"
  ];
in
{
  options.programs.clippit = {
    enable = mkEnableOption "Clippit, the clipboard history, launcher and local AI for GNOME";

    package = mkOption {
      type = types.package;
      default = defaultPackage;
      defaultText = lib.literalExpression "clippit.packages.\${system}.default";
      example = lib.literalExpression ''
        clippit.packages.''${system}.default.override {
          llama-cpp = pkgs.llama-cpp.override { vulkanSupport = true; };
        }
      '';
      description = "The Clippit package. Override `llama-cpp` for a GPU build of the AI, or set `withAI = false`.";
    };

    quickshell = mkOption {
      type = types.bool;
      default = false;
      description = ''
        Run the Quickshell interface (`clippit-shell`) instead of the GNOME extension, for Hyprland,
        Sway, niri and other wlroots-style compositors: launcher, spell checker next to the cursor,
        reply assistant and AI chat. It registers its own shortcuts with Hyprland and Sway. The
        compositor must export WAYLAND_DISPLAY (and HYPRLAND_INSTANCE_SIGNATURE) to systemd.
        On a GNOME-only install, set `package = pkgs.clippit.override { withQuickshell = false; }` to leave
        Quickshell (and Qt) out of the closure.
      '';
    };

    gnomeExtension = mkOption {
      type = types.bool;
      default = true;
      description = ''
        Turn on the GNOME Shell extension (launcher, spell checker, AI). It is added to
        `org.gnome.shell.enabled-extensions` without touching the other extensions.
      '';
    };

    historyShortcut = mkOption {
      type = types.nullOr types.str;
      default = "<Alt>c";
      description = "GNOME shortcut that opens the history. `null` leaves the GNOME shortcuts alone.";
    };

    freeAltSpace = mkOption {
      type = types.bool;
      default = true;
      description = "Free Alt+Space (GNOME's window menu) for the launcher.";
    };

    popupAutostart = mkOption {
      type = types.bool;
      default = true;
      description = "Start the history popup hidden at login, so Alt+C opens instantly.";
    };

    doctor = mkOption {
      type = types.bool;
      default = true;
      description = "Run `clippit-doutor` every 2 minutes to find and fix problems.";
    };

    settings = mkOption {
      type = types.attrsOf confValue;
      default = { };
      example = lib.literalExpression ''
        {
          device = "auto";
          ia_doutor = false;
          nomes = [ "Maria Silva" "Maria" ];
          atalho_ia = "<Super>i";
        }
      '';
      description = ''
        Keys of `~/.config/clippit/corretor.conf`: AI, models, spell checker and shortcuts.
        Every key is listed in `docs/configuration.md`. Lists become comma-separated values.
        Only these keys are written; what you change in the settings app stays, except these
        keys, which go back to the Nix value on the next `home-manager switch`.
      '';
    };

    historySettings = mkOption {
      inherit (tomlFormat) type;
      default = { };
      example = lib.literalExpression ''
        {
          general.max_history_items = 500;
          features.enable_ocr = false;
          ui.language = "pt";
        }
      '';
      description = ''
        Keys of `~/.config/clippit/config.toml`: history, privacy, OCR and language
        (see `docs/configuration.md`). Merged into the file the same way as `settings`;
        the daemon restarts when they change.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = unknownSettings == [ ];
        message = "programs.clippit.settings: unknown keys ${lib.concatStringsSep ", " unknownSettings} (see docs/configuration.md).";
      }
      {
        assertion = unknownHistorySettings == [ ];
        message = "programs.clippit.historySettings: unknown keys ${lib.concatStringsSep ", " unknownHistorySettings} (see docs/configuration.md).";
      }
      {
        assertion = lib.all (v: !(lib.isString v) || !(lib.hasInfix "#" v)) (lib.attrValues cfg.settings);
        message = "programs.clippit.settings: values cannot contain `#`, corretor.conf reads it as a comment.";
      }
      {
        assertion = !(cfg.settings ? brave_api_key);
        message = ''
          programs.clippit.settings.brave_api_key would be copied to the world-readable
          /nix/store. Set the key in the Clippit settings app (or in
          ~/.config/clippit/corretor.conf) instead.
        '';
      }
    ];

    home.packages = [ cfg.package ];

    xdg.configFile."autostart/clippit-popup.desktop" = lib.mkIf cfg.popupAutostart {
      source = "${cfg.package}/share/clippit/clippit-popup-autostart.desktop";
    };

    systemd.user.services.clippit = {
      Unit = {
        Description = "Clippit Clipboard Manager";
        After = [ "graphical-session.target" ];
      };
      Service = {
        ExecStart = lib.getExe' cfg.package "clippit-daemon";
        Restart = "on-failure";
        RestartSec = "5s";
        Environment = [
          "RUST_LOG=info"
          "PATH=${servicePath}"
        ];
      };
      Install.WantedBy = [ "default.target" ];
    };

    systemd.user.services.clippit-shell = lib.mkIf cfg.quickshell {
      Unit = {
        Description = "Interface do Clippit (Quickshell, fora do GNOME)";
        PartOf = [ "graphical-session.target" ];
        After = [ "graphical-session.target" ];
        # Without WAYLAND_DISPLAY in the user manager the shell can only fail: do not loop.
        ConditionEnvironment = "WAYLAND_DISPLAY";
        StartLimitIntervalSec = 60;
        StartLimitBurst = 5;
      };
      Service = {
        ExecStart = lib.getExe' cfg.package "clippit-shell";
        Restart = "on-failure";
        RestartSec = "3s";
        Environment = [ "PATH=${servicePath}" ];
      };
      Install.WantedBy = [ "graphical-session.target" ];
    };

    systemd.user.services.clippit-doutor = lib.mkIf cfg.doctor {
      Unit.Description = "Doutor do Clippit (acha e corrige problemas do Clippit)";
      Service = {
        Type = "oneshot";
        ExecStart = lib.getExe' cfg.package "clippit-doutor";
        Nice = 10;
        TimeoutStartSec = 300;
        Environment = [ "PATH=${servicePath}" ];
      };
    };

    systemd.user.timers.clippit-doutor = lib.mkIf cfg.doctor {
      Unit.Description = "Roda o doutor do Clippit a cada 2 minutos";
      Timer = {
        OnStartupSec = "90s";
        OnUnitActiveSec = "2min";
        AccuracySec = "15s";
      };
      Install.WantedBy = [ "timers.target" ];
    };

    home.activation.clippit = lib.hm.dag.entryAfter [ "writeBoundary" "dconfSettings" ] ''
      run ${python.interpreter} ${./apply-settings.py} ${plan}
    '';
  };
}
